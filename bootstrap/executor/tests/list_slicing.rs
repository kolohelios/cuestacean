use std::{path::Path, sync::Arc};

use bootstrap_executor::{Error, execute_json, generated, runtime, unification};
use runtime::{ClosedList, ClosedRecord, Value as V};
use serde_json::{Value as J, json};

#[path = "../../generated/list-slice-identity.rs"]
mod identity;
#[path = "../../generated/evaluation-left-twice.rs"]
mod left_twice;

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("fresh export")
}
fn integer(value: i64) -> J {
    json!({"kind":"integer","value":value})
}
fn binary(kind: &str, left: J, right: J) -> J {
    json!({"kind":kind,"left":left,"right":right})
}
fn slice(operand: J, lower: J, upper: J) -> J {
    json!({"kind":"listSlice","operand":operand,"lower":lower,"upper":upper})
}
fn list(values: Vec<V>) -> V {
    V::List(ClosedList::new(values).expect("certified list"))
}
fn evaluations(entry: J, selected: bool) -> [Result<J, Error>; 2] {
    let input = json!({"formatVersion":1,"entry":entry}).to_string();
    if selected {
        [
            execute_json(&input, &export("tests/valid/evaluation-left-twice")),
            left_twice::execute_json(&input),
        ]
    } else {
        [
            execute_json(&input, &export("evaluation")),
            generated::execute_json(&input),
        ]
    }
}
fn outcomes(value: &V, lower: i64, upper: i64) -> [Result<V, Error>; 3] {
    controls(value, &V::Integer(lower), &V::Integer(upper))
}
fn controls(value: &V, lower: &V, upper: &V) -> [Result<V, Error>; 3] {
    [
        runtime::list_slice_elements(value, lower, upper),
        runtime::list_slice(value.clone(), lower.clone(), upper.clone()),
        unification::interpret_ternary(
            &export("list-slice"),
            value.clone(),
            lower.clone(),
            upper.clone(),
        ),
    ]
}
fn assert_error(results: impl IntoIterator<Item = Result<J, Error>>, expected: Error) {
    for result in results {
        let error = result.expect_err("independent error oracle");
        assert_eq!(
            std::mem::discriminant(&error),
            std::mem::discriminant(&expected)
        );
        assert_eq!(error.to_string(), expected.to_string());
    }
}
fn negate(mut child: J, count: usize) -> J {
    for _ in 0..count {
        child = json!({"kind":"negate","operand":child});
    }
    child
}
fn spine(levels: usize) -> J {
    let mut child = integer(0);
    for _ in 0..levels {
        child = binary("add", child, integer(0));
    }
    child
}

#[test]
fn independent_ast_values_include_empty_end_exact_integers_and_mixed_children() {
    let operand = json!({"kind":"listExpression","elements":[
        integer(i64::MIN),
        {"kind":"recordExpression","fields":[{"name":"é🙂","value":integer(i64::MAX)}]},
        {"kind":"listExpression","elements":[{"kind":"string","value":"é🙂"}]},
        {"kind":"null"}
    ]});
    for (lower, upper, expected) in [
        (0, 0, json!([])),
        (4, 4, json!([])),
        (0, 4, json!([i64::MIN, {"é🙂":i64::MAX}, ["é🙂"], null])),
        (1, 3, json!([{"é🙂":i64::MAX}, ["é🙂"]])),
    ] {
        // Computed endpoints are ordinary children, not literal-only metadata.
        let entry = slice(
            operand.clone(),
            binary("add", integer(lower), integer(0)),
            binary("subtract", integer(upper), integer(0)),
        );
        for result in evaluations(entry, false) {
            assert_eq!(result.expect("independent slice output"), expected);
        }
    }
    let empty = slice(json!({"kind":"list","value":[]}), integer(0), integer(0));
    let legacy = json!({"formatVersion":1,"entry":{"kind":"length","operand":empty}});
    assert_eq!(
        left_twice::execute(&legacy.to_string()).expect("slice length"),
        0
    );
    for result in evaluations(
        slice(json!({"kind":"list","value":[]}), integer(0), integer(0)),
        false,
    ) {
        assert_eq!(result.expect("empty input"), json!([]));
    }
}

