use std::{path::Path, sync::Arc};

use bootstrap_executor::{Error, execute_json, generated, runtime, unification};
use runtime::Value;
use serde_json::{Value as Json, json};

#[path = "../../generated/divide-as-quotient.rs"]
mod alternate;
#[path = "../../generated/integer-literals.rs"]
mod integer_literals;

type Facade = fn(Value, Value) -> Result<Value, Error>;
type Kernel = fn(&Value, &Value) -> Result<Value, Error>;
const OPERATIONS: [(&str, Facade, Kernel); 4] = [
    ("divide", runtime::divide, runtime::integer_divide),
    ("modulo", runtime::modulo, runtime::integer_modulo),
    ("quotient", runtime::quotient, runtime::integer_quotient),
    ("remainder", runtime::remainder, runtime::integer_remainder),
];

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("export")
}

fn program(function: &str, result: Json) -> String {
    json!({"formatVersion":1,"function":function,"result":result}).to_string()
}

fn literal(value: i64) -> Json {
    json!({"op":"integer","value":value})
}

#[test]
fn fixed_signed_and_extreme_oracles() {
    // Each row independently specifies div, mod, quo, rem; no runtime-derived oracle.
    for (left, right, expected) in [
        (7, 3, [2, 1, 2, 1]),
        (-7, 3, [-3, 2, -2, -1]),
        (7, -3, [-2, 1, -2, 1]),
        (-7, -3, [3, 2, 2, -1]),
        (0, i64::MIN, [0, 0, 0, 0]),
        (i64::MIN, 1, [i64::MIN, 0, i64::MIN, 0]),
        (i64::MAX, -1, [-i64::MAX, 0, -i64::MAX, 0]),
        (i64::MIN, i64::MIN, [1, 0, 1, 0]),
        (-1, i64::MIN, [1, i64::MAX, 0, -1]),
        (i64::MIN, i64::MAX, [-2, i64::MAX - 1, -1, -1]),
        (i64::MAX, i64::MIN, [0, i64::MAX, 0, i64::MAX]),
    ] {
        for ((name, facade, kernel), expected) in OPERATIONS.into_iter().zip(expected) {
            let (left, right) = (Value::Integer(left), Value::Integer(right));
            for result in [
                facade(left.clone(), right.clone()),
                kernel(&left, &right),
                unification::interpret(&export(&format!("{name}-control")), left, right),
            ] {
                assert_eq!(result.expect(name), Value::Integer(expected), "{name}");
            }
        }
    }
}

#[test]
fn cue_unit_divisor_branch_is_not_a_native_remainder_exception() {
    for (name, facade, kernel) in OPERATIONS {
        let control = export(&format!("{name}-control"));
        let (left, right) = (Value::Integer(i64::MIN), Value::Integer(-1));
        assert!(matches!(
            kernel(&left, &right),
            Err(Error::DivisionOverflow)
        ));
        for result in [
            facade(left.clone(), right.clone()),
            unification::interpret(&control, left.clone(), right.clone()),
        ] {
            if matches!(name, "modulo" | "remainder") {
                assert_eq!(result.expect("CUE literal zero"), Value::Integer(0));
            } else {
                assert!(matches!(result, Err(Error::DivisionOverflow)));
            }
        }
        // The DSL primitive itself must retain the defensive overflow too.
        let raw = program(name, json!({"op":"integerPrimitive","operation":name}));
        assert!(matches!(
            unification::interpret(&raw, left, right),
            Err(Error::DivisionOverflow)
        ));
    }
}

