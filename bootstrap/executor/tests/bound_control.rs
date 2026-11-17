use std::{path::Path, sync::Arc};

use bootstrap_executor::{Error, execute_json, generated, runtime, unification};
use runtime::Value;
use serde_json::{Value as Json, json};

#[path = "../../generated/unary-bound-endpoints.rs"]
mod endpoints;
#[path = "../../generated/bound-reversed.rs"]
mod reversed;

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("export")
}

fn program(function: &str, result: Json) -> String {
    json!({"formatVersion":1,"function":function,"result":result}).to_string()
}

fn bounds(lower: Json, upper: Json) -> Json {
    json!({"op":"bounds","lower":lower,"upper":upper})
}

fn condition() -> Json {
    json!({"op":"kindIs","input":"left","kind":"integer"})
}

fn choose(then: Json, otherwise: Json) -> Json {
    json!({"op":"choose","condition":condition(),"then":then,"else":otherwise})
}

fn facade(function: &str, value: Value) -> Result<Value, Error> {
    match function {
        "lowerBound" => runtime::lower_bound(value),
        "upperBound" => runtime::upper_bound(value),
        _ => panic!("unknown bound function"),
    }
}

#[test]
fn fixed_thresholds_and_all_non_integer_domains() {
    for (function, name) in [("lowerBound", "lower-bound"), ("upperBound", "upper-bound")] {
        let control = export(name);
        for n in [i64::MIN, i64::MAX, -1, 0, 7] {
            let expected = if function == "lowerBound" {
                Value::Bounds {
                    lower: Some(n),
                    upper: None,
                }
            } else {
                Value::Bounds {
                    lower: None,
                    upper: Some(n),
                }
            };
            for result in [
                facade(function, Value::Integer(n)),
                unification::interpret_unary(&control, Value::Integer(n)),
            ] {
                assert_eq!(result.expect("bound"), expected, "{function}({n})");
            }
        }
        for bad in [
            Value::Null,
            Value::Top,
            Value::Boolean(false),
            Value::Boolean(true),
            Value::Bool,
            Value::Text(Arc::from("7")),
            Value::String,
            Value::Int,
            Value::Bounds {
                lower: None,
                upper: None,
            },
            Value::Bounds {
                lower: Some(7),
                upper: None,
            },
            Value::Bounds {
                lower: None,
                upper: Some(7),
            },
            Value::Bounds {
                lower: Some(7),
                upper: Some(7),
            },
        ] {
            for result in [
                facade(function, bad.clone()),
                unification::interpret_unary(&control, bad.clone()),
            ] {
                let error = result.expect_err("threshold domain");
                assert!(
                    matches!(error, Error::BoundThreshold),
                    "{function}({bad:?}): {error}"
                );
                assert_eq!(
                    error.to_string(),
                    "bound threshold requires a concrete integer"
                );
            }
        }
    }
}

#[test]
fn actual_compiled_reversed_policy_changes_endpoint_placement() {
    assert_eq!(
        reversed::lower_bound(Value::Integer(7)).expect("compiled alternate"),
        Value::Bounds {
            lower: None,
            upper: Some(7)
        }
    );
    let expected = Value::Bounds {
        lower: Some(7),
        upper: None,
    };
    assert_eq!(
        runtime::lower_bound(Value::Integer(7)).expect("production"),
        expected
    );
    assert_eq!(
        unification::interpret_unary(&export("lower-bound"), Value::Integer(7)).expect("reference"),
        expected
    );
}