#[test]
fn kernel_generated_and_reference_cover_half_open_ranges_and_extrema() {
    let values = vec![
        V::Null,
        V::Boolean(false),
        V::Integer(i64::MIN),
        V::Integer(i64::MAX),
        V::Text(Arc::from("é🙂")),
    ];
    let items = list(values.clone());
    for lower in 0..=5 {
        for upper in lower..=5 {
            let expected = list(values[lower..upper].to_vec());
            for result in outcomes(&items, lower as i64, upper as i64) {
                assert_eq!(result.expect("half-open slice"), expected);
            }
        }
    }
    for (value, lower, upper) in [
        (list(vec![]), 0, 1),
        (items.clone(), -1, 0),
        (items.clone(), 3, 2),
        (items.clone(), 0, 6),
        (items.clone(), i64::MIN, i64::MAX),
        (items.clone(), 0, i64::MAX),
        (items, i64::MAX, i64::MAX),
    ] {
        for result in outcomes(&value, lower, upper) {
            assert!(matches!(result, Err(Error::ListSliceRange)), "{result:?}");
        }
    }
    for result in outcomes(&list(vec![V::Null; 64]), 64, 64) {
        assert_eq!(result.expect("width boundary"), list(vec![]));
    }
    assert_eq!(
        Error::ListSliceOperands.to_string(),
        "list slicing requires a concrete closed list and integer endpoints"
    );
    assert_eq!(Error::ListSliceRange.to_string(), "list slice out of range");
}

#[test]
fn each_operand_domain_is_checked_before_range_without_coercion() {
    let domains = [
        V::Top,
        V::Null,
        V::Bool,
        V::Boolean(false),
        V::Int,
        V::Integer(0),
        V::String,
        V::Text(Arc::from("0")),
        V::Bounds {
            lower: Some(0),
            upper: Some(0),
        },
        list(vec![]),
        V::Record(ClosedRecord::new(vec![]).expect("record")),
    ];
    for value in &domains {
        for slot in 0..3 {
            let mut inputs = [list(vec![]), V::Integer(0), V::Integer(0)];
            inputs[slot] = value.clone();
            let valid = if slot == 0 {
                matches!(value, V::List(_))
            } else {
                matches!(value, V::Integer(_))
            };
            for result in controls(&inputs[0], &inputs[1], &inputs[2]) {
                if valid {
                    assert_eq!(result.expect("valid domains"), list(vec![]));
                } else {
                    assert!(
                        matches!(result, Err(Error::ListSliceOperands)),
                        "slot {slot}: {result:?}"
                    );
                }
            }
        }
    }
    for result in controls(&V::Null, &V::Integer(-1), &V::Integer(i64::MAX)) {
        assert!(matches!(result, Err(Error::ListSliceOperands)));
    }
}

#[test]
fn all_three_children_are_eager_and_errors_are_ordered_before_dispatch() {
    let zero = binary("divide", integer(1), integer(0));
    let overflow = binary("add", integer(i64::MAX), integer(1));
    let unknown = json!({"kind":"unknownTestKind"});
    for (operand, lower, upper, expected) in [
        (
            zero.clone(),
            overflow.clone(),
            unknown.clone(),
            Error::DivisionByZero,
        ),
        (
            integer(0),
            zero.clone(),
            overflow.clone(),
            Error::DivisionByZero,
        ),
        (integer(0), integer(-1), zero.clone(), Error::DivisionByZero),
        (
            json!({"kind":"list","value":[]}),
            integer(-1),
            overflow,
            Error::Overflow,
        ),
        (integer(0), integer(0), integer(0), Error::ListSliceOperands),
        (
            json!({"kind":"list","value":[]}),
            integer(0),
            integer(1),
            Error::ListSliceRange,
        ),
    ] {
        assert_error(evaluations(slice(operand, lower, upper), false), expected);
    }
    for field in ["operand", "lower", "upper"] {
        let mut entry = slice(integer(0), integer(0), integer(0));
        entry.as_object_mut().expect("node").remove(field);
        assert_error(
            evaluations(entry, false),
            Error::MissingField {
                field: field.into(),
            },
        );
    }
    let mut entry = slice(zero, integer(0), integer(0));
    entry.as_object_mut().expect("node").remove("upper");
    assert_error(evaluations(entry, false), Error::DivisionByZero);
}

