use std::{path::Path, process::Command, sync::Arc};

use bootstrap_executor::{
    Error, execute, execute_json, generated, parser, runtime, scanner, unification,
};
use runtime::Value;
use serde_json::{Value as Json, json};

#[path = "../../generated/equal-false.rs"]
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
fn tree(entry: Json) -> String {
    json!({"formatVersion":1,"entry":entry}).to_string()
}
fn integer(n: i64) -> Json {
    json!({"kind":"integer","value":n})
}
fn both(entry: Json) -> [Result<Json, Error>; 2] {
    let input = tree(entry);
    [
        execute_json(&input, &export("evaluation")),
        generated::execute_json(&input),
    ]
}

#[test]
fn concrete_truth_matrix_and_content_equality() {
    let eq = export("equal");
    let ne = export("not-equal");
    // Group IDs are independent expected equality classes, not Value::eq.
    let shared = text("shared");
    let samples = [
        (Value::Integer(1), 0),
        (Value::Integer(2), 1),
        (Value::Integer(i64::MIN), 2),
        (Value::Integer(i64::MAX), 3),
        (Value::Boolean(true), 4),
        (Value::Boolean(false), 5),
        (text(""), 6),
        (text("é"), 7),
        (text("e\u{301}"), 8),
        (shared.clone(), 9),
        (shared, 9),
        (text("shared"), 9),
        (text("1"), 10),
    ];
    for (left, l) in &samples {
        for (right, r) in &samples {
            for (control, expected, actual) in [
                (&eq, l == r, runtime::equal(left.clone(), right.clone())),
                (&ne, l != r, runtime::not_equal(left.clone(), right.clone())),
            ] {
                assert_eq!(
                    actual.expect("compiled"),
                    Value::Boolean(expected),
                    "{left:?} {right:?}"
                );
                assert_eq!(
                    unification::interpret(control, left.clone(), right.clone())
                        .expect("interpreted"),
                    Value::Boolean(expected),
                    "{left:?} {right:?}"
                );
            }
        }
    }
}

#[test]
fn abstract_operands_are_rejected_even_when_identical() {
    let controls = [export("equal"), export("not-equal")];
    let abstracts = [
        Value::String,
        Value::Bool,
        Value::Int,
        Value::Bounds {
            lower: Some(1),
            upper: Some(1),
        },
        Value::Bounds {
            lower: None,
            upper: Some(2),
        },
    ];
    for a in abstracts {
        for b in [
            a.clone(),
            Value::Integer(1),
            Value::Boolean(true),
            text("x"),
        ] {
            for (left, right) in [(a.clone(), b.clone()), (b, a.clone())] {
                for result in [
                    runtime::equal(left.clone(), right.clone()),
                    runtime::not_equal(left.clone(), right.clone()),
                    unification::interpret(&controls[0], left.clone(), right.clone()),
                    unification::interpret(&controls[1], left, right),
                ] {
                    let error = result.expect_err("non-concrete comparison");
                    assert!(matches!(error, Error::Comparison));
                    assert_eq!(error.to_string(), "comparison requires concrete operands");
                }
            }
        }
    }
}

#[test]
fn compiled_alternate_and_wire_validation() {
    let left = Value::Integer(1);
    let control = export("tests/valid/equal-false");
    assert_eq!(
        false_policy::equal(left.clone(), left.clone()).expect("alternate"),
        Value::Boolean(false)
    );
    assert_eq!(
        unification::interpret(&control, left.clone(), left.clone())
            .expect("interpreted alternate"),
        Value::Boolean(false)
    );
    assert_eq!(
        runtime::equal(left.clone(), left.clone()).expect("production"),
        Value::Boolean(true)
    );
    for invalid in [
        r#"{"formatVersion":1,"function":"equal","result":{"op":"boolean"}}"#,
        r#"{"formatVersion":1,"function":"equal","result":{"op":"boolean","value":null}}"#,
        r#"{"formatVersion":1,"function":"equal","result":{"op":"boolean","value":"false"}}"#,
        r#"{"formatVersion":1,"function":"equal","result":{"op":"boolean","value":false,"extra":0}}"#,
        r#"{"formatVersion":1,"function":"not_equal","result":{"op":"boolean","value":false}}"#,
        r#"{"formatVersion":1,"function":"equal","result":{"op":"fail","reason":"unknown"}}"#,
        r#"{"formatVersion":1,"function":"equal","result":{"op":"if","condition":{"op":"same"},"then":{"op":"boolean","value":true},"else":{"op":"boolean","value":null}}}"#,
    ] {
        assert!(unification::generate(invalid).is_err(), "{invalid}");
        assert!(
            unification::interpret(invalid, left.clone(), left.clone()).is_err(),
            "{invalid}"
        );
    }
    let path = std::env::temp_dir().join(format!("equality-invalid-{}.json", std::process::id()));
    std::fs::write(
        &path,
        r#"{"formatVersion":1,"function":"equal","result":{"op":"boolean","value":null}}"#,
    )
    .expect("write");
    let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-unify-codegen"))
        .arg(&path)
        .output()
        .expect("CLI");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    std::fs::remove_file(path).expect("remove");
}