#[test]
fn unary_endpoints_execute_without_rhs_or_locals() {
    let control = export("tests/valid/unary-bound-endpoints");
    let source = unification::generate(&control).expect("endpoint lowering");
    assert!(!source.contains("_right"));
    for (input, expected) in [
        (
            Value::Integer(7),
            Value::Bounds {
                lower: Some(-7),
                upper: Some(7),
            },
        ),
        (
            Value::Null,
            Value::Bounds {
                lower: None,
                upper: None,
            },
        ),
        (
            Value::Bounds {
                lower: Some(1),
                upper: Some(9),
            },
            Value::Bounds {
                lower: None,
                upper: Some(9),
            },
        ),
    ] {
        assert_eq!(
            unification::interpret_unary(&control, input.clone()).expect("reference endpoints"),
            expected
        );
        assert_eq!(
            endpoints::lower_bound(input).expect("compiled endpoints"),
            expected
        );
    }
    // Literal and empty endpoints have no input-domain admission policy.
    let literal = program(
        "lowerBound",
        bounds(
            json!({"op":"integer","value":i64::MIN}),
            json!({"op":"empty"}),
        ),
    );
    assert_eq!(
        unification::interpret_unary(&literal, Value::Top).expect("literal endpoint"),
        Value::Bounds {
            lower: Some(i64::MIN),
            upper: None
        }
    );
    assert!(
        !unification::generate(&literal)
            .expect("literal lowering")
            .contains("_right")
    );
}

#[test]
fn unary_endpoint_subset_checks_both_endpoints_and_unused_branches() {
    let empty = json!({"op":"empty"});
    let left = json!({"op":"input","input":"left","endpoint":"lower"});
    for bad in [
        json!({"op":"input","input":"right","endpoint":"lower"}),
        json!({"op":"input","input":"right","endpoint":"upper"}),
        json!({"op":"local","endpoint":"lower"}),
        json!({"op":"local","endpoint":"upper"}),
        json!({"op":"choose","condition":{"op":"hasEndpoint","value":left},"then":empty,"else":empty}),
        json!({"op":"choose","condition":{"op":"compare","left":{"op":"integer","value":0},"right":{"op":"integer","value":1},"relation":"less"},"then":empty,"else":empty}),
        json!({"op":"choose","condition":{"op":"same"},"then":empty,"else":empty}),
        json!({"op":"choose","condition":{"op":"booleanIs","input":"right","value":true},"then":empty,"else":empty}),
    ] {
        for endpoint in [
            bad.clone(),
            choose(empty.clone(), bad.clone()),
            choose(bad, empty.clone()),
        ] {
            for result in [
                bounds(endpoint.clone(), empty.clone()),
                bounds(empty.clone(), endpoint),
            ] {
                // Also bury the invalid endpoint in an unselected value branch.
                for result in [
                    result.clone(),
                    json!({"op":"if","condition":condition(),"then":{"op":"integer","value":0},"else":result}),
                ] {
                    for function in ["lowerBound", "upperBound"] {
                        let input = program(function, result.clone());
                        assert!(unification::generate(&input).is_err(), "{input}");
                        assert!(
                            unification::interpret_unary(&input, Value::Integer(7)).is_err(),
                            "{input}"
                        );
                    }
                }
            }
        }
    }
    for bad in [
        json!({"op":"letBounds","lower":empty,"upper":empty,"result":{"op":"integer","value":0}}),
        json!({"op":"integerSum"}),
        json!({"op":"integerPrimitive","operation":"difference"}),
        json!({"op":"if","condition":{"op":"same"},"then":{"op":"integer","value":0},"else":{"op":"integer","value":1}}),
    ] {
        let input = program("lowerBound", bad);
        assert!(unification::generate(&input).is_err(), "{input}");
        assert!(
            unification::interpret_unary(&input, Value::Integer(7)).is_err(),
            "{input}"
        );
    }
}

#[test]
fn bound_entry_arity_and_input_independent_failure() {
    for function in ["lowerBound", "upperBound"] {
        let input = program(function, json!({"op":"fail","reason":"boundThreshold"}));
        assert!(matches!(
            unification::interpret_unary(&input, Value::Integer(7)),
            Err(Error::BoundThreshold)
        ));
        assert!(unification::interpret(&input, Value::Integer(7), Value::Null).is_err());
        let source = unification::generate(&input).expect("unary failure");
        assert!(!source.contains("_right"));
        assert!(source.contains("Error::BoundThreshold"));
    }
    let binary =
        json!({"formatVersion":1,"result":{"op":"fail","reason":"boundThreshold"}}).to_string();
    assert!(matches!(
        unification::interpret(&binary, Value::Null, Value::Top),
        Err(Error::BoundThreshold)
    ));
    assert!(unification::interpret_unary(&binary, Value::Null).is_err());
    let source = unification::generate(&binary).expect("default binary arity");
    assert!(source.contains("fn unify(_left: Value, _right: Value)"));
}