#[test]
fn domains_precede_zero_checks_in_policy_and_defensive_kernels() {
    for (name, facade, kernel) in OPERATIONS {
        let control = export(&format!("{name}-control"));
        for bad in [
            Value::Null,
            Value::Top,
            Value::Boolean(false),
            Value::Bool,
            Value::Text(Arc::from("0")),
            Value::String,
            Value::Int,
            Value::Bounds {
                lower: Some(0),
                upper: Some(0),
            },
        ] {
            for (left, right) in [
                (bad.clone(), Value::Integer(0)),
                (Value::Integer(0), bad.clone()),
                (bad.clone(), bad),
            ] {
                for result in [
                    facade(left.clone(), right.clone()),
                    kernel(&left, &right),
                    unification::interpret(&control, left.clone(), right.clone()),
                ] {
                    assert!(
                        matches!(result, Err(Error::Arithmetic)),
                        "{name}: {left:?}, {right:?}"
                    );
                }
            }
        }
        for left in [i64::MIN, 0, i64::MAX] {
            let (left, right) = (Value::Integer(left), Value::Integer(0));
            for result in [
                facade(left.clone(), right.clone()),
                kernel(&left, &right),
                unification::interpret(&control, left, right),
            ] {
                assert!(matches!(result, Err(Error::DivisionByZero)), "{name}");
            }
        }
    }
}

#[test]
fn compiled_alternate_changes_the_kernel_of_unchanged_control() {
    let control = export("divide-control");
    let (left, right) = (Value::Integer(-7), Value::Integer(3));
    assert_eq!(
        alternate::divide(left.clone(), right.clone()).expect("actual compiled alternate"),
        Value::Integer(-2)
    );
    assert_eq!(
        runtime::divide(left.clone(), right.clone()).expect("default compiled"),
        Value::Integer(-3)
    );
    assert_eq!(
        unification::interpret(&control, left, right).expect("reference"),
        Value::Integer(-3)
    );
    let source = unification::generate_with_policy(
        &control,
        &export("tests/valid/division-kernel-alternate"),
    )
    .expect("alternate compiler");
    assert!(source.contains("integer_quotient"));
    assert_ne!(
        source,
        unification::generate(&control).expect("default compiler")
    );
    for (name, _, _) in OPERATIONS {
        let source = unification::generate(&export(&format!("{name}-control"))).expect("entry");
        assert!(source.contains(&format!("fn {name}(_left: Value, _right: Value)")));
    }
}

#[test]
fn compiled_extrema_fixture_preserves_literal_roles() {
    for result in [
        integer_literals::unify(Value::Null, Value::Top),
        unification::interpret(
            &export("tests/valid/integer-literals"),
            Value::Null,
            Value::Top,
        ),
    ] {
        assert_eq!(
            result.expect("extrema literal roles"),
            Value::Integer(i64::MIN)
        );
    }
}

#[test]
fn original_integer_wire_has_distinct_value_and_endpoint_roles() {
    for value in [i64::MIN, 0, i64::MAX] {
        let input = program("divide", literal(value));
        assert_eq!(
            unification::interpret(&input, Value::Null, Value::Top).expect("value literal"),
            Value::Integer(value)
        );
        assert!(
            unification::generate(&input)
                .expect("value code")
                .contains("Value::Integer")
        );
        let unary = program("negate", literal(value));
        assert_eq!(
            unification::interpret_unary(&unary, Value::Null).expect("unary literal"),
            Value::Integer(value)
        );
        assert!(
            !unification::generate(&unary)
                .expect("unary code")
                .contains("_right")
        );
        let condition = json!({"op":"compare","relation":"equal",
            "left":{"op":"input","input":"right","endpoint":"lower"},"right":literal(value)});
        let endpoint = program(
            "divide",
            json!({"op":"if","condition":condition,"then":literal(17),"else":literal(29)}),
        );
        unification::generate(&endpoint).expect("endpoint literal code");
        for (right, expected) in [(value, 17), (if value == 0 { 1 } else { 0 }, 29)] {
            assert_eq!(
                unification::interpret(&endpoint, Value::Top, Value::Integer(right))
                    .expect("finite compare"),
                Value::Integer(expected)
            );
        }
        assert!(unification::interpret(&endpoint, Value::Top, Value::Int).is_err());
    }
}

