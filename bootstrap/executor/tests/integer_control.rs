use std::{path::Path, process::Command, sync::Arc};

use bootstrap_executor::{Error, execute_json, generated, runtime, unification};
use runtime::Value;
use serde_json::{Value as Json, json};

#[path = "../../generated/difference-as-product.rs"]
mod alternate;

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("export")
}

fn binary_results(name: &str, left: i64, right: i64) -> [Result<Value, Error>; 2] {
    let (left, right) = (Value::Integer(left), Value::Integer(right));
    let production = match name {
        "difference-control" => runtime::difference(left.clone(), right.clone()),
        "product-control" => runtime::product(left.clone(), right.clone()),
        _ => panic!("unknown control"),
    };
    [
        production,
        unification::interpret(&export(name), left, right),
    ]
}

#[test]
fn fixed_integer_results_and_overflows() {
    let control = export("negate-control");
    for (input, expected) in [(0, 0), (7, -7), (-7, 7), (i64::MAX, -i64::MAX)] {
        for result in [
            runtime::negate(Value::Integer(input)),
            unification::interpret_unary(&control, Value::Integer(input)),
            runtime::integer_negate(&Value::Integer(input)),
        ] {
            assert_eq!(result.expect("negation"), Value::Integer(expected));
        }
    }
    for result in [
        runtime::negate(Value::Integer(i64::MIN)),
        unification::interpret_unary(&control, Value::Integer(i64::MIN)),
        runtime::integer_negate(&Value::Integer(i64::MIN)),
    ] {
        assert!(matches!(result, Err(Error::NegationOverflow)));
    }
    for (name, left, right, expected) in [
        ("difference-control", 7, 6, 1),
        ("difference-control", i64::MIN, i64::MIN, 0),
        ("difference-control", -1, i64::MIN, i64::MAX),
        ("difference-control", i64::MAX, 0, i64::MAX),
        ("product-control", 7, 6, 42),
        ("product-control", i64::MIN, 1, i64::MIN),
        ("product-control", i64::MAX, 1, i64::MAX),
        ("product-control", i64::MIN, 0, 0),
    ] {
        for result in binary_results(name, left, right) {
            assert_eq!(result.expect("binary integer"), Value::Integer(expected));
        }
    }
    assert_eq!(
        runtime::integer_difference(&Value::Integer(i64::MIN), &Value::Integer(i64::MIN))
            .expect("borrowed difference"),
        Value::Integer(0)
    );
    assert_eq!(
        runtime::integer_product(&Value::Integer(i64::MIN), &Value::Integer(1))
            .expect("borrowed product"),
        Value::Integer(i64::MIN)
    );
    for (left, right) in [(i64::MIN, 1), (i64::MAX, -1)] {
        for result in binary_results("difference-control", left, right) {
            assert!(matches!(result, Err(Error::DifferenceOverflow)));
        }
        assert!(matches!(
            runtime::integer_difference(&Value::Integer(left), &Value::Integer(right)),
            Err(Error::DifferenceOverflow)
        ));
    }
    for (left, right) in [(i64::MIN, -1), (i64::MAX, 2)] {
        for result in binary_results("product-control", left, right) {
            assert!(matches!(result, Err(Error::ProductOverflow)));
        }
        assert!(matches!(
            runtime::integer_product(&Value::Integer(left), &Value::Integer(right)),
            Err(Error::ProductOverflow)
        ));
    }
}

#[test]
fn non_integer_domains_fail_in_facades_reference_and_kernels() {
    let negate = export("negate-control");
    let difference = export("difference-control");
    let product = export("product-control");
    for bad in [
        Value::Null,
        Value::Top,
        Value::Bool,
        Value::Boolean(true),
        Value::Int,
        Value::String,
        Value::Text(Arc::from("7")),
        Value::Bounds {
            lower: Some(1),
            upper: Some(1),
        },
    ] {
        for result in [
            runtime::negate(bad.clone()),
            unification::interpret_unary(&negate, bad.clone()),
            runtime::integer_negate(&bad),
        ] {
            assert!(matches!(result, Err(Error::Arithmetic)), "{bad:?}");
        }
        for (left, right) in [
            (bad.clone(), Value::Integer(1)),
            (Value::Integer(1), bad.clone()),
            (bad.clone(), bad.clone()),
        ] {
            for result in [
                runtime::difference(left.clone(), right.clone()),
                runtime::product(left.clone(), right.clone()),
                unification::interpret(&difference, left.clone(), right.clone()),
                unification::interpret(&product, left.clone(), right.clone()),
                runtime::integer_difference(&left, &right),
                runtime::integer_product(&left, &right),
            ] {
                assert!(
                    matches!(result, Err(Error::Arithmetic)),
                    "{left:?}, {right:?}"
                );
            }
        }
    }
}

#[test]
fn alternate_compiler_changes_kernel_not_control() {
    let control = export("difference-control");
    let policy = export("tests/valid/integer-kernel-alternate");
    let left = Value::Integer(7);
    let right = Value::Integer(6);
    assert_eq!(
        alternate::difference(left.clone(), right.clone()).expect("compiled alternate"),
        Value::Integer(42)
    );
    assert_eq!(
        runtime::difference(left.clone(), right.clone()).expect("production"),
        Value::Integer(1)
    );
    assert_eq!(
        unification::interpret(&control, left, right).expect("reference"),
        Value::Integer(1)
    );
    let source = unification::generate_with_policy(&control, &policy).expect("alternate compiler");
    assert!(source.contains("integer_product"));
    assert_ne!(
        source,
        unification::generate(&control).expect("production compiler")
    );
    assert!(include_str!("../../generated/difference-as-product.rs").contains("integer_product"));
}