#[test]
fn json_instructions_evaluate_children_left_first() {
    let overflow = binary("add", integer(i64::MAX), integer(1));
    let conflict = binary("unify", json!({"kind":"true"}), json!({"kind":"false"}));
    for op in ["equal", "notEqual"] {
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
            for result in both(binary(op, left, right)) {
                assert_eq!(result.expect_err("child error").to_string(), message);
            }
        }
        let entry = binary(op, integer(1), integer(1));
        for result in both(entry.clone()) {
            assert_eq!(result.expect("comparison"), json!(op == "equal"));
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
fn source_precedence_associativity_and_cli() {
    let grammar = export("grammar");
    let pratt = export("pratt");
    for (source, entry, expected) in [
        (
            "1 + 1 == 2",
            binary("equal", binary("add", integer(1), integer(1)), integer(2)),
            true,
        ),
        (
            "true & 1 == 1",
            binary(
                "unify",
                json!({"kind":"true"}),
                binary("equal", integer(1), integer(1)),
            ),
            true,
        ),
        (
            "1 == 1 == true",
            binary(
                "equal",
                binary("equal", integer(1), integer(1)),
                json!({"kind":"true"}),
            ),
            true,
        ),
        ("1 != 1", binary("notEqual", integer(1), integer(1)), false),
        (
            "1 == \"1\"",
            binary("equal", integer(1), json!({"kind":"string","value":"1"})),
            false,
        ),
    ] {
        let expected_tree: Json = serde_json::from_str(&tree(entry.clone())).expect("tree");
        for parsed in [
            parser::parse(source, &grammar),
            parser::parse_interpreted(source, &grammar, &pratt),
        ] {
            assert_eq!(parsed.expect("parse"), expected_tree, "{source}");
        }
        for result in both(entry) {
            assert_eq!(result.expect("evaluate"), json!(expected));
        }
    }
    let path = std::env::temp_dir().join(format!("equality-source-{}.cue", std::process::id()));
    for (source, offset, reason) in [
        ("1 = 1", 2, "unexpected or unsupported syntax"),
        ("1 ! 1", 2, "unexpected or unsupported syntax"),
        ("1 === 1", 4, "expected signed decimal integer"),
        ("1 !== 1", 4, "expected signed decimal integer"),
    ] {
        let expected = format!("source byte {offset}: {reason}");
        for parsed in [
            parser::parse(source, &grammar),
            parser::parse_interpreted(source, &grammar, &pratt),
        ] {
            assert_eq!(
                parsed.expect_err("invalid symbol").to_string(),
                expected,
                "{source}"
            );
        }
        std::fs::write(&path, source).expect("invalid source");
        let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-source"))
            .arg(&path)
            .output()
            .expect("CLI");
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains(&expected));
    }
    for (source, output) in [
        ("1 == 1", Some("true\n")),
        ("1 != 1", Some("false\n")),
        ("int == 1", None),
        ("1 === 1", None),
    ] {
        std::fs::write(&path, source).expect("write source");
        let result = Command::new(env!("CARGO_BIN_EXE_bootstrap-source"))
            .arg(&path)
            .output()
            .expect("CLI");
        if let Some(expected) = output {
            assert!(result.status.success(), "{result:?}");
            assert_eq!(result.stdout, expected.as_bytes());
            assert!(result.stderr.is_empty());
        } else {
            assert!(!result.status.success());
            assert!(result.stdout.is_empty());
            assert!(!result.stderr.is_empty());
        }
    }
    std::fs::remove_file(path).expect("remove");
}

#[test]
fn infix_scanners_consume_only_complete_symbols() {
    let control = export("infix");
    for (source, end, flag) in [
        ("==", 2, true),
        ("!=", 2, true),
        ("=", 0, false),
        ("!", 0, false),
        ("===", 2, true),
        ("!==", 2, true),
    ] {
        for result in [
            scanner::infix(source, 0),
            scanner::interpret(&control, source, 0),
        ] {
            let result = result.expect("scan");
            assert_eq!(result.offset, end, "{source}");
            assert_eq!(result.flag, flag, "{source}");
        }
    }
}
