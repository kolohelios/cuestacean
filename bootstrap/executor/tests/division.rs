use std::path::{Path, PathBuf};
use std::process::Command;

use bootstrap_executor::{Error, execute, parser, runtime};
use runtime::Value as RuntimeValue;
use serde_json::{Value, json};

#[path = "../../generated/evaluation.rs"]
mod generated;

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("read export")
}

fn source(category: &str, name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../source/{category}/{name}.cue"))
}

fn integer(value: i64) -> Value {
    json!({"kind":"integer", "value":value})
}

fn binary(kind: &str, left: Value, right: Value) -> Value {
    json!({"kind":kind, "left":left, "right":right})
}

fn program(entry: Value) -> Value {
    json!({"formatVersion":1, "entry":entry})
}

fn both(tree: &Value) -> [Result<i64, Error>; 2] {
    let text = tree.to_string();
    [
        execute(&text, &export("evaluation")),
        generated::execute(&text),
    ]
}

const ZERO: &str = "integer division by zero";
const OVERFLOW: &str = "signed 64-bit integer division overflow";
const ABSTRACT: &str = "arithmetic requires concrete integer operands";

#[test]
fn independent_signs_and_boundaries() {
    use RuntimeValue::Integer;
    type Primitive = fn(RuntimeValue, RuntimeValue) -> Result<RuntimeValue, Error>;
    type Examples = &'static [(i64, i64, i64)];
    let cases: [(&str, Primitive, Examples); 4] = [
        (
            "divide",
            runtime::divide,
            &[
                (7, 3, 2),
                (-7, 3, -3),
                (7, -3, -2),
                (-7, -3, 3),
                (i64::MIN, i64::MIN, 1),
            ],
        ),
        (
            "modulo",
            runtime::modulo,
            &[
                (7, 3, 1),
                (-7, 3, 2),
                (7, -3, 1),
                (-7, -3, 2),
                (i64::MIN, -1, 0),
                (-1, i64::MIN, i64::MAX),
                (i64::MIN, i64::MIN, 0),
            ],
        ),
        (
            "quotient",
            runtime::quotient,
            &[(7, 3, 2), (-7, 3, -2), (7, -3, -2), (-7, -3, 2)],
        ),
        (
            "remainder",
            runtime::remainder,
            &[
                (7, 3, 1),
                (-7, 3, -1),
                (7, -3, 1),
                (-7, -3, -1),
                (i64::MIN, -1, 0),
            ],
        ),
    ];
    for (kind, primitive, values) in cases {
        for &(left, right, expected) in values {
            assert_eq!(
                primitive(Integer(left), Integer(right)).expect("runtime result"),
                Integer(expected),
                "{kind} {left} {right}"
            );
            let tree = program(binary(kind, integer(left), integer(right)));
            for result in both(&tree) {
                assert_eq!(result.expect("tree result"), expected, "{tree}");
            }
        }
        let error = primitive(Integer(1), Integer(0)).expect_err("zero divisor");
        assert!(matches!(error, Error::DivisionByZero));
        assert_eq!(error.to_string(), ZERO);
        for result in both(&program(binary(kind, integer(1), integer(0)))) {
            assert_eq!(result.expect_err("zero divisor").to_string(), ZERO);
        }
        let bound = RuntimeValue::Bounds {
            lower: Some(0),
            upper: Some(0),
        };
        for (left, right) in [
            (RuntimeValue::Int, Integer(0)),
            (Integer(1), RuntimeValue::Int),
            (bound, Integer(1)),
        ] {
            assert!(matches!(primitive(left, right), Err(Error::Arithmetic)));
        }
        for entry in [
            binary(kind, json!({"kind":"intType"}), integer(0)),
            binary(kind, integer(1), json!({"kind":"intType"})),
        ] {
            for result in both(&program(entry)) {
                assert_eq!(result.expect_err("abstract operand").to_string(), ABSTRACT);
            }
        }
    }
    for (kind, primitive) in [
        ("divide", runtime::divide as Primitive),
        ("quotient", runtime::quotient as Primitive),
    ] {
        let error = primitive(Integer(i64::MIN), Integer(-1)).expect_err("overflow");
        assert!(matches!(error, Error::DivisionOverflow));
        assert_eq!(error.to_string(), OVERFLOW);
        for result in both(&program(binary(kind, integer(i64::MIN), integer(-1)))) {
            assert_eq!(result.expect_err("overflow").to_string(), OVERFLOW);
        }
    }
    for result in both(&serde_json::from_str(&export("division")).expect("division export")) {
        assert_eq!(result.expect("division export result"), 42);
    }
}

