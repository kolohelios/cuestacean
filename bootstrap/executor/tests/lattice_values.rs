use std::{path::Path, process::Command, sync::Arc};

use bootstrap_executor::{Error, execute, execute_json, generated, parser, runtime, unification};
use runtime::Value;
use serde_json::{Value as Json, json};
use unification::{Endpoint, Kind};

#[path = "../../generated/unify-right.rs"]
mod right_policy;

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("export")
}

fn tree(entry: Json) -> String {
    json!({"formatVersion":1,"entry":entry}).to_string()
}
fn leaf(kind: &str) -> Json {
    json!({"kind":kind})
}
fn binary(kind: &str, left: Json, right: Json) -> Json {
    json!({"kind":kind,"left":left,"right":right})
}
fn both(entry: Json) -> [Result<Json, Error>; 2] {
    let input = tree(entry);
    [
        execute_json(&input, &export("evaluation")),
        generated::execute_json(&input),
    ]
}
fn samples() -> Vec<Value> {
    vec![
        Value::Top,
        Value::Null,
        Value::Int,
        Value::Integer(7),
        Value::Bounds {
            lower: None,
            upper: None,
        },
        Value::Bounds {
            lower: Some(-3),
            upper: Some(8),
        },
        Value::Bounds {
            lower: Some(0),
            upper: None,
        },
        Value::Bounds {
            lower: None,
            upper: Some(1),
        },
        Value::Bool,
        Value::Boolean(false),
        Value::Boolean(true),
        Value::String,
        Value::Text(Arc::from("shared 🦀")),
    ]
}

#[test]
fn top_identity_and_null_guard_cover_every_domain_and_order() {
    let control = export("unify");
    for value in samples() {
        for (left, right) in [(Value::Top, value.clone()), (value.clone(), Value::Top)] {
            for result in [
                runtime::unify(left.clone(), right.clone()),
                unification::interpret(&control, left, right),
            ] {
                assert_eq!(result.expect("top identity"), value);
            }
        }
        if value != Value::Top && value != Value::Null {
            for (left, right) in [(Value::Null, value.clone()), (value.clone(), Value::Null)] {
                for result in [
                    runtime::unify(left.clone(), right.clone()),
                    unification::interpret(&control, left, right),
                ] {
                    assert!(matches!(result, Err(Error::TypeConflict)), "{value:?}");
                }
            }
        }
    }
    for result in [
        runtime::unify(Value::Null, Value::Null),
        unification::interpret(&control, Value::Null, Value::Null),
    ] {
        assert_eq!(result.expect("null identity"), Value::Null);
    }
}

#[test]
fn checked_tags_endpoints_and_shared_text_ownership() {
    for (value, kind, tag) in [
        (Value::Null, Kind::Null, "null"),
        (Value::Top, Kind::Top, "top"),
    ] {
        assert!(unification::kind_is(&value, kind));
        assert_eq!(serde_json::to_value(kind).expect("kind tag"), json!(tag));
        assert_eq!(unification::endpoint(&value, Endpoint::Lower), None);
        assert_eq!(unification::endpoint(&value, Endpoint::Upper), None);
    }
    let original: Arc<str> = Arc::from("retained text");
    let value = Value::Text(original.clone());
    for (left, right) in [(Value::Top, value.clone()), (value.clone(), Value::Top)] {
        let selected = runtime::unify(left, right).expect("selected text");
        match selected {
            Value::Text(text) => assert!(Arc::ptr_eq(&original, &text)),
            other => panic!("unexpected value {other:?}"),
        }
    }
    let selected = runtime::unify(Value::Top, value).expect("selected text");
    drop(original);
    assert_eq!(selected, Value::Text(Arc::from("retained text")));
}

#[test]
fn compiled_alternate_really_changes_top_policy() {
    let left = Value::Integer(1);
    let right = Value::Top;
    assert_eq!(
        right_policy::unify(left.clone(), right.clone()).expect("compiled alternate"),
        Value::Top
    );
    assert_eq!(
        unification::interpret(
            &export("tests/valid/unify-right"),
            left.clone(),
            right.clone()
        )
        .expect("reference alternate"),
        Value::Top
    );
    assert_eq!(
        runtime::unify(left, right).expect("production"),
        Value::Integer(1)
    );
}

