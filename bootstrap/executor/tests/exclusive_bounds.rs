use std::{path::Path, process::Command, sync::Arc};

use bootstrap_executor::{Error, execute_json, generated, parser, runtime, scanner, unification};
use runtime::Value;
use serde_json::{Value as Json, json};

#[path = "../../generated/bound-inclusive-only.rs"]
mod inclusive_only;
#[path = "../../generated/primary.rs"]
mod primary;
#[path = "../../generated/exclusive-reversed.rs"]
mod reversed;
#[path = "../../generated/pratt-right.rs"]
mod right_expression;

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("export")
}
fn integer(value: i64) -> Json {
    json!({"kind":"integer","value":value})
}
fn unary(kind: &str, operand: Json) -> Json {
    json!({"kind":kind,"operand":operand})
}
fn binary(kind: &str, left: Json, right: Json) -> Json {
    json!({"kind":kind,"left":left,"right":right})
}
fn evaluations(entry: Json) -> [Result<Json, Error>; 2] {
    let input = json!({"formatVersion":1,"entry":entry}).to_string();
    [
        execute_json(&input, &export("evaluation")),
        generated::execute_json(&input),
    ]
}
fn parses(source: &str, grammar: &str) -> [Result<Json, parser::Error>; 2] {
    [
        parser::parse(source, grammar),
        parser::parse_interpreted(source, grammar, &export("pratt")),
    ]
}
fn control(function: &str, result: Json) -> String {
    json!({"formatVersion":1,"function":function,"result":result}).to_string()
}

