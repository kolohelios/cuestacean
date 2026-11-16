use std::{path::Path, process::Command, sync::Arc};

use bootstrap_executor::{
    Error, execute, execute_json, generated, parser, runtime, scanner, unification,
};
use runtime::Value;
use serde_json::{Value as Json, json};

#[path = "../../generated/logic-false.rs"]
mod false_policy;

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("export")
}
fn text(s: &str) -> Value {
    Value::Text(Arc::from(s))
}
fn binary(kind: &str, left: Json, right: Json) -> Json {
    json!({"kind":kind,"left":left,"right":right})
}
fn integer(n: i64) -> Json {
    json!({"kind":"integer","value":n})
}
fn boolean(b: bool) -> Json {
    json!({"kind":if b { "true" } else { "false" }})
}
fn tree(entry: Json) -> String {
    json!({"formatVersion":1,"entry":entry}).to_string()
}
fn both(entry: Json) -> [Result<Json, Error>; 2] {
    let input = tree(entry);
    [
        execute_json(&input, &export("evaluation")),
        generated::execute_json(&input),
    ]
}
type Operator = fn(Value, Value) -> Result<Value, Error>;
fn operators() -> [(&'static str, &'static str, Operator, bool); 2] {
    [
        ("logicalAnd", "logical-and", runtime::logical_and, false),
        ("logicalOr", "logical-or", runtime::logical_or, true),
    ]
}

#[test]
fn concrete_selection_and_abstract_rejection() {
    for (kind, name, op, short) in operators() {
        let control = export(name);
        for left in [false, true] {
            for right in [false, true] {
                let expected = if kind == "logicalAnd" {
                    left && right
                } else {
                    left || right
                };
                for result in [
                    op(Value::Boolean(left), Value::Boolean(right)),
                    unification::interpret(&control, Value::Boolean(left), Value::Boolean(right)),
                ] {
                    assert_eq!(result.expect("boolean selection"), Value::Boolean(expected));
                }
                for result in both(binary(kind, boolean(left), boolean(right))) {
                    assert_eq!(result.expect("JSON boolean selection"), json!(expected));
                }
            }
        }
        for right in [Value::Integer(1), text("x")] {
            for left in [short, !short] {
                for result in [
                    op(Value::Boolean(left), right.clone()),
                    unification::interpret(&control, Value::Boolean(left), right.clone()),
                ] {
                    if left == short {
                        assert_eq!(
                            result.expect("ignored concrete right"),
                            Value::Boolean(short)
                        );
                    } else {
                        assert!(matches!(result, Err(Error::Logical)));
                    }
                }
            }
            for other in [
                Value::Boolean(false),
                Value::Boolean(true),
                Value::Integer(2),
                text("y"),
            ] {
                for result in [
                    op(right.clone(), other.clone()),
                    unification::interpret(&control, right.clone(), other),
                ] {
                    assert!(matches!(result, Err(Error::Logical)), "non-boolean left");
                }
            }
        }
        for abstract_value in [
            Value::Bool,
            Value::Int,
            Value::String,
            Value::Bounds {
                lower: Some(1),
                upper: Some(1),
            },
        ] {
            for (left, right) in [
                (Value::Boolean(short), abstract_value.clone()),
                (abstract_value.clone(), Value::Boolean(short)),
                (abstract_value.clone(), abstract_value),
            ] {
                for result in [
                    op(left.clone(), right.clone()),
                    unification::interpret(&control, left.clone(), right.clone()),
                ] {
                    let error = result.expect_err("abstract operand");
                    assert!(matches!(error, Error::Logical));
                    assert_eq!(error.to_string(), "invalid logical operands");
                }
            }
        }
    }
}