fn program(function: &str, result: Json) -> String {
    json!({"formatVersion":1,"function":function,"result":result}).to_string()
}

#[test]
fn wire_validation_arity_and_entry_names() {
    for (function, symbol) in [
        ("negate", "negate"),
        ("difference", "difference"),
        ("product", "product"),
        ("notEqual", "not_equal"),
        ("lessEqual", "less_equal"),
        ("greaterEqual", "greater_equal"),
        ("logicalAnd", "logical_and"),
        ("logicalOr", "logical_or"),
        ("logicalNot", "logical_not"),
    ] {
        let input = program(function, json!({"op":"input","input":"left"}));
        assert!(
            unification::generate(&input)
                .expect("header")
                .contains(&format!("fn {symbol}(_left: Value"))
        );
    }
    let left = Value::Integer(1);
    assert!(unification::interpret(&export("negate-control"), left.clone(), left.clone()).is_err());
    assert!(unification::interpret_unary(&export("difference-control"), left.clone()).is_err());
    assert!(unification::interpret_unary(&export("product-control"), left.clone()).is_err());
    let fail = program("negate", json!({"op":"fail","reason":"arithmetic"}));
    assert!(matches!(
        unification::interpret_unary(&fail, left.clone()),
        Err(Error::Arithmetic)
    ));
    let source = unification::generate(&fail).expect("unary failure");
    assert!(!source.contains("_right"));
    assert!(source.contains("Error::Arithmetic"));
    for bad in [
        json!({"op":"integerPrimitive","operation":"difference"}),
        json!({"op":"integerPrimitive","operation":"product"}),
        json!({"op":"integerPrimitive","operation":"unknown"}),
        json!({"op":"integerPrimitive","operation":1}),
        json!({"op":"integerPrimitive"}),
        json!({"op":"integerPrimitive","operation":"negate","extra":true}),
    ] {
        let input = program(
            "negate",
            json!({"op":"if","condition":{"op":"kindIs","input":"left","kind":"integer"},"then":{"op":"input","input":"left"},"else":bad}),
        );
        assert!(unification::generate(&input).is_err(), "{input}");
        assert!(
            unification::interpret_unary(&input, left.clone()).is_err(),
            "{input}"
        );
    }
    for bad in [
        json!({"op":"integerPrimitive","operation":"unknown"}),
        json!({"op":"integerPrimitive","operation":false}),
        json!({"op":"integerPrimitive"}),
        json!({"op":"integerPrimitive","operation":"difference","extra":0}),
    ] {
        let input = program("difference", bad);
        assert!(unification::generate(&input).is_err());
        assert!(unification::interpret(&input, left.clone(), left.clone()).is_err());
    }
    let injected = program("difference;evil", json!({"op":"input","input":"left"}));
    assert!(unification::generate(&injected).is_err());
    assert!(unification::interpret(&injected, left.clone(), left).is_err());
}

fn binary(kind: &str, left: Json, right: Json) -> Json {
    json!({"kind":kind,"left":left,"right":right})
}

#[test]
fn evaluator_children_fail_left_first_before_integer_dispatch() {
    let integer = |n| json!({"kind":"integer","value":n});
    let overflow = binary("subtract", integer(i64::MAX), integer(-1));
    let conflict = binary("unify", integer(1), integer(2));
    for kind in ["subtract", "multiply"] {
        for (left, right, expected) in [
            (
                overflow.clone(),
                conflict.clone(),
                "signed 64-bit integer subtraction overflow",
            ),
            (
                conflict.clone(),
                overflow.clone(),
                "conflicting integer values 1 and 2",
            ),
            (
                json!({"kind":"intType"}),
                overflow.clone(),
                "signed 64-bit integer subtraction overflow",
            ),
        ] {
            let tree = json!({"formatVersion":1,"entry":binary(kind, left, right)}).to_string();
            for result in [
                execute_json(&tree, &export("evaluation")),
                generated::execute_json(&tree),
            ] {
                assert_eq!(result.expect_err("child error").to_string(), expected);
            }
        }
    }
    let tree = json!({"formatVersion":1,"entry":{"kind":"negate","operand":overflow}}).to_string();
    for result in [
        execute_json(&tree, &export("evaluation")),
        generated::execute_json(&tree),
    ] {
        assert_eq!(
            result.expect_err("unary child error").to_string(),
            "signed 64-bit integer subtraction overflow"
        );
    }
}

#[test]
fn invalid_primitive_cli_has_no_partial_output() {
    let path = std::env::temp_dir().join(format!("integer-invalid-{}.json", std::process::id()));
    std::fs::write(
        &path,
        program(
            "difference",
            json!({"op":"integerPrimitive","operation":"invalid"}),
        ),
    )
    .expect("invalid input");
    let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-unify-codegen"))
        .arg(&path)
        .output()
        .expect("CLI");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
    std::fs::remove_file(path).expect("remove input");
}
