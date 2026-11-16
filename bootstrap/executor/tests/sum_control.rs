use std::{path::Path, process::Command, sync::Arc};

use bootstrap_executor::{Error, execute_json, generated, parser, runtime, unification};
use runtime::Value;
use serde_json::{Value as Json, json};

#[path = "../../generated/sum-right.rs"]
mod right;

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("export")
}

fn text(s: &str) -> Value {
    Value::Text(Arc::from(s))
}

fn results(control: &str, left: Value, right: Value) -> [Result<Value, Error>; 2] {
    [
        runtime::sum(left.clone(), right.clone()),
        unification::interpret(control, left, right),
    ]
}

#[test]
fn sum_policy_has_fixed_scalar_and_error_oracles() {
    let control = export("sum");
    for (left, right, expected) in [
        (0, 0, 0),
        (20, 22, 42),
        (-9, 4, -5),
        (i64::MAX, 0, i64::MAX),
        (i64::MIN, 0, i64::MIN),
        (i64::MAX, i64::MIN, -1),
    ] {
        for result in results(&control, Value::Integer(left), Value::Integer(right)) {
            assert_eq!(result.expect("integer sum"), Value::Integer(expected));
        }
    }
    for (left, right) in [(i64::MAX, 1), (i64::MIN, -1)] {
        for result in results(&control, Value::Integer(left), Value::Integer(right)) {
            assert!(matches!(result, Err(Error::Overflow)));
        }
    }
    for (left, right, expected) in [
        ("", "", ""),
        ("", "🦀", "🦀"),
        ("é", "e\u{301}", "ée\u{301}"),
        ("quote\"\\", "\n\t\0", "quote\"\\\n\t\0"),
    ] {
        for result in results(&control, text(left), text(right)) {
            assert_eq!(result.expect("text sum"), text(expected));
        }
    }
    let invalid = [
        Value::Bool,
        Value::Boolean(true),
        Value::Int,
        Value::String,
        Value::Bounds {
            lower: Some(0),
            upper: None,
        },
    ];
    for value in invalid {
        for other in [Value::Integer(1), text("x"), value.clone()] {
            for (left, right) in [(value.clone(), other.clone()), (other, value.clone())] {
                for result in results(&control, left, right) {
                    assert!(matches!(result, Err(Error::Arithmetic)));
                }
            }
        }
    }
    for (left, right) in [
        (text("x"), Value::Integer(1)),
        (Value::Integer(1), text("x")),
    ] {
        for result in results(&control, left, right) {
            assert!(matches!(result, Err(Error::Arithmetic)));
        }
    }
}

#[test]
fn concatenation_does_not_mutate_shared_inputs_and_owns_output() {
    let control = export("sum");
    let left = text("left");
    let shared = left.clone();
    for result in results(&control, left.clone(), text(" right")) {
        let output = result.expect("concatenation");
        if let (Value::Text(input), Value::Text(alias), Value::Text(output)) =
            (&left, &shared, &output)
        {
            assert!(Arc::ptr_eq(input, alias));
            assert!(!Arc::ptr_eq(input, output));
            assert_eq!(&**input, "left");
            assert_eq!(&**output, "left right");
        } else {
            panic!("expected text");
        }
    }
}

#[test]
fn alternate_policy_and_wire_validation() {
    let control = export("tests/valid/sum-right");
    let left = Value::Integer(20);
    let rhs = Value::Integer(22);
    assert_eq!(
        right::sum(left.clone(), rhs.clone()).expect("compiled alternate"),
        rhs
    );
    assert_eq!(
        unification::interpret(&control, left.clone(), rhs.clone()).expect("alternate"),
        rhs
    );
    assert_eq!(
        runtime::sum(left.clone(), rhs.clone()).expect("production"),
        Value::Integer(42)
    );
    let default = r#"{"formatVersion":1,"result":{"op":"input","input":"right"}}"#;
    assert_eq!(
        unification::interpret(default, left.clone(), rhs.clone()).expect("default"),
        rhs
    );
    assert!(
        unification::generate(default)
            .expect("default generation")
            .contains("fn unify(")
    );
    assert!(
        unification::generate(&control)
            .expect("sum generation")
            .contains("fn sum(")
    );
    for invalid in [
        r#"{"formatVersion":1,"function":null,"result":{"op":"input","input":"left"}}"#,
        r#"{"formatVersion":1,"function":"sum;evil","result":{"op":"input","input":"left"}}"#,
        r#"{"formatVersion":1,"function":7,"result":{"op":"input","input":"left"}}"#,
        r#"{"formatVersion":1,"function":"sum","result":{"op":"integerSum","extra":1}}"#,
        r#"{"formatVersion":1,"function":"sum","result":{"op":"textConcat","extra":1}}"#,
        r#"{"formatVersion":1,"function":"sum","result":{"op":"unknown"}}"#,
        r#"{"formatVersion":1,"function":"sum","result":{"op":"fail","reason":"unknown"}}"#,
        r#"{"formatVersion":1,"function":"sum","function":"unify","result":{"op":"input","input":"left"}}"#,
    ] {
        assert!(unification::generate(invalid).is_err(), "{invalid}");
        assert!(
            unification::interpret(invalid, left.clone(), rhs.clone()).is_err(),
            "{invalid}"
        );
    }
}