#[test]
fn each_child_shares_depth_and_selected_evaluator_visit_budgets() {
    for slot in 0..3 {
        for wrappers in [62, 63] {
            let mut inputs = [json!({"kind":"list","value":[]}), integer(0), integer(0)];
            inputs[slot] = negate(integer(0), wrappers);
            let [operand, lower, upper] = inputs;
            for selected in [false, true] {
                let results = evaluations(
                    slice(operand.clone(), lower.clone(), upper.clone()),
                    selected,
                );
                if wrappers == 63 {
                    assert_error(results, Error::EvaluationDepth);
                } else if slot == 0 {
                    assert_error(results, Error::ListSliceOperands);
                } else {
                    for result in results {
                        assert_eq!(result.expect("depth restored between children"), json!([]));
                    }
                }
            }
        }
        let mut inputs = [integer(0), integer(0), integer(0)];
        inputs[slot] = negate(spine(11), 1); // 4096 selected visits in this child alone.
        let [operand, lower, upper] = inputs;
        let entry = slice(operand, lower, upper);
        assert_error(evaluations(entry.clone(), true), Error::EvaluationVisits);
        assert_error(evaluations(entry, false), Error::ListSliceOperands);
    }
    // Root + two 2047-visit children leave exactly one entry for upper.
    let entry = slice(spine(10), spine(10), integer(0));
    assert_error(evaluations(entry, true), Error::ListSliceOperands);
    assert_error(
        evaluations(slice(spine(10), spine(10), negate(integer(0), 1)), true),
        Error::EvaluationVisits,
    );
}

#[test]
fn required_field_lookup_precedes_attempted_child_entry_at_visit_limit() {
    for (field, mut entry) in [
        ("lower", slice(spine(11), integer(0), integer(0))),
        ("upper", slice(spine(10), negate(spine(10), 1), integer(0))),
    ] {
        entry.as_object_mut().expect("node").remove(field);
        assert_error(
            evaluations(entry.clone(), true),
            Error::MissingField {
                field: field.into(),
            },
        );
        entry[field] = json!({});
        assert_error(evaluations(entry, true), Error::EvaluationVisits);
    }
}

#[test]
fn same_function_alternate_preserves_certified_input_instead_of_slicing() {
    let policy = export("tests/valid/list-slice-identity");
    let items = list(vec![V::Integer(7), V::Integer(8)]);
    for result in [
        identity::list_slice(items.clone(), V::Integer(1), V::Integer(2)),
        unification::interpret_ternary(&policy, items.clone(), V::Integer(1), V::Integer(2)),
    ] {
        assert_eq!(
            result.expect("alternate identity"),
            list(vec![V::Integer(7), V::Integer(8)])
        );
    }
    for result in outcomes(&items, 1, 2) {
        assert_eq!(
            result.expect("default policy differs"),
            list(vec![V::Integer(8)])
        );
    }
}

#[test]
fn mixed_children_share_storage_outlive_inputs_and_keep_private_shape_certificates() {
    let text: Arc<str> = Arc::from("owned é🙂");
    let child = V::Record(
        ClosedRecord::new(vec![("x".into(), list(vec![V::Text(text.clone())]))])
            .expect("mixed child"),
    );
    let parent = list(vec![V::Null, child.clone()]);
    let selected =
        runtime::list_slice_elements(&parent, &V::Integer(1), &V::Integer(2)).expect("owned slice");
    let V::List(items) = &selected else {
        panic!("list result")
    };
    let (V::Record(a), V::Record(b)) = (&items.as_slice()[0], &child) else {
        panic!("record child")
    };
    assert!(std::ptr::eq(a.as_map(), b.as_map()));
    drop(parent);
    drop(child);
    drop(text);
    assert_eq!(
        runtime::encode_json(selected).expect("surviving slice"),
        json!([{"x":["owned é🙂"]}])
    );

    let shared = V::Record(
        ClosedRecord::new((0..64).map(|i| (format!("f{i}"), V::Null)).collect()).expect("65 nodes"),
    );
    let boundary = list(vec![shared; 63]); // Exactly 4096 occurrence nodes.
    for result in outcomes(&boundary, 0, 63) {
        let retained = result.expect("full slice independently recertified");
        assert!(matches!(
            ClosedList::new(vec![retained]),
            Err(Error::ListNodes)
        ));
    }
    let mut deep = V::Null;
    for level in 0..30 {
        deep = if level % 2 == 0 {
            list(vec![deep])
        } else {
            V::Record(ClosedRecord::new(vec![("x".into(), deep)]).expect("mixed depth"))
        };
    }
    let boundary = list(vec![deep]); // Root-inclusive value depth 32.
    for result in outcomes(&boundary, 0, 1) {
        assert!(matches!(
            ClosedList::new(vec![result.expect("depth-32 slice")]),
            Err(Error::ListDepth)
        ));
    }
}