#[test]
fn output_and_legacy_boundaries() {
    for entry in [leaf("null"), binary("unify", leaf("top"), leaf("null"))] {
        for result in both(entry.clone()) {
            assert_eq!(result.expect("JSON null"), Json::Null);
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
    for entry in [leaf("top"), binary("unify", leaf("top"), leaf("top"))] {
        for result in both(entry) {
            let error = result.expect_err("incomplete top");
            assert!(matches!(error, Error::IncompleteValue));
            assert_eq!(error.to_string(), "non-concrete value");
        }
    }
    for (value, message) in [
        (leaf("intType"), "non-concrete int value"),
        (leaf("stringType"), "non-concrete string value"),
        (leaf("boolType"), "non-concrete bool value"),
        (
            json!({"kind":"lowerBound","value":1}),
            "non-concrete integer bounds",
        ),
    ] {
        for (left, right) in [(leaf("top"), value.clone()), (value, leaf("top"))] {
            for result in both(binary("unify", left, right)) {
                assert_eq!(
                    result.expect_err("incomplete selected value").to_string(),
                    message
                );
            }
        }
    }
}

#[test]
fn equality_logic_and_other_operators_do_not_coerce() {
    for (kind, expected) in [("equal", true), ("notEqual", false)] {
        for result in both(binary(kind, leaf("null"), leaf("null"))) {
            assert_eq!(result.expect("null equality"), json!(expected));
        }
        for other in [
            leaf("true"),
            json!({"kind":"integer","value":1}),
            json!({"kind":"string","value":"x"}),
        ] {
            for (left, right) in [(leaf("null"), other.clone()), (other, leaf("null"))] {
                for result in both(binary(kind, left, right)) {
                    assert_eq!(result.expect("mixed equality"), json!(!expected));
                }
            }
        }
        for (left, right) in [
            (leaf("top"), leaf("null")),
            (leaf("null"), leaf("top")),
            (leaf("top"), leaf("top")),
        ] {
            for result in both(binary(kind, left, right)) {
                assert!(matches!(result, Err(Error::Comparison)));
            }
        }
    }
    for (kind, short, selected) in [
        ("logicalAnd", "false", "true"),
        ("logicalOr", "true", "false"),
    ] {
        for result in both(binary(kind, leaf(short), leaf("null"))) {
            assert_eq!(result.expect("ignored null"), json!(short == "true"));
        }
        for (left, right) in [
            (leaf(selected), leaf("null")),
            (leaf("null"), leaf(short)),
            (leaf(short), leaf("top")),
            (leaf("top"), leaf(short)),
        ] {
            for result in both(binary(kind, left, right)) {
                assert!(matches!(result, Err(Error::Logical)));
            }
        }
    }
    for kind in [
        "add",
        "subtract",
        "multiply",
        "divide",
        "modulo",
        "quotient",
        "remainder",
    ] {
        for value in ["null", "top"] {
            for result in both(binary(
                kind,
                leaf(value),
                json!({"kind":"integer","value":1}),
            )) {
                assert!(matches!(result, Err(Error::Arithmetic)), "{kind} {value}");
            }
        }
    }
    for kind in ["less", "lessEqual", "greater", "greaterEqual"] {
        for value in ["null", "top"] {
            for other in [
                json!({"kind":"integer","value":1}),
                json!({"kind":"string","value":"1"}),
            ] {
                for (left, right) in [(leaf(value), other.clone()), (other, leaf(value))] {
                    for result in both(binary(kind, left, right)) {
                        assert!(matches!(result, Err(Error::Ordering)));
                    }
                }
            }
        }
    }
    for value in ["null", "top"] {
        for result in both(json!({"kind":"logicalNot","operand":leaf(value)})) {
            assert!(matches!(result, Err(Error::Logical)));
        }
    }
}

#[test]
fn children_fail_before_enclosing_control() {
    let zero = binary(
        "divide",
        json!({"kind":"integer","value":1}),
        json!({"kind":"integer","value":0}),
    );
    for kind in ["logicalAnd", "logicalOr", "unify"] {
        for (left, right) in [(leaf("null"), zero.clone()), (zero.clone(), leaf("top"))] {
            for result in both(binary(kind, left, right)) {
                assert_eq!(
                    result.expect_err("child failure").to_string(),
                    "integer division by zero"
                );
            }
        }
    }
}

#[test]
fn source_atoms_boundaries_and_cli_output() {
    let grammar = export("grammar");
    let pratt = export("pratt");
    for (source, entry) in [
        ("null", leaf("null")),
        ("_", leaf("top")),
        ("_ & null", binary("unify", leaf("top"), leaf("null"))),
    ] {
        let expected: Json = serde_json::from_str(&tree(entry)).expect("AST");
        for result in [
            parser::parse(source, &grammar),
            parser::parse_interpreted(source, &grammar, &pratt),
        ] {
            assert_eq!(result.expect("source AST"), expected);
        }
    }
    for source in ["nullfoo", "_x"] {
        for result in [
            parser::parse(source, &grammar),
            parser::parse_interpreted(source, &grammar, &pratt),
        ] {
            assert!(
                result
                    .expect_err("complete unknown atom")
                    .to_string()
                    .contains("unknown atom")
            );
        }
    }
    let path = std::env::temp_dir().join(format!("lattice-source-{}.cue", std::process::id()));
    for (source, output, error) in [
        ("null", "null\n", ""),
        ("_ & null", "null\n", ""),
        ("null == 1", "false\n", ""),
        ("false && null", "false\n", ""),
        ("_", "", "non-concrete value"),
        ("_ & int", "", "non-concrete int value"),
        ("_ == null", "", "comparison requires concrete operands"),
    ] {
        std::fs::write(&path, source).expect("write source");
        let result = Command::new(env!("CARGO_BIN_EXE_bootstrap-source"))
            .arg(&path)
            .output()
            .expect("CLI");
        assert_eq!(result.stdout, output.as_bytes(), "{source}");
        assert_eq!(result.status.success(), error.is_empty(), "{source}");
        if error.is_empty() {
            assert!(result.stderr.is_empty());
        } else {
            assert!(
                String::from_utf8_lossy(&result.stderr).contains(error),
                "{source}"
            );
        }
    }
    std::fs::remove_file(path).expect("remove source");
}
