use std::{path::Path, process::Command, sync::Arc};

use bootstrap_executor::{
    Error, execute, execute_json, generated, parser, runtime, scanner, unification,
};
use runtime::Value;
use serde_json::{Value as Json, json};

#[path = "../../generated/less-false.rs"]
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
fn int(n: i64) -> Json {
    json!({"kind":"integer","value":n})
}
fn binary(kind: &str, left: Json, right: Json) -> Json {
    json!({"kind":kind,"left":left,"right":right})
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
fn operators() -> [(&'static str, &'static str, Operator); 4] {
    [
        ("less", "less", runtime::less),
        ("lessEqual", "less-equal", runtime::less_equal),
        ("greater", "greater", runtime::greater),
        ("greaterEqual", "greater-equal", runtime::greater_equal),
    ]
}

#[test]
fn independent_truth_tables_and_invalid_domains() {
    // Explicit expected order, including extrema and UTF-8 without normalization.
    let rows = [
        (
            Value::Integer(i64::MIN),
            Value::Integer(i64::MAX),
            [true, true, false, false],
        ),
        (
            Value::Integer(i64::MAX - 1),
            Value::Integer(i64::MAX),
            [true, true, false, false],
        ),
        (
            Value::Integer(-1),
            Value::Integer(0),
            [true, true, false, false],
        ),
        (
            Value::Integer(1),
            Value::Integer(1),
            [false, true, false, true],
        ),
        (
            Value::Integer(0),
            Value::Integer(-1),
            [false, false, true, true],
        ),
        (text(""), text("a"), [true, true, false, false]),
        (text("10"), text("2"), [true, true, false, false]),
        (text("é"), text("e\u{301}"), [false, false, true, true]),
        (text("🦀"), text("🦀"), [false, true, false, true]),
    ];
    for (index, (_, name, op)) in operators().into_iter().enumerate() {
        let control = export(name);
        for (left, right, expected) in &rows {
            for result in [
                op(left.clone(), right.clone()),
                unification::interpret(&control, left.clone(), right.clone()),
            ] {
                assert_eq!(
                    result.expect("comparison"),
                    Value::Boolean(expected[index]),
                    "{name}: {left:?} {right:?}"
                );
            }
        }
        for invalid in [
            Value::Boolean(false),
            Value::Boolean(true),
            Value::Bool,
            Value::Int,
            Value::String,
            Value::Bounds {
                lower: Some(1),
                upper: Some(1),
            },
            Value::Bounds {
                lower: None,
                upper: Some(2),
            },
        ] {
            for other in [invalid.clone(), Value::Integer(1), text("1")] {
                for (left, right) in [(invalid.clone(), other.clone()), (other, invalid.clone())] {
                    for result in [
                        op(left.clone(), right.clone()),
                        unification::interpret(&control, left, right),
                    ] {
                        let error = result.expect_err("invalid ordering");
                        assert!(matches!(error, Error::Ordering));
                        assert_eq!(
                            error.to_string(),
                            "ordering requires matching concrete integers or strings"
                        );
                    }
                }
            }
        }
        for (left, right) in [
            (Value::Integer(1), text("1")),
            (text("1"), Value::Integer(1)),
        ] {
            assert!(matches!(
                op(left.clone(), right.clone()),
                Err(Error::Ordering)
            ));
            assert!(matches!(
                unification::interpret(&control, left, right),
                Err(Error::Ordering)
            ));
        }
    }
}

#[test]
fn alternate_compiled_policy_and_typed_conditions() {
    let (left, right) = (Value::Integer(1), Value::Integer(2));
    assert_eq!(
        false_policy::less(left.clone(), right.clone()).expect("compiled alternate"),
        Value::Boolean(false)
    );
    assert_eq!(
        unification::interpret(
            &export("tests/valid/less-false"),
            left.clone(),
            right.clone()
        )
        .expect("interpreted alternate"),
        Value::Boolean(false)
    );
    assert_eq!(
        runtime::less(left.clone(), right.clone()).expect("production"),
        Value::Boolean(true)
    );
    for (relation, expected) in [
        ("less", true),
        ("lessEqual", true),
        ("greater", false),
        ("greaterEqual", false),
        ("equal", false),
    ] {
        let control = json!({"formatVersion":1,"function":"less","result":{"op":"if",
            "condition":{"op":"textCompare","relation":relation},
            "then":{"op":"boolean","value":true},"else":{"op":"boolean","value":false}}})
        .to_string();
        assert!(unification::generate(&control).is_ok(), "{relation}");
        assert_eq!(
            unification::interpret(&control, text("a"), text("b")).expect("text relation"),
            Value::Boolean(expected)
        );
        assert!(matches!(
            unification::interpret(&control, left.clone(), right.clone()),
            Err(Error::Ordering)
        ));
    }
    for condition in [
        json!({"op":"textCompare"}),
        json!({"op":"textCompare","relation":"unknown"}),
        json!({"op":"textCompare","relation":1}),
        json!({"op":"textCompare","relation":"less","extra":true}),
    ] {
        let control = json!({"formatVersion":1,"function":"less","result":{"op":"if",
            "condition":{"op":"and","left":{"op":"same"},"right":condition},
            "then":{"op":"boolean","value":true},"else":{"op":"boolean","value":false}}})
        .to_string();
        assert!(unification::generate(&control).is_err(), "{control}");
        assert!(unification::interpret(&control, left.clone(), right.clone()).is_err());
    }
}

#[test]
fn json_children_are_eager_left_first_and_boolean_output_is_not_i64() {
    let overflow = binary("add", int(i64::MAX), int(1));
    let conflict = binary("unify", json!({"kind":"true"}), json!({"kind":"false"}));
    for (kind, _, _) in operators() {
        for (left, right, message) in [
            (
                overflow.clone(),
                conflict.clone(),
                "signed 64-bit integer addition overflow",
            ),
            (
                conflict.clone(),
                overflow.clone(),
                "conflicting boolean values true and false",
            ),
            (
                json!({"kind":"intType"}),
                overflow.clone(),
                "signed 64-bit integer addition overflow",
            ),
        ] {
            for result in both(binary(kind, left, right)) {
                assert_eq!(result.expect_err("child error").to_string(), message);
            }
        }
        let entry = binary(kind, int(1), int(2));
        for result in both(entry.clone()) {
            assert_eq!(
                result.expect("JSON ordering"),
                json!(kind == "less" || kind == "lessEqual")
            );
        }
        let input = tree(entry);
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
fn source_ast_precedence_bounds_and_scanner() {
    let grammar = export("grammar");
    let pratt = export("pratt");
    for (source, entry) in [
        ("1 > 2", binary("greater", int(1), int(2))),
        (
            "1 + 1 < 3",
            binary("less", binary("add", int(1), int(1)), int(3)),
        ),
        (
            "true & 1 <= 1",
            binary(
                "unify",
                json!({"kind":"true"}),
                binary("lessEqual", int(1), int(1)),
            ),
        ),
        (
            "1 < 2 == true",
            binary(
                "equal",
                binary("less", int(1), int(2)),
                json!({"kind":"true"}),
            ),
        ),
        (
            "2 > 1 >= 0",
            binary("greaterEqual", binary("greater", int(2), int(1)), int(0)),
        ),
        (
            ">=1 < 2",
            binary("less", json!({"kind":"lowerBound","value":1}), int(2)),
        ),
        (
            "1 <= <=2",
            binary("lessEqual", int(1), json!({"kind":"upperBound","value":2})),
        ),
    ] {
        let expected: Json = serde_json::from_str(&tree(entry)).expect("expected AST");
        for result in [
            parser::parse(source, &grammar),
            parser::parse_interpreted(source, &grammar, &pratt),
        ] {
            assert_eq!(result.expect("source AST"), expected, "{source}");
        }
    }
    for source in [">=1 < 2", "1 <= <=2"] {
        let entry = parser::parse(source, &grammar).expect("source bounds")["entry"].clone();
        for result in both(entry) {
            assert!(matches!(result, Err(Error::Ordering)), "{source}");
        }
    }
    let path = std::env::temp_dir().join(format!("ordering-{}.cue", std::process::id()));
    for (source, offset) in [("1 <== 2", 4), ("1 >== 2", 4), (">1", 0)] {
        let expected = format!("source byte {offset}: expected signed decimal integer");
        for parsed in [
            parser::parse(source, &grammar),
            parser::parse_interpreted(source, &grammar, &pratt),
        ] {
            assert_eq!(parsed.expect_err("invalid symbol").to_string(), expected);
        }
    }
    for (source, expected) in [
        ("1 <= 1", "true\n"),
        ("2 > 3", "false\n"),
        ("false < true", ""),
        ("1 < \"2\"", ""),
    ] {
        std::fs::write(&path, source).expect("source file");
        let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-source"))
            .arg(&path)
            .output()
            .expect("CLI");
        assert_eq!(output.status.success(), !expected.is_empty());
        assert_eq!(output.stdout, expected.as_bytes());
        if expected.is_empty() {
            assert!(
                String::from_utf8_lossy(&output.stderr)
                    .contains("ordering requires matching concrete integers or strings")
            );
        } else {
            assert!(output.stderr.is_empty());
        }
    }
    std::fs::remove_file(path).expect("remove source file");
    let control = export("infix");
    for (source, offset, flag) in [
        ("<", 1, true),
        (">", 1, true),
        ("<=", 2, true),
        (">=", 2, true),
        ("<==", 2, true),
        (">==", 2, true),
        ("=", 0, false),
        ("!", 0, false),
        ("+<", 1, true),
        ("->", 1, true),
        ("==<", 2, true),
        ("!=>", 2, true),
        ("div<", 3, true),
    ] {
        for result in [
            scanner::infix(source, 0),
            scanner::interpret(&control, source, 0),
        ] {
            let scan = result.expect("operator scan");
            assert_eq!((scan.offset, scan.flag), (offset, flag), "{source}");
        }
    }
}