#[test]
fn independent_threshold_extrema_and_domain_matrix() {
    for (lower, name) in [
        (true, "lower-exclusive-bound"),
        (false, "upper-exclusive-bound"),
    ] {
        let policy = export(name);
        let facade = if lower {
            runtime::lower_exclusive_bound
        } else {
            runtime::upper_exclusive_bound
        };
        for n in [i64::MIN, i64::MIN + 1, -1, 0, 7, i64::MAX - 1, i64::MAX] {
            let expected = match (lower, n) {
                (true, i64::MAX) | (false, i64::MIN) => None,
                (true, n) => Some(Value::Bounds {
                    lower: Some(n + 1),
                    upper: None,
                }),
                (false, n) => Some(Value::Bounds {
                    lower: None,
                    upper: Some(n - 1),
                }),
            };
            for actual in [
                facade(Value::Integer(n)),
                unification::interpret_unary(&policy, Value::Integer(n)),
            ] {
                if let Some(expected) = &expected {
                    assert_eq!(&actual.expect("nonempty bound"), expected);
                } else {
                    let error = actual.expect_err("empty i64 domain");
                    assert!(matches!(error, Error::EmptyBound));
                    assert_eq!(
                        error.to_string(),
                        "exclusive bound is empty in the signed 64-bit integer domain"
                    );
                }
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
            Value::List(runtime::ClosedList::new(vec![]).expect("list")),
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
            for actual in [
                facade(bad.clone()),
                unification::interpret_unary(&policy, bad),
            ] {
                let error = actual.expect_err("concrete integer only");
                assert!(matches!(error, Error::BoundThreshold));
                assert_eq!(
                    error.to_string(),
                    "bound threshold requires a concrete integer"
                );
            }
        }
    }
}

#[test]
fn literal_and_computed_json_bounds_remain_incomplete_and_refine_candidates() {
    for (kind, computed, threshold, candidate) in [
        ("lowerExclusiveBound", "lowerExclusiveBoundExpression", 3, 4),
        ("upperExclusiveBound", "upperExclusiveBoundExpression", 3, 2),
        (
            "lowerExclusiveBound",
            "lowerExclusiveBoundExpression",
            i64::MAX - 1,
            i64::MAX,
        ),
        (
            "upperExclusiveBound",
            "upperExclusiveBoundExpression",
            i64::MIN + 1,
            i64::MIN,
        ),
        (
            "lowerExclusiveBound",
            "lowerExclusiveBoundExpression",
            i64::MIN,
            i64::MAX,
        ),
        (
            "upperExclusiveBound",
            "upperExclusiveBoundExpression",
            i64::MAX,
            i64::MIN,
        ),
    ] {
        for bound in [
            json!({"kind":kind,"value":threshold}),
            unary(computed, integer(threshold)),
        ] {
            for result in evaluations(bound.clone()) {
                assert!(matches!(result, Err(Error::IncompleteBounds)));
            }
            for result in evaluations(binary("unify", bound.clone(), integer(candidate))) {
                assert_eq!(result.expect("candidate"), json!(candidate));
            }
            for result in evaluations(binary("unify", bound, integer(threshold))) {
                assert_eq!(
                    result.expect_err("excluded threshold").to_string(),
                    "conflicting integer bounds"
                );
            }
        }
    }
    for result in evaluations(binary(
        "unify",
        json!({"kind":"lowerExclusiveBound","value":2}),
        json!({"kind":"upperExclusiveBound","value":4}),
    )) {
        assert!(
            matches!(result, Err(Error::IncompleteBounds)),
            "singleton is not concrete"
        );
    }
    for (kind, n) in [
        ("lowerExclusiveBound", i64::MAX),
        ("upperExclusiveBound", i64::MIN),
    ] {
        for entry in [
            json!({"kind":kind,"value":n}),
            unary(&format!("{kind}Expression"), integer(n)),
        ] {
            for result in evaluations(entry) {
                assert!(matches!(result, Err(Error::EmptyBound)));
            }
        }
        for value in [Json::Null, json!(true), json!("7"), integer(7)] {
            for result in evaluations(json!({"kind":kind,"value":value})) {
                assert!(matches!(result, Err(Error::Integer { field }) if field == "value"));
            }
        }
    }
}

#[test]
fn children_precede_admission_which_precedes_enclosing_rhs() {
    for kind in [
        "lowerExclusiveBoundExpression",
        "upperExclusiveBoundExpression",
    ] {
        for operand in [
            json!({"kind":"null"}),
            json!({"kind":"top"}),
            json!({"kind":"true"}),
            json!({"kind":"false"}),
            json!({"kind":"string","value":"3"}),
            json!({"kind":"intType"}),
            json!({"kind":"boolType"}),
            json!({"kind":"stringType"}),
            json!({"kind":"list","value":[]}),
            json!({"kind":"lowerBound","value":3}),
        ] {
            let entry = binary(
                "unify",
                unary(kind, operand),
                binary("divide", integer(1), integer(0)),
            );
            for result in evaluations(entry) {
                assert!(matches!(result, Err(Error::BoundThreshold)));
            }
        }
        for (child, message) in [
            (
                binary("divide", integer(1), integer(0)),
                "integer division by zero",
            ),
            (
                binary("add", integer(i64::MAX), integer(1)),
                "signed 64-bit integer addition overflow",
            ),
            (
                unary("negate", integer(i64::MIN)),
                "signed 64-bit integer negation overflow",
            ),
            (
                binary("unify", integer(1), integer(2)),
                "conflicting integer values 1 and 2",
            ),
        ] {
            for result in evaluations(binary(
                "unify",
                unary(kind, child),
                binary("divide", integer(2), integer(0)),
            )) {
                assert_eq!(result.expect_err("child first").to_string(), message);
            }
        }
        let n = if kind.starts_with("lower") {
            i64::MAX
        } else {
            i64::MIN
        };
        for result in evaluations(binary(
            "unify",
            unary(kind, integer(n)),
            binary("divide", integer(1), integer(0)),
        )) {
            assert!(matches!(result, Err(Error::EmptyBound)));
        }
    }
}

#[test]
fn compiled_alternate_changes_direction_not_just_function_name() {
    let expected = Value::Bounds {
        lower: None,
        upper: Some(6),
    };
    assert_eq!(
        reversed::lower_exclusive_bound(Value::Integer(7)).expect("alternate"),
        expected
    );
    assert_eq!(
        unification::interpret_unary(&export("tests/valid/exclusive-reversed"), Value::Integer(7))
            .expect("reference alternate"),
        expected
    );
    assert_eq!(
        runtime::lower_exclusive_bound(Value::Integer(7)).expect("production"),
        Value::Bounds {
            lower: Some(8),
            upper: None
        }
    );
}

#[test]
fn unit_offset_kernel_preserves_infinity_and_checks_overflow() {
    for (value, delta, expected) in [
        (None, 1, None),
        (None, -1, None),
        (Some(7), 1, Some(8)),
        (Some(7), -1, Some(6)),
        (Some(i64::MIN), 1, Some(i64::MIN + 1)),
        (Some(i64::MAX), -1, Some(i64::MAX - 1)),
    ] {
        assert_eq!(
            unification::offset_endpoint(value, delta).expect("offset"),
            expected
        );
    }
    for (value, delta) in [(i64::MAX, 1), (i64::MIN, -1)] {
        let error = unification::offset_endpoint(Some(value), delta).expect_err("overflow");
        assert!(matches!(error, Error::EndpointOffsetOverflow));
        assert_eq!(
            error.to_string(),
            "signed 64-bit integer endpoint offset overflow"
        );
    }
}

#[test]
fn unary_arity_and_recursive_unused_endpoint_validation() {
    let binary =
        json!({"formatVersion":1,"result":{"op":"fail","reason":"emptyBound"}}).to_string();
    assert!(matches!(
        unification::interpret(&binary, Value::Null, Value::Top),
        Err(Error::EmptyBound)
    ));
    assert!(unification::interpret_unary(&binary, Value::Null).is_err());
    for function in ["lowerExclusiveBound", "upperExclusiveBound"] {
        let policy = control(function, json!({"op":"fail","reason":"emptyBound"}));
        assert!(matches!(
            unification::interpret_unary(&policy, Value::Top),
            Err(Error::EmptyBound)
        ));
        assert!(unification::interpret(&policy, Value::Top, Value::Null).is_err());
        assert!(
            !unification::generate(&policy)
                .expect("unary source")
                .contains("_right")
        );
        for bad in [
            json!({"op":"offset","value":{"op":"empty"},"delta":0}),
            json!({"op":"offset","value":{"op":"empty"},"delta":2}),
            json!({"op":"offset","value":{"op":"empty"},"delta":-2}),
            json!({"op":"offset","value":{"op":"empty"},"delta":1,"extra":true}),
            json!({"op":"offset","value":{"op":"input","input":"right","endpoint":"lower"},"delta":1}),
            json!({"op":"offset","value":{"op":"local","endpoint":"upper"},"delta":-1}),
        ] {
            // Validation cannot depend on branch selection or endpoint side.
            for result in [
                json!({"op":"bounds","lower":bad,"upper":{"op":"empty"}}),
                json!({"op":"bounds","lower":{"op":"empty"},"upper":bad}),
            ] {
                let policy = control(
                    function,
                    json!({"op":"if","condition":{"op":"kindIs","input":"left","kind":"integer"},"then":{"op":"integer","value":0},"else":result}),
                );
                assert!(unification::generate(&policy).is_err(), "{policy}");
                assert!(
                    unification::interpret_unary(&policy, Value::Integer(7)).is_err(),
                    "{policy}"
                );
            }
        }
        let valid = control(
            function,
            json!({"op":"if","condition":{"op":"compare","left":{"op":"offset","value":{"op":"input","input":"left","endpoint":"lower"},"delta":1},"right":{"op":"integer","value":8},"relation":"equal"},"then":{"op":"integer","value":1},"else":{"op":"integer","value":0}}),
        );
        assert_eq!(
            unification::interpret_unary(&valid, Value::Integer(7)).expect("left-only compare"),
            Value::Integer(1)
        );
        assert!(
            !unification::generate(&valid)
                .expect("left-only compare lowering")
                .contains("_right")
        );
        for input in ["right", "local"] {
            let endpoint = if input == "local" {
                json!({"op":"local","endpoint":"lower"})
            } else {
                json!({"op":"input","input":input,"endpoint":"lower"})
            };
            let unused = json!({"op":"if","condition":{"op":"compare","left":{"op":"offset","value":endpoint,"delta":1},"right":{"op":"integer","value":0},"relation":"less"},"then":{"op":"integer","value":0},"else":{"op":"integer","value":1}});
            let policy = control(
                function,
                json!({"op":"if","condition":{"op":"kindIs","input":"left","kind":"integer"},"then":{"op":"integer","value":7},"else":unused}),
            );
            assert!(unification::generate(&policy).is_err());
            assert!(unification::interpret_unary(&policy, Value::Integer(7)).is_err());
        }
    }
}

#[test]
fn independent_source_trees_and_longest_prefix_scanner() {
    let grammar = export("grammar");
    for (source, bound, candidate) in [
        (">3&4", json!({"kind":"lowerExclusiveBound","value":3}), 4),
        (
            "<0x10&15",
            json!({"kind":"upperExclusiveBound","value":16}),
            15,
        ),
        (
            "> // threshold\n (1+2)&4",
            unary(
                "lowerExclusiveBoundExpression",
                binary("add", integer(1), integer(2)),
            ),
            4,
        ),
        (
            "<(0b100)&3",
            unary("upperExclusiveBoundExpression", integer(4)),
            3,
        ),
        (">=3&3", json!({"kind":"lowerBound","value":3}), 3),
        ("<=3&3", json!({"kind":"upperBound","value":3}), 3),
    ] {
        let entry = binary("unify", bound, integer(candidate));
        for actual in parses(source, &grammar) {
            assert_eq!(
                actual.expect(source),
                json!({"formatVersion":1,"entry":entry})
            );
        }
        for result in evaluations(entry) {
            assert_eq!(result.expect(source), json!(candidate));
        }
    }
    for (source, end, flag) in [
        (">3", 1, true),
        ("<3", 1, true),
        (">=3", 2, true),
        ("<=3", 2, true),
        ("> =3", 1, true),
        ("=3", 0, false),
    ] {
        let expected = scanner::Scan { offset: end, flag };
        assert_eq!(
            scanner::bound_prefix(source, 0).expect("compiled"),
            expected
        );
        assert_eq!(
            scanner::interpret(&export("bound-prefix"), source, 0).expect("reference"),
            expected
        );
    }
    for source in [">3", "<3"] {
        let expected = scanner::Scan {
            offset: 0,
            flag: false,
        };
        assert_eq!(
            inclusive_only::scan(source, 0).expect("saved CUE scanner"),
            expected
        );
        assert_eq!(
            scanner::interpret(&export("tests/valid/bound-inclusive-only"), source, 0)
                .expect("saved reference"),
            expected
        );
    }
}

#[test]
fn metadata_and_selected_expression_driver_propagate_into_exclusive_groups() {
    let original: Json = serde_json::from_str(&export("grammar")).expect("grammar");
    for token in [">", "<"] {
        let index = original["bounds"]
            .as_array()
            .expect("bounds")
            .iter()
            .position(|b| b["token"] == token)
            .expect("exclusive entry");
        let mut grammar = original.clone();
        grammar["bounds"][index]["expression"] =
            json!({"kind":"customBound","operandField":"child"});
        for actual in parses(&format!("{token}(3)"), &grammar.to_string()) {
            assert_eq!(
                actual.expect("custom")["entry"],
                json!({"kind":"customBound","child":integer(3)})
            );
        }
        grammar["bounds"][index]
            .as_object_mut()
            .expect("entry")
            .remove("expression");
        for actual in parses(&format!("{token}(3)"), &grammar.to_string()) {
            assert!(actual.is_err());
        }
        for actual in parses(&format!("{token}3"), &grammar.to_string()) {
            assert_eq!(actual.expect("literal")["entry"]["value"], 3);
        }
    }
    let grammar = original.to_string();
    for source in [">(1+2+3)", "(>(1+2+3))", "7&>(1+2+3)"] {
        let threshold = unary(
            "lowerExclusiveBoundExpression",
            binary("add", integer(1), binary("add", integer(2), integer(3))),
        );
        let entry = if source.starts_with('7') {
            binary("unify", integer(7), threshold)
        } else {
            threshold
        };
        let expected = json!({"formatVersion":1,"entry":entry});
        assert_eq!(
            parser::parse_with_drivers(
                source,
                &grammar,
                right_expression::execute,
                primary::execute
            )
            .expect(source),
            expected
        );
        assert_eq!(
            parser::parse_interpreted_with_primary(
                source,
                &grammar,
                &export("tests/valid/pratt-right"),
                &export("primary")
            )
            .expect(source),
            expected
        );
    }
}

#[test]
fn cli_semantic_failures_are_atomic() {
    let path =
        std::env::temp_dir().join(format!("cuestacean-exclusive-{}.cue", std::process::id()));
    for (source, message) in [
        (
            ">9223372036854775807",
            "exclusive bound is empty in the signed 64-bit integer domain",
        ),
        (
            "<-9223372036854775808",
            "exclusive bound is empty in the signed 64-bit integer domain",
        ),
        (">(true)", "bound threshold requires a concrete integer"),
        ("<(1 div 0)", "integer division by zero"),
        (">3&3", "conflicting integer bounds"),
        (">2&<4", "non-concrete integer bounds"),
    ] {
        std::fs::write(&path, source).expect("source");
        let output = Command::new(env!("CARGO_BIN_EXE_bootstrap-source"))
            .arg(&path)
            .output()
            .expect("CLI");
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty(), "{source}: {output:?}");
        assert!(
            String::from_utf8(output.stderr)
                .expect("UTF-8")
                .contains(message),
            "{source}"
        );
    }
    std::fs::remove_file(path).expect("cleanup");
}