#[test]
fn sampled_integer_division_identities_hold_at_extrema() {
    type Op = fn(RuntimeValue, RuntimeValue) -> Result<RuntimeValue, Error>;
    let values = [
        i64::MIN,
        i64::MIN + 1,
        -9,
        -3,
        -1,
        0,
        1,
        3,
        9,
        i64::MAX - 1,
        i64::MAX,
    ];
    for a in values {
        for b in values.into_iter().filter(|b| *b != 0) {
            for (quotient, remainder, euclidean) in [
                (runtime::divide as Op, runtime::modulo as Op, true),
                (runtime::quotient as Op, runtime::remainder as Op, false),
            ] {
                let r = runtime::concrete(
                    remainder(RuntimeValue::Integer(a), RuntimeValue::Integer(b))
                        .expect("remainder always representable"),
                )
                .expect("concrete remainder");
                if a == i64::MIN && b == -1 {
                    assert_eq!(r, 0);
                    assert!(matches!(
                        quotient(RuntimeValue::Integer(a), RuntimeValue::Integer(b)),
                        Err(Error::DivisionOverflow)
                    ));
                    continue;
                }
                let q = runtime::concrete(
                    quotient(RuntimeValue::Integer(a), RuntimeValue::Integer(b)).expect("quotient"),
                )
                .expect("concrete quotient");
                // Wide arithmetic is only an independent test oracle, not the runtime.
                assert_eq!(i128::from(q) * i128::from(b) + i128::from(r), i128::from(a));
                assert!(i128::from(r).abs() < i128::from(b).abs());
                if euclidean {
                    assert!(r >= 0);
                } else {
                    assert!(r == 0 || r.signum() == a.signum());
                }
            }
        }
    }
}

#[test]
fn children_evaluate_left_first_before_primitive_checks() {
    let zero = binary("divide", integer(1), integer(0));
    let overflow = binary("quotient", integer(i64::MIN), integer(-1));
    let conflict = binary("unify", integer(1), integer(2));
    for kind in ["divide", "modulo", "quotient", "remainder"] {
        for (left, right, expected) in [
            (zero.clone(), overflow.clone(), ZERO),
            (overflow.clone(), zero.clone(), OVERFLOW),
            (
                conflict.clone(),
                zero.clone(),
                "conflicting integer values 1 and 2",
            ),
            (json!({"kind":"intType"}), zero.clone(), ZERO),
            (
                integer(1),
                conflict.clone(),
                "conflicting integer values 1 and 2",
            ),
        ] {
            for result in both(&program(binary(kind, left, right))) {
                assert_eq!(result.expect_err("child failure").to_string(), expected);
            }
        }
    }
}

