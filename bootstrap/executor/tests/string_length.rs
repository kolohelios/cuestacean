use bootstrap_executor::{Error, execute, execute_json, generated, runtime, unification};
use runtime::Value;
use serde_json::{Value as Json, json};
use std::{path::Path, sync::Arc};

#[path = "../../generated/length-zero.rs"]
mod alternate;

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("fresh export")
}
fn control(function: &str, result: Json) -> String {
    json!({"formatVersion":1,"function":function,"result":result}).to_string()
}
fn tree(operand: Json) -> String {
    json!({"formatVersion":1,"entry":{"kind":"stringLength","operand":operand}}).to_string()
}
fn all_apis(input: &str, rules: &str) -> [Result<Json, Error>; 4] {
    [
        execute_json(input, rules),
        generated::execute_json(input),
        execute(input, rules).map(|value| json!(value)),
        generated::execute(input).map(|value| json!(value)),
    ]
}
fn rejects(input: &str) {
    assert!(unification::generate(input).is_err(), "{input}");
    assert!(
        unification::interpret_unary(input, Value::Text(Arc::from("abc"))).is_err(),
        "{input}"
    );
}

#[test]
fn fixed_utf8_byte_counts_through_kernels_control_and_all_evaluator_apis() {
    let policy = export("string-length-control");
    let rules = export("evaluation");
    for (text, expected) in [
        ("", 0),
        ("abc", 3),
        ("ASCII", 5),
        ("é", 2),
        ("🙂", 4),
        ("e\u{301}", 3),
        ("\0", 1),
        ("\n", 1),
        ("\r", 1),
        ("\t", 1),
        ("\u{8}", 1),
        ("\u{c}", 1),
        ("é🙂", 6),
        ("\"\\", 2),
    ] {
        let value = Value::Text(Arc::from(text));
        for result in [
            runtime::text_length(&value),
            runtime::string_length(value.clone()),
            unification::interpret_unary(&policy, value),
        ] {
            assert_eq!(
                result.expect("byte count"),
                Value::Integer(expected),
                "{text:?}"
            );
        }
        let input = tree(json!({"kind":"string","value":text}));
        for result in all_apis(&input, &rules) {
            let output = result.expect("exact integer output");
            assert_eq!(output, json!(expected), "{text:?}");
            assert_eq!(output.to_string(), expected.to_string());
        }
    }
    // JSON escapes are decoded before counting, with no Unicode normalization.
    let escaped = r#"{"formatVersion":1,"entry":{"kind":"stringLength","operand":{"kind":"string","value":"\u00e9\ud83d\ude42"}}}"#;
    for result in all_apis(escaped, &rules) {
        assert_eq!(result.expect("decoded UTF-8 bytes"), json!(6));
    }
}

#[test]
fn every_non_text_representation_rejects_including_abstract_and_singleton_bounds() {
    let policy = export("string-length-control");
    let rules = export("evaluation");
    for (value, ast) in [
        (Value::Null, json!({"kind":"null"})),
        (Value::Top, json!({"kind":"top"})),
        (Value::Integer(0), json!({"kind":"integer","value":0})),
        (Value::Int, json!({"kind":"intType"})),
        (Value::Boolean(false), json!({"kind":"false"})),
        (Value::Boolean(true), json!({"kind":"true"})),
        (Value::Bool, json!({"kind":"boolType"})),
        (Value::String, json!({"kind":"stringType"})),
    ] {
        for result in [
            runtime::text_length(&value),
            runtime::string_length(value.clone()),
            unification::interpret_unary(&policy, value.clone()),
        ] {
            assert!(
                matches!(result, Err(Error::Length)),
                "{value:?}: {result:?}"
            );
        }
        for result in all_apis(&tree(ast), &rules) {
            let error = result.expect_err("length domain before output admission");
            assert!(matches!(error, Error::Length));
            assert_eq!(error.to_string(), "length requires a concrete string");
        }
    }
    for (lower, upper) in [
        (None, None),
        (Some(i64::MIN), None),
        (None, Some(i64::MAX)),
        (Some(7), Some(7)),
        (Some(3), Some(-2)),
    ] {
        let value = Value::Bounds { lower, upper };
        for result in [
            runtime::text_length(&value),
            runtime::string_length(value.clone()),
            unification::interpret_unary(&policy, value),
        ] {
            assert!(matches!(result, Err(Error::Length)));
        }
    }
    for ast in [
        json!({"kind":"lowerBound","value":7}),
        json!({"kind":"upperBound","value":7}),
        json!({"kind":"unify","left":{"kind":"lowerBound","value":7},"right":{"kind":"upperBound","value":7}}),
    ] {
        for result in all_apis(&tree(ast), &rules) {
            assert!(matches!(result, Err(Error::Length)));
        }
    }
}

#[test]
fn child_errors_precede_length_domain_and_output_checks() {
    let rules = export("evaluation");
    for (operand, expected) in [
        (
            json!({"kind":"divide","left":{"kind":"integer","value":1},"right":{"kind":"integer","value":0}}),
            "integer division by zero",
        ),
        (
            json!({"kind":"add","left":{"kind":"integer","value":i64::MAX},"right":{"kind":"integer","value":1}}),
            "signed 64-bit integer addition overflow",
        ),
        (
            json!({"kind":"unify","left":{"kind":"true"},"right":{"kind":"false"}}),
            "conflicting boolean values true and false",
        ),
        (
            json!({"kind":"string","value":false}),
            "field \"value\" must be a string",
        ),
        (
            json!({"kind":"unknownTestKind"}),
            "no rule for node kind \"unknownTestKind\"",
        ),
    ] {
        for result in all_apis(&tree(operand), &rules) {
            assert_eq!(result.expect_err("eager child error").to_string(), expected);
        }
    }
    let input = tree(json!({"kind":"stringLength","operand":{"kind":"string","value":"abc"}}));
    for result in all_apis(&input, &rules) {
        assert!(
            matches!(result, Err(Error::Length)),
            "nested integer is not text"
        );
    }
}