#[test]
fn evaluators_keep_bounds_incomplete_and_unify_literal_candidates() {
    let rules = export("evaluation");
    for kind in ["lowerBound", "upperBound"] {
        for n in [i64::MIN, i64::MAX, -1, 0, 7] {
            let bound = json!({"kind":kind,"value":n});
            let input = json!({"formatVersion":1,"entry":bound}).to_string();
            for result in [
                execute_json(&input, &rules),
                generated::execute_json(&input),
            ] {
                assert!(matches!(result, Err(Error::IncompleteBounds)));
            }
            let candidate = json!({"formatVersion":1,"entry":{"kind":"unify","left":bound,"right":{"kind":"integer","value":n}}}).to_string();
            for result in [
                execute_json(&candidate, &rules),
                generated::execute_json(&candidate),
            ] {
                assert_eq!(result.expect("candidate"), json!(n));
            }
        }
    }
}

#[test]
fn reference_custom_threshold_children_fail_before_bound_admission() {
    // Default rules read scalar thresholds. Only this custom reference rule
    // evaluates a threshold child; this is not evidence about generated rules.
    let mut rules: Json = serde_json::from_str(&export("evaluation")).expect("rules");
    for kind in ["lowerBound", "upperBound"] {
        rules["rules"][kind]["result"]["value"] = json!({"op":"evaluate","field":"operand"});
    }
    for kind in ["lowerBound", "upperBound"] {
        for (child, message) in [
            (
                json!({"kind":"negate","operand":{"kind":"integer","value":i64::MIN}}),
                "signed 64-bit integer negation overflow",
            ),
            (
                json!({"kind":"unify","left":{"kind":"integer","value":1},"right":{"kind":"integer","value":2}}),
                "conflicting integer values 1 and 2",
            ),
            (
                json!({"kind":"intType"}),
                "bound threshold requires a concrete integer",
            ),
        ] {
            let input =
                json!({"formatVersion":1,"entry":{"kind":kind,"operand":child}}).to_string();
            assert_eq!(
                execute_json(&input, &rules.to_string())
                    .expect_err("threshold child")
                    .to_string(),
                message
            );
        }
    }
}

#[test]
fn default_threshold_fields_remain_scalar_in_both_json_evaluators() {
    let rules = export("evaluation");
    for kind in ["lowerBound", "upperBound"] {
        for bad in [
            Json::Null,
            json!(true),
            json!("7"),
            json!({"kind":"integer","value":7}),
        ] {
            let input = json!({"formatVersion":1,"entry":{"kind":kind,"value":bad}}).to_string();
            for result in [
                execute_json(&input, &rules),
                generated::execute_json(&input),
            ] {
                assert!(matches!(result, Err(Error::Integer { field }) if field == "value"));
            }
        }
    }
}

#[cfg(feature = "source")]
#[test]
fn source_inclusive_bounds_preserve_exact_json() {
    let grammar = export("grammar");
    let rules = export("evaluation");
    for (source, expected) in [
        (">=7 & 7", "7"),
        ("<=-1 & -1", "-1"),
        (
            ">=-9223372036854775808 & -9223372036854775808",
            "-9223372036854775808",
        ),
        (
            "<=9223372036854775807 & 9223372036854775807",
            "9223372036854775807",
        ),
    ] {
        let input = bootstrap_executor::parser::parse(source, &grammar)
            .expect("source")
            .to_string();
        for result in [
            execute_json(&input, &rules),
            generated::execute_json(&input),
        ] {
            assert_eq!(result.expect("source JSON").to_string(), expected);
        }
    }
}