fn reject(function: &str, result: Json) {
    let input = program(function, result);
    assert!(unification::generate(&input).is_err(), "{input}");
    if function == "negate" {
        assert!(
            unification::interpret_unary(&input, Value::Integer(1)).is_err(),
            "{input}"
        );
    } else {
        assert!(
            unification::interpret(&input, Value::Integer(1), Value::Integer(1)).is_err(),
            "{input}"
        );
    }
}

#[test]
fn literal_validation_checks_unused_branches_aliases_and_i64_limits() {
    for bad in [
        json!({"op":"integerValue","value":0}),
        json!({"op":"integerEndpoint","value":0}),
        json!({"op":"integer"}),
        json!({"op":"integer","value":true}),
        json!({"op":"integer","value":"0"}),
        json!({"op":"integer","value":1.5}),
        json!({"op":"integer","value":0,"extra":0}),
        json!({"op":"integer","value":9223372036854775808_u64}),
        serde_json::from_str(r#"{"op":"integer","value":-9223372036854775809}"#)
            .expect("JSON number"),
    ] {
        // Check both roles, including an unselected value branch.
        reject(
            "divide",
            json!({"op":"if","condition":{"op":"kindIs","input":"left","kind":"integer"},"then":literal(1),"else":bad}),
        );
        let endpoint_branch = json!({"op":"bounds","lower":literal(0),"upper":bad});
        reject(
            "divide",
            json!({"op":"if","condition":{"op":"kindIs","input":"left","kind":"integer"},"then":literal(1),"else":endpoint_branch}),
        );
    }
    reject("divide", json!({"op":"integer","value":0,"input":"left"}));
    reject(
        "divide",
        json!({"op":"bounds","lower":{"op":"boolean","value":true},"upper":literal(0)}),
    );
}

#[test]
fn new_operations_remain_binary_even_when_unused() {
    for (name, _, _) in OPERATIONS {
        let input = program(name, literal(0));
        assert!(unification::interpret_unary(&input, Value::Integer(1)).is_err());
        let bad = json!({"op":"integerPrimitive","operation":name});
        reject(
            "negate",
            json!({"op":"if","condition":{"op":"kindIs","input":"left","kind":"integer"},"then":literal(0),"else":bad}),
        );
    }
    reject("negate", json!({"op":"fail","reason":"divisionByZero"}));
    reject(
        "negate",
        json!({"op":"if","condition":{"op":"compare","relation":"equal","left":literal(0),"right":{"op":"input","input":"right","endpoint":"lower"}},"then":literal(1),"else":literal(2)}),
    );
    let failure = program("divide", json!({"op":"fail","reason":"divisionByZero"}));
    assert!(matches!(
        unification::interpret(&failure, Value::Null, Value::Top),
        Err(Error::DivisionByZero)
    ));
    unification::generate(&failure).expect("binary zero failure");
}

#[test]
fn both_json_evaluators_propagate_eager_children_before_policy() {
    let integer = |value| json!({"kind":"integer","value":value});
    let binary = |kind, left, right| json!({"kind":kind,"left":left,"right":right});
    let zero = binary("divide", integer(1), integer(0));
    let overflow = binary("quotient", integer(i64::MIN), integer(-1));
    let conflict = binary("unify", integer(1), integer(2));
    let rules = export("evaluation");
    for (name, _, _) in OPERATIONS {
        for (left, right, expected) in [
            (zero.clone(), overflow.clone(), "integer division by zero"),
            (
                overflow.clone(),
                zero.clone(),
                "signed 64-bit integer division overflow",
            ),
            (
                conflict.clone(),
                zero.clone(),
                "conflicting integer values 1 and 2",
            ),
            (
                json!({"kind":"intType"}),
                overflow.clone(),
                "signed 64-bit integer division overflow",
            ),
            (
                integer(1),
                conflict.clone(),
                "conflicting integer values 1 and 2",
            ),
        ] {
            let tree = json!({"formatVersion":1,"entry":binary(name, left, right)}).to_string();
            for result in [execute_json(&tree, &rules), generated::execute_json(&tree)] {
                assert_eq!(
                    result.expect_err("child error").to_string(),
                    expected,
                    "{name}"
                );
            }
        }
    }
}