#[test]
fn borrowed_kernel_and_consuming_controls_do_not_retain_shared_text() {
    let text: Arc<str> = Arc::from("é🙂");
    let value = Value::Text(Arc::clone(&text));
    let count = Arc::strong_count(&text);
    assert_eq!(
        runtime::text_length(&value).expect("borrowed"),
        Value::Integer(6)
    );
    assert_eq!(Arc::strong_count(&text), count);
    assert_eq!(
        runtime::string_length(value.clone()).expect("compiled"),
        Value::Integer(6)
    );
    assert_eq!(Arc::strong_count(&text), count);
    assert_eq!(
        unification::interpret_unary(&export("string-length-control"), value.clone())
            .expect("reference"),
        Value::Integer(6)
    );
    assert_eq!(Arc::strong_count(&text), count);
    drop(value);
    assert_eq!(Arc::strong_count(&text), 1);
}

#[test]
fn actual_compiled_alternate_proves_function_name_is_arity_not_behavior() {
    let policy = export("tests/valid/length-zero");
    for value in [Value::Boolean(false), Value::Text(Arc::from("abc"))] {
        assert_eq!(
            alternate::string_length(value.clone()).expect("compiled alternate"),
            Value::Integer(0)
        );
        assert_eq!(
            unification::interpret_unary(&policy, value).expect("reference alternate"),
            Value::Integer(0)
        );
    }
    assert!(matches!(
        runtime::string_length(Value::Boolean(false)),
        Err(Error::Length)
    ));
    assert_eq!(
        runtime::string_length(Value::Text(Arc::from("abc"))).expect("default"),
        Value::Integer(3)
    );
    let identity = control("stringLength", json!({"op":"input","input":"left"}));
    assert_eq!(
        unification::interpret_unary(&identity, Value::Boolean(false)).expect("name is not policy"),
        Value::Boolean(false)
    );
    let code = unification::generate(&export("string-length-control")).expect("unary lowering");
    assert!(code.contains("fn string_length(_left: Value)"));
    assert!(!code.contains("_right"));
    assert!(unification::interpret(&policy, Value::Null, Value::Null).is_err());
    assert!(unification::interpret_unary(&export("unify"), Value::Null).is_err());
}

#[test]
fn text_length_uses_only_left_and_length_failure_is_input_independent() {
    let binary = control("unify", json!({"op":"textLength"}));
    assert!(unification::generate(&binary).is_ok());
    for right in [
        Value::Null,
        Value::Top,
        Value::Boolean(false),
        Value::Integer(99),
    ] {
        assert_eq!(
            unification::interpret(&binary, Value::Text(Arc::from("é🙂")), right)
                .expect("left-only primitive"),
            Value::Integer(6)
        );
    }
    assert!(matches!(
        unification::interpret(&binary, Value::Null, Value::Text(Arc::from("abc"))),
        Err(Error::Length)
    ));
    for function in ["stringLength", "unify"] {
        let input = control(function, json!({"op":"fail","reason":"length"}));
        let code = unification::generate(&input).expect("failure lowering");
        let result = if function == "stringLength" {
            assert!(code.contains("Error::Length"));
            assert!(!code.contains("_right"));
            unification::interpret_unary(&input, Value::Text(Arc::from("valid")))
        } else {
            assert!(code.contains("Failure::Length"));
            unification::interpret(&input, Value::Null, Value::Top)
        };
        assert!(matches!(result, Err(Error::Length)));
    }
}

#[test]
fn unary_subset_validates_dead_branches_and_rejects_wire_aliases() {
    for bad in [
        json!({"op":"input","input":"right"}),
        json!({"op":"integerSum"}),
        json!({"op":"textConcat"}),
        json!({"op":"integerPrimitive","operation":"product"}),
        json!({"op":"fail","reason":"types"}),
        json!({"op":"textLength","input":"right"}),
        json!({"op":"textLength","unknown":0}),
        json!({"op":"unaryFail","reason":"length"}),
        json!({"op":"integerValue","value":0}),
        json!({"op":"inputValue","input":"left"}),
    ] {
        rejects(&control("stringLength", bad.clone()));
        rejects(&control(
            "stringLength",
            json!({"op":"if","condition":{"op":"kindIs","input":"left","kind":"text"},"then":{"op":"integer","value":0},"else":bad}),
        ));
    }
    for condition in [
        json!({"op":"same"}),
        json!({"op":"kindIs","input":"right","kind":"text"}),
        json!({"op":"booleanIs","input":"right","value":false}),
        json!({"op":"textCompare","relation":"equal"}),
    ] {
        rejects(&control(
            "stringLength",
            json!({"op":"if","condition":condition,"then":{"op":"integer","value":0},"else":{"op":"textLength"}}),
        ));
    }
    for invalid in [
        r#"{"formatVersion":1,"function":"stringLength","result":{"op":"textLength","op":"textLength"}}"#,
        r#"{"formatVersion":1,"function":"stringLength","function":"stringLength","result":{"op":"textLength"}}"#,
        r#"{"formatVersion":1,"function":"stringLength","unknown":0,"result":{"op":"textLength"}}"#,
        r#"{"formatVersion":1,"function":"stringLength","result":{"op":"if","condition":{"op":"kindIs","input":"left","kind":"text"},"then":{"op":"integer","value":0},"else":{"op":"fail","reason":"length","reason":"length"}}}"#,
    ] {
        rejects(invalid);
    }
}