#[test]
fn alternate_policy_and_boolean_condition_validation() {
    let left = Value::Boolean(true);
    let right = Value::Boolean(false);
    assert_eq!(
        false_policy::logical_or(left.clone(), right.clone()).expect("compiled alternate"),
        Value::Boolean(false)
    );
    assert_eq!(
        unification::interpret(
            &export("tests/valid/logic-false"),
            left.clone(),
            right.clone()
        )
        .expect("reference alternate"),
        Value::Boolean(false)
    );
    assert_eq!(
        runtime::logical_or(left.clone(), right.clone()).expect("default"),
        Value::Boolean(true)
    );
    for (input, expected) in [("left", true), ("right", false)] {
        let program = json!({"formatVersion":1,"function":"logicalOr","result":{"op":"if",
            "condition":{"op":"booleanIs","input":input,"value":true},
            "then":{"op":"boolean","value":true},"else":{"op":"boolean","value":false}}})
        .to_string();
        assert!(unification::generate(&program).is_ok());
        assert_eq!(
            unification::interpret(&program, left.clone(), right.clone()).expect("condition"),
            Value::Boolean(expected)
        );
    }
    for condition in [
        json!({"op":"booleanIs","input":"left"}),
        json!({"op":"booleanIs","input":"left","value":null}),
        json!({"op":"booleanIs","input":"left","value":"true"}),
        json!({"op":"booleanIs","input":"middle","value":true}),
        json!({"op":"booleanIs","input":null,"value":true}),
        json!({"op":"booleanIs","input":"left","value":true,"extra":0}),
    ] {
        // Invalid conditions must be validated even in an unused branch.
        let invalid = json!({"formatVersion":1,"function":"logicalOr","result":{"op":"if",
            "condition":{"op":"same"},"then":{"op":"boolean","value":true},
            "else":{"op":"if","condition":condition,"then":{"op":"boolean","value":true},"else":{"op":"boolean","value":false}}}}).to_string();
        assert!(unification::generate(&invalid).is_err(), "{invalid}");
        assert!(
            unification::interpret(&invalid, left.clone(), left.clone()).is_err(),
            "{invalid}"
        );
    }
    let invalid = json!({"formatVersion":1,"function":"logicalOr","result":{"op":"if",
        "condition":{"op":"booleanIs","input":"left","value":null},
        "then":{"op":"boolean","value":true},"else":{"op":"boolean","value":false}}})
    .to_string();
    let path = std::env::temp_dir().join(format!("logic-invalid-{}.json", std::process::id()));
    std::fs::write(&path, invalid).expect("write");
    let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-unify-codegen"))
        .arg(&path)
        .output()
        .expect("CLI");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
    std::fs::remove_file(path).expect("remove");
}

#[test]
fn children_are_eager_and_left_first_before_operand_checks() {
    let zero = binary("divide", integer(1), integer(0));
    let overflow = binary("add", integer(i64::MAX), integer(1));
    let conflict = binary("unify", boolean(true), boolean(false));
    for (kind, _, _, short) in operators() {
        for (left, right, message) in [
            (boolean(short), zero.clone(), "integer division by zero"),
            (
                boolean(short),
                overflow.clone(),
                "signed 64-bit integer addition overflow",
            ),
            (
                boolean(short),
                conflict.clone(),
                "conflicting boolean values true and false",
            ),
            (zero.clone(), conflict.clone(), "integer division by zero"),
            (
                conflict.clone(),
                zero.clone(),
                "conflicting boolean values true and false",
            ),
            (integer(1), zero.clone(), "integer division by zero"),
        ] {
            for result in both(binary(kind, left, right)) {
                assert_eq!(result.expect_err("child error").to_string(), message);
            }
        }
        let input = tree(binary(kind, boolean(short), boolean(false)));
        assert!(matches!(
            execute(&input, &export("evaluation")),
            Err(Error::IntegerOutput)
        ));
        assert!(matches!(
            generated::execute(&input),
            Err(Error::IntegerOutput)
        ));
    }
}