#[test]
fn fixtures_parse_evaluate_and_cli() {
    let grammar = export("grammar");
    for (name, expected) in [
        ("div-sign", 3),
        ("div-euclidean", -3),
        ("div-minimum", 1),
        ("mod-sign", 2),
        ("mod-minimum", i64::MAX),
        ("mod-overflow-pair", 0),
        ("quo-sign", 2),
        ("quo-glued", 4),
        ("quo-signed-glued", -2),
        ("quo-zero-result", 0),
        ("rem-overflow-pair", 0),
        ("rem-sign", -1),
        ("division-precedence", 12),
        ("division-unify", 2),
        ("division-negation", 4),
    ] {
        let path = source("valid", name);
        let text = std::fs::read_to_string(&path).expect("fixture");
        let tree = parser::parse(&text, &grammar).expect("parse fixture");
        let reference = parser::parse_interpreted_with_primary(
            &text,
            &grammar,
            &export("pratt"),
            &export("primary"),
        )
        .expect("reference parser");
        assert_eq!(tree, reference, "{name}");
        for result in both(&tree) {
            assert_eq!(result.expect("evaluate fixture"), expected, "{name}");
        }
        let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-source"))
            .arg(path)
            .output()
            .expect("source CLI");
        assert!(output.status.success(), "{name}: {output:?}");
        assert_eq!(output.stdout, format!("{expected}\n").as_bytes());
        assert!(output.stderr.is_empty());
    }
    assert_eq!(
        parser::parse("7quo-3", &grammar).expect("signed RHS token is not consumed"),
        program(binary("quotient", integer(7), integer(-3)))
    );
    let expected = program(binary(
        "multiply",
        binary("divide", integer(20), integer(3)),
        integer(2),
    ));
    assert_eq!(
        parser::parse("20 div 3 * 2", &grammar).expect("left associativity"),
        expected
    );
    for (category, name, message) in [
        ("rejected", "div-overflow", OVERFLOW),
        ("rejected", "quo-overflow", OVERFLOW),
        ("conflicts", "division-zero", ZERO),
        ("conflicts", "mod-zero", ZERO),
        ("conflicts", "quo-zero", ZERO),
        ("conflicts", "rem-zero", ZERO),
        ("incomplete", "division-int", ABSTRACT),
        ("incomplete", "mod-bound", ABSTRACT),
        ("conflicts", "division-refine", "conflicting integer"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-source"))
            .arg(source(category, name))
            .output()
            .expect("source CLI");
        assert_eq!(output.status.code(), Some(2), "{name}: {output:?}");
        assert!(output.stdout.is_empty(), "{name}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(message),
            "{name}: {output:?}"
        );
    }
    for name in [
        "division-partial",
        "division-extended",
        "quo-digit",
        "quo-underscore",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-source"))
            .arg(source("rejected", name))
            .output()
            .expect("source CLI");
        assert_eq!(output.status.code(), Some(2), "{name}: {output:?}");
        assert!(output.stdout.is_empty(), "{name}");
    }
}

#[test]
fn keyword_boundaries_and_runtime_metadata() {
    let mut grammar: Value = serde_json::from_str(&export("grammar")).expect("grammar");
    for (index, token, kind) in [
        (4, "div", "divide"),
        (5, "mod", "modulo"),
        (6, "quo", "quotient"),
        (7, "rem", "remainder"),
    ] {
        assert_eq!(grammar["operators"][index]["token"], token);
        assert_eq!(grammar["operators"][index]["kind"], kind);
        assert_eq!(grammar["operators"][index]["precedence"], 20);
        assert_eq!(grammar["operators"][index]["associativity"], "left");
        for text in [
            format!("8 {token}x 2"),
            format!("8 {token}2 2"),
            format!("8 {token}_foo 2"),
        ] {
            assert!(
                parser::parse(&text, &grammar.to_string()).is_err(),
                "{text}"
            );
        }
        let text = format!("8 {token} 2");
        assert_eq!(
            parser::parse(&text, &grammar.to_string()).expect("exact token"),
            program(binary(kind, integer(8), integer(2)))
        );
        let mut disabled = grammar.clone();
        disabled["operators"]
            .as_array_mut()
            .expect("operators")
            .remove(index);
        assert!(
            parser::parse(&text, &disabled.to_string()).is_err(),
            "disabled {token}"
        );
    }
    for text in ["8 qu 2", "8 di 2", "8 mo 2", "8 re 2"] {
        assert!(parser::parse(text, &grammar.to_string()).is_err(), "{text}");
    }
    grammar["operators"][4]["associativity"] = "right".into();
    let tree = parser::parse("24 div 6 div 2", &grammar.to_string()).expect("right associative");
    for result in both(&tree) {
        assert_eq!(result.expect("right associative result"), 8);
    }
    grammar["operators"][4]["precedence"] = 30.into();
    let tree = parser::parse("24 div 3 * 2", &grammar.to_string()).expect("custom precedence");
    for result in both(&tree) {
        assert_eq!(result.expect("custom precedence result"), 16);
    }
    grammar["operators"][4]["kind"] = "customDivide".into();
    grammar["operators"][4]["leftField"] = "lhs".into();
    grammar["operators"][4]["rightField"] = "rhs".into();
    assert_eq!(
        parser::parse("8 div 2", &grammar.to_string()).expect("custom constructor")["entry"],
        json!({"kind":"customDivide", "lhs":integer(8), "rhs":integer(2)})
    );
}