fn binary(kind: &str, left: Json, right: Json) -> Json {
    json!({"kind":kind,"left":left,"right":right})
}
fn tree(entry: Json) -> String {
    json!({"formatVersion":1,"entry":entry}).to_string()
}
fn evaluate(entry: Json) -> [Result<Json, Error>; 2] {
    let input = tree(entry);
    [
        execute_json(&input, &export("evaluation")),
        generated::execute_json(&input),
    ]
}

#[test]
fn ast_addition_and_left_first_failures() {
    let string = |s: &str| json!({"kind":"string","value":s});
    let integer = |n: i64| json!({"kind":"integer","value":n});
    for result in evaluate(binary("add", string("é\n"), string("e\u{301}🦀"))) {
        assert_eq!(result.expect("JSON addition"), json!("é\ne\u{301}🦀"));
    }
    let overflow = binary("add", integer(i64::MAX), integer(1));
    let conflict = binary("unify", string("a"), string("b"));
    for (left, right, expected) in [
        (
            overflow.clone(),
            conflict.clone(),
            "signed 64-bit integer addition overflow",
        ),
        (
            conflict,
            overflow.clone(),
            "conflicting string values \"a\" and \"b\"",
        ),
        (
            json!({"kind":"stringType"}),
            overflow,
            "signed 64-bit integer addition overflow",
        ),
    ] {
        for result in evaluate(binary("add", left, right)) {
            assert_eq!(result.expect_err("child failure").to_string(), expected);
        }
    }
    for op in [
        "subtract",
        "multiply",
        "divide",
        "modulo",
        "quotient",
        "remainder",
    ] {
        for result in evaluate(binary(op, string("x"), string("y"))) {
            assert!(matches!(result, Err(Error::Arithmetic)), "{op}");
        }
    }
}

#[test]
fn source_and_cli_concatenate_and_fail_without_partial_output() {
    let grammar = export("grammar");
    let source = r#""é\n" + "é🦀""#;
    let expected_tree = json!({"formatVersion":1,"entry":{"kind":"add","left":{"kind":"string","value":"é\n"},"right":{"kind":"string","value":"é🦀"}}});
    for parsed in [
        parser::parse(source, &grammar),
        parser::parse_interpreted(source, &grammar, &export("pratt")),
    ] {
        let parsed = parsed.expect("source tree");
        assert_eq!(parsed, expected_tree);
        for result in [
            execute_json(&parsed.to_string(), &export("evaluation")),
            generated::execute_json(&parsed.to_string()),
        ] {
            assert_eq!(result.expect("source addition"), json!("é\né🦀"));
        }
    }
    let path = std::env::temp_dir().join(format!("cuestacean-sum-{}.cue", std::process::id()));
    for (input, success, expected) in [
        (source, true, Some(json!("é\né🦀"))),
        ("string + \"x\"", false, None),
        ("\"x\" + 1", false, None),
    ] {
        std::fs::write(&path, input).expect("source");
        let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-source"))
            .arg(&path)
            .output()
            .expect("CLI");
        assert_eq!(output.status.success(), success, "{output:?}");
        if let Some(expected) = expected {
            assert_eq!(
                serde_json::from_slice::<Json>(&output.stdout).expect("JSON"),
                expected
            );
            assert!(output.stderr.is_empty());
        } else {
            assert!(output.stdout.is_empty());
            assert!(!output.stderr.is_empty());
        }
    }
    std::fs::remove_file(path).expect("remove source");
}