#[test]
fn source_precedence_associativity_and_scanner() {
    let grammar = export("grammar");
    let pratt = export("pratt");
    let t = boolean(true);
    let f = boolean(false);
    for (source, entry) in [
        (
            "true & false || true && 1 == 1 + 2 * 3",
            binary(
                "unify",
                t.clone(),
                binary(
                    "logicalOr",
                    f.clone(),
                    binary(
                        "logicalAnd",
                        t.clone(),
                        binary(
                            "equal",
                            integer(1),
                            binary(
                                "add",
                                integer(1),
                                binary("multiply", integer(2), integer(3)),
                            ),
                        ),
                    ),
                ),
            ),
        ),
        (
            "true || false || true",
            binary(
                "logicalOr",
                binary("logicalOr", t.clone(), f.clone()),
                t.clone(),
            ),
        ),
        (
            "true && false && true",
            binary(
                "logicalAnd",
                binary("logicalAnd", t.clone(), f.clone()),
                t.clone(),
            ),
        ),
        (
            "1 < 2 == true",
            binary("equal", binary("less", integer(1), integer(2)), t),
        ),
    ] {
        let expected: Json = serde_json::from_str(&tree(entry)).expect("AST");
        for result in [
            parser::parse(source, &grammar),
            parser::parse_interpreted(source, &grammar, &pratt),
        ] {
            assert_eq!(result.expect("parse"), expected, "{source}");
        }
    }
    for (source, offset, reason) in [
        ("true | false", 5, "unexpected or unsupported syntax"),
        ("true &&& false", 7, "expected signed decimal integer"),
        ("true ||| false", 7, "expected signed decimal integer"),
    ] {
        let expected = format!("source byte {offset}: {reason}");
        for parsed in [
            parser::parse(source, &grammar),
            parser::parse_interpreted(source, &grammar, &pratt),
        ] {
            assert_eq!(
                parsed.expect_err("invalid logical symbol").to_string(),
                expected
            );
        }
    }
    let control = export("infix");
    for first in [b'&', b'|'] {
        for second in 0_u8..=127 {
            let source = String::from_utf8(vec![first, second]).expect("ASCII");
            let end = if first == second {
                2
            } else if first == b'&' {
                1
            } else {
                0
            };
            for result in [
                scanner::infix(&source, 0),
                scanner::interpret(&control, &source, 0),
            ] {
                let scan = result.expect("scan");
                assert_eq!((scan.offset, scan.flag), (end, end != 0), "{source:?}");
            }
        }
    }
    for (source, expected) in [("&&&", (3, true)), ("|||", (2, false))] {
        for result in [
            scanner::infix(source, 2),
            scanner::interpret(&control, source, 2),
        ] {
            let scan = result.expect("remaining symbol");
            assert_eq!((scan.offset, scan.flag), expected, "{source}");
        }
    }
    for (source, end, flag) in [
        ("+&", 1, true),
        ("&&<", 2, true),
        ("||<", 2, true),
        ("div|", 3, true),
        ("=", 0, false),
        ("&&&", 2, true),
        ("|||", 2, true),
        ("|", 0, false),
        ("&", 1, true),
    ] {
        for result in [
            scanner::infix(source, 0),
            scanner::interpret(&control, source, 0),
        ] {
            let scan = result.expect("scan");
            assert_eq!((scan.offset, scan.flag), (end, flag), "{source}");
        }
    }
}

#[test]
fn source_cli_success_and_failure_are_atomic() {
    let path = std::env::temp_dir().join(format!("logic-source-{}.cue", std::process::id()));
    for (source, expected, error) in [
        ("false && 1", "false\n", ""),
        ("true || \"x\"", "true\n", ""),
        ("true && 1", "", "invalid logical operands"),
        ("false || int", "", "invalid logical operands"),
        ("false && (1 div 0 == 0)", "", "integer division by zero"),
        ("true || (1 div 0 == 0)", "", "integer division by zero"),
    ] {
        std::fs::write(&path, source).expect("write");
        let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-source"))
            .arg(&path)
            .output()
            .expect("CLI");
        assert_eq!(output.status.success(), !expected.is_empty(), "{source}");
        assert_eq!(output.stdout, expected.as_bytes(), "{source}");
        if error.is_empty() {
            assert!(output.stderr.is_empty());
        } else {
            assert!(
                String::from_utf8_lossy(&output.stderr).contains(error),
                "{source}"
            );
        }
    }
    std::fs::remove_file(path).expect("remove");
}
