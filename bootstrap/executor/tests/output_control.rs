use bootstrap_executor::{Error, execute, execute_json, generated, runtime, unification};
use runtime::Value;
use serde_json::{Value as Json, json};
use std::{path::Path, sync::Arc};

#[path = "../../generated/output-zero.rs"]
mod alternate;
#[path = "../../generated/integer-output.rs"]
mod integer;
#[path = "../../generated/json-output.rs"]
mod scalar;

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("fresh export")
}
fn control(function: &str, result: Json) -> String {
    json!({"formatVersion":1,"function":function,"result":result}).to_string()
}
fn message<T>(result: Result<T, Error>) -> Result<T, String> {
    result.map_err(|error| error.to_string())
}

#[test]
fn fixed_domain_and_bounds_matrix() {
    let integer_control = export("integer-output");
    let json_control = export("json-output");
    let integer_error = "integer output requires an integer value";
    let mut cases = vec![
        (Value::Null, Ok(json!(null)), Err(integer_error)),
        (
            Value::Top,
            Err("non-concrete value"),
            Err("non-concrete value"),
        ),
        (
            Value::Int,
            Err("non-concrete int value"),
            Err("non-concrete int value"),
        ),
        (
            Value::Bool,
            Err("non-concrete bool value"),
            Err("non-concrete bool value"),
        ),
        (
            Value::String,
            Err("non-concrete string value"),
            Err("non-concrete string value"),
        ),
        (Value::Boolean(false), Ok(json!(false)), Err(integer_error)),
        (Value::Boolean(true), Ok(json!(true)), Err(integer_error)),
        (
            Value::Text(Arc::from("\"\\\n\r\t\u{8}\u{c} λ🦀")),
            Ok(json!("\"\\\n\r\t\u{8}\u{c} λ🦀")),
            Err(integer_error),
        ),
        (
            Value::Text(Arc::from("")),
            Ok(json!("")),
            Err(integer_error),
        ),
    ];
    for number in [
        i64::MIN,
        -9_007_199_254_740_993,
        -1,
        0,
        1,
        9_007_199_254_740_993,
        i64::MAX,
    ] {
        cases.push((Value::Integer(number), Ok(json!(number)), Ok(number)));
    }
    for (lower, upper) in [
        (None, None),
        (Some(i64::MIN), None),
        (None, Some(i64::MAX)),
        (Some(i64::MIN), Some(i64::MAX)),
        (Some(7), Some(7)),
        (Some(-2), Some(3)),
        (Some(3), Some(-2)),
    ] {
        cases.push((
            Value::Bounds { lower, upper },
            Err("non-concrete integer bounds"),
            Err("non-concrete integer bounds"),
        ));
    }
    for (value, expected_json, expected_integer) in cases {
        let json_expected = expected_json.map_err(str::to_owned);
        let integer_expected = expected_integer.map_err(str::to_owned);
        for result in [
            runtime::concrete_json(value.clone()),
            scalar::json_output(value.clone()).and_then(runtime::encode_json),
            unification::interpret_unary(&json_control, value.clone())
                .and_then(runtime::encode_json),
        ] {
            assert_eq!(message(result), json_expected, "JSON {value:?}");
        }
        for result in [
            runtime::concrete(value.clone()),
            integer::integer_output(value.clone()).and_then(runtime::encode_integer),
            unification::interpret_unary(&integer_control, value.clone())
                .and_then(runtime::encode_integer),
        ] {
            assert_eq!(message(result), integer_expected, "integer {value:?}");
        }
        if matches!(
            value,
            Value::Null | Value::Integer(_) | Value::Boolean(_) | Value::Text(_)
        ) {
            assert_eq!(message(runtime::encode_json(value.clone())), json_expected);
        } else {
            assert!(matches!(
                runtime::encode_json(value.clone()),
                Err(Error::OutputPolicy)
            ));
        }
        if matches!(value, Value::Integer(_)) {
            assert_eq!(message(runtime::encode_integer(value)), integer_expected);
        } else {
            assert!(matches!(
                runtime::encode_integer(value),
                Err(Error::OutputPolicy)
            ));
        }
    }
}

#[test]
fn admission_preserves_shared_text_and_conversion_releases_it() {
    let text: Arc<str> = Arc::from("shared λ\n\"🦀");
    let weak = Arc::downgrade(&text);
    for result in [
        scalar::json_output(Value::Text(Arc::clone(&text))),
        unification::interpret_unary(&export("json-output"), Value::Text(Arc::clone(&text))),
    ] {
        let Value::Text(admitted) = result.expect("admitted text") else {
            panic!("text changed kind")
        };
        assert_eq!(&*admitted, "shared λ\n\"🦀");
        assert!(Arc::ptr_eq(&text, &admitted));
    }
    assert_eq!(Arc::strong_count(&text), 1);
    let output = runtime::concrete_json(Value::Text(text)).expect("JSON copy");
    assert!(
        weak.upgrade().is_none(),
        "conversion must not retain shared text"
    );
    assert_eq!(output, json!("shared λ\n\"🦀"));
}

#[test]
fn actual_compiled_alternate_changes_admission_not_encoding() {
    let input = Value::Boolean(false);
    assert_eq!(
        runtime::concrete_json(input.clone()).expect("default"),
        json!(false)
    );
    assert_eq!(
        unification::interpret_unary(&export("json-output"), input.clone())
            .and_then(runtime::encode_json)
            .expect("reference default"),
        json!(false)
    );
    assert_eq!(
        alternate::json_output(input.clone()).expect("compiled alternate admission"),
        Value::Integer(0)
    );
    assert_eq!(
        alternate::json_output(input.clone())
            .and_then(runtime::encode_json)
            .expect("compiled alternate JSON"),
        json!(0)
    );
    assert_eq!(
        unification::interpret_unary(&export("tests/valid/output-zero"), input)
            .and_then(runtime::encode_json)
            .expect("reference alternate"),
        json!(0)
    );
}

#[test]
fn output_names_have_real_unary_arity_and_validate_unused_branches() {
    for (wire, rust) in [
        ("integerOutput", "integer_output"),
        ("jsonOutput", "json_output"),
    ] {
        let input = control(wire, json!({"op":"input","input":"left"}));
        let code = unification::generate(&input).expect("unary output");
        assert!(code.contains(&format!("fn {rust}(_left: Value)")));
        assert!(!code.contains("_right"));
        assert_eq!(
            unification::interpret_unary(&input, Value::Boolean(false))
                .expect("name is not semantics"),
            Value::Boolean(false)
        );
        assert!(unification::interpret(&input, Value::Null, Value::Null).is_err());
        for bad in [
            json!({"op":"input","input":"right"}),
            json!({"op":"integerSum"}),
            json!({"op":"integerPrimitive","operation":"product"}),
            json!({"op":"bounds","lower":{"op":"input","input":"right","endpoint":"lower"},"upper":{"op":"empty"}}),
            json!({"op":"fail","reason":"types"}),
            json!({"op":"fail","reason":"integers"}),
            json!({"op":"fail","reason":"bounds"}),
            json!({"op":"fail","reason":"booleans"}),
            json!({"op":"fail","reason":"strings"}),
            json!({"op":"unaryFail","reason":"incompleteInt"}),
        ] {
            let invalid = control(
                wire,
                json!({"op":"if","condition":{"op":"kindIs","input":"left","kind":"null"},"then":{"op":"input","input":"left"},"else":bad}),
            );
            assert!(unification::generate(&invalid).is_err());
            assert!(unification::interpret_unary(&invalid, Value::Null).is_err());
        }
        for condition in [
            json!({"op":"same"}),
            json!({"op":"kindIs","input":"right","kind":"null"}),
            json!({"op":"hasEndpoint","value":{"op":"input","input":"left","endpoint":"lower"}}),
        ] {
            let invalid = control(
                wire,
                json!({"op":"if","condition":condition,"then":{"op":"integer","value":0},"else":{"op":"integer","value":1}}),
            );
            assert!(unification::generate(&invalid).is_err());
            assert!(unification::interpret_unary(&invalid, Value::Null).is_err());
        }
    }
    assert!(unification::interpret_unary(&export("unify"), Value::Null).is_err());
}

#[test]
fn six_failures_are_input_independent_in_both_arities() {
    for (reason, variant, expected) in [
        ("incompleteInt", "Incomplete", "non-concrete int value"),
        ("incompleteValue", "IncompleteValue", "non-concrete value"),
        (
            "incompleteBool",
            "IncompleteBool",
            "non-concrete bool value",
        ),
        (
            "incompleteString",
            "IncompleteString",
            "non-concrete string value",
        ),
        (
            "incompleteBounds",
            "IncompleteBounds",
            "non-concrete integer bounds",
        ),
        (
            "integerOutput",
            "IntegerOutput",
            "integer output requires an integer value",
        ),
    ] {
        for function in ["jsonOutput", "integerOutput"] {
            let input = control(function, json!({"op":"fail","reason":reason}));
            let code = unification::generate(&input).expect("unary failure lowering");
            assert!(!code.contains("_right"));
            assert!(code.contains(&format!("Err(Error::{variant})")));
            assert_eq!(
                unification::interpret_unary(&input, Value::Null)
                    .expect_err("unary failure")
                    .to_string(),
                expected
            );
        }
        let binary = control("unify", json!({"op":"fail","reason":reason}));
        assert!(unification::generate(&binary).is_ok());
        assert_eq!(
            unification::interpret(&binary, Value::Integer(1), Value::Boolean(false))
                .expect_err("binary failure")
                .to_string(),
            expected
        );
    }
    for invalid in [
        r#"{"formatVersion":1,"function":"jsonOutput","result":{"op":"input","input":"left","input":"left"}}"#,
        r#"{"formatVersion":1,"function":"jsonOutput","result":{"op":"input","input":"left","unknown":0}}"#,
        r#"{"formatVersion":1,"function":"jsonOutput","function":"jsonOutput","result":{"op":"integer","value":0}}"#,
        r#"{"formatVersion":1,"function":"jsonOutput","unknown":0,"result":{"op":"integer","value":0}}"#,
        r#"{"formatVersion":1,"function":"jsonOutput","result":{"op":"if","condition":{"op":"kindIs","input":"left","kind":"null"},"then":{"op":"integer","value":0},"else":{"op":"fail","reason":"incompleteInt","reason":"incompleteInt"}}}"#,
        r#"{"formatVersion":1,"function":"jsonOutput","result":{"op":"if","condition":{"op":"kindIs","input":"left","kind":"null"},"then":{"op":"integer","value":0},"else":{"op":"fail","reason":"incompleteInt","unknown":0}}}"#,
    ] {
        assert!(unification::generate(invalid).is_err());
        assert!(unification::interpret_unary(invalid, Value::Null).is_err());
    }
}

#[test]
fn evaluator_child_errors_precede_output_and_abstract_unification_survives() {
    let evaluation = export("evaluation");
    for (entry, expected) in [
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
            json!({"kind":"unify","left":{"kind":"intType"},"right":{"kind":"true"}}),
            "conflicting value types",
        ),
        (
            json!({"kind":"unify","left":{"kind":"top"},"right":{"kind":"intType"}}),
            "non-concrete int value",
        ),
        (
            json!({"kind":"unify","left":{"kind":"top"},"right":{"kind":"boolType"}}),
            "non-concrete bool value",
        ),
        (
            json!({"kind":"unify","left":{"kind":"top"},"right":{"kind":"stringType"}}),
            "non-concrete string value",
        ),
    ] {
        let input = json!({"formatVersion":1,"entry":entry}).to_string();
        for result in [
            execute_json(&input, &evaluation),
            generated::execute_json(&input),
        ] {
            assert_eq!(result.expect_err("JSON diagnostic").to_string(), expected);
        }
        for result in [execute(&input, &evaluation), generated::execute(&input)] {
            assert_eq!(
                result.expect_err("integer diagnostic").to_string(),
                expected
            );
        }
    }
}
