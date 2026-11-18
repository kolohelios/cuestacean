use std::{path::Path, sync::Arc};

use bootstrap_executor::{Error, execute_json, generated, pratt, runtime, unification};
use runtime::{ClosedList, ClosedRecord, EvaluationContext, SliceDriver, SliceFrame, Value as V};
use serde_json::{Value as J, json};

#[path = "../../generated/evaluation-left-twice.rs"]
mod left_twice;
#[path = "../../generated/slice-control-lower-one.rs"]
mod lower_one;
#[path = "../../generated/slice-control.rs"]
mod production;

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("fresh export")
}
fn integer(value: i64) -> J {
    json!({"kind":"integer","value":value})
}
fn add(left: J, right: J) -> J {
    json!({"kind":"add","left":left,"right":right})
}
fn slice(operand: J, lower: Option<J>, upper: Option<J>) -> J {
    let mut node = json!({"kind":"slice","operand":operand});
    for (name, value) in [("lower", lower), ("upper", upper)] {
        if let Some(value) = value {
            node[name] = value;
        }
    }
    node
}
fn both(entry: J, selected: bool) -> [Result<J, Error>; 2] {
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
fn fails(entry: J, expected: Error) {
    for result in both(entry, false) {
        let actual = result.expect_err("independent error oracle");
        assert_eq!(
            std::mem::discriminant(&actual),
            std::mem::discriminant(&expected)
        );
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
fn forward(frame: &mut SliceFrame<'_>) -> Result<(), Error> {
    pratt::interpret_slice(&export("slice-control"), frame)
}
fn alternate(frame: &mut SliceFrame<'_>) -> Result<(), Error> {
    pratt::interpret_slice(&export("tests/valid/slice-control-lower-one"), frame)
}
fn drivers() -> [(SliceDriver, i64); 4] {
    [
        (production::execute, 0),
        (forward, 0),
        (lower_one::execute, 1),
        (alternate, 1),
    ]
}
fn list(values: Vec<V>) -> V {
    V::List(ClosedList::new(values).expect("certified list"))
}

#[test]
fn complete_asts_have_independent_outputs_for_all_four_presence_patterns() {
    let legacy = json!({"formatVersion":1,"entry":{"kind":"length","operand":slice(json!({"kind":"list","value":[]}),None,None)}});
    assert_eq!(
        left_twice::execute(&legacy.to_string()).expect("legacy optional-slice length"),
        0
    );
    let operand = json!({"kind":"listExpression","elements":[
        integer(i64::MIN),
        {"kind":"recordExpression","fields":[{"name":"é🙂","value":integer(i64::MAX)}]},
        {"kind":"listExpression","elements":[{"kind":"string","value":"é🙂"}]},
        {"kind":"null"}
    ]});
    for (lower, upper, expected) in [
        (
            None,
            None,
            json!([i64::MIN, {"é🙂":i64::MAX}, ["é🙂"], null]),
        ),
        (
            Some(integer(1)),
            None,
            json!([{"é🙂":i64::MAX}, ["é🙂"], null]),
        ),
        (None, Some(integer(2)), json!([i64::MIN, {"é🙂":i64::MAX}])),
        (
            Some(add(integer(1), integer(0))),
            Some(integer(3)),
            json!([{ "é🙂":i64::MAX }, ["é🙂"]]),
        ),
        (Some(integer(4)), None, json!([])),
        (None, Some(integer(0)), json!([])),
    ] {
        for result in both(slice(operand.clone(), lower, upper), false) {
            assert_eq!(result.expect("independent half-open result"), expected);
        }
    }
    for result in both(slice(json!({"kind":"list","value":[]}), None, None), false) {
        assert_eq!(result.expect("empty list"), json!([]));
    }
}

#[test]
fn absence_is_not_json_null_and_existing_domain_range_diagnostics_survive() {
    let empty = json!({"kind":"list","value":[]});
    for field in ["lower", "upper"] {
        let mut node = slice(empty.clone(), None, None);
        node[field] = J::Null;
        fails(node, Error::Kind);
        let mut node = slice(empty.clone(), None, None);
        node[field] = json!({"kind":"null"});
        fails(node, Error::ListSliceOperands);
    }
    fails(
        json!({"kind":"slice"}),
        Error::MissingField {
            field: "operand".into(),
        },
    );
    fails(slice(J::Null, None, None), Error::Kind);
    for operand in [
        integer(0),
        json!({"kind":"null"}),
        json!({"kind":"string","value":"abc"}),
    ] {
        fails(slice(operand, None, None), Error::ListSliceOperands);
    }
    for (lower, upper) in [
        (Some(integer(-1)), None),
        (None, Some(integer(1))),
        (Some(integer(i64::MIN)), Some(integer(i64::MAX))),
        (Some(integer(1)), Some(integer(0))),
    ] {
        fails(slice(empty.clone(), lower, upper), Error::ListSliceRange);
    }
    assert_eq!(
        Error::ListSliceOperands.to_string(),
        "list slicing requires a concrete closed list and integer endpoints"
    );
    assert_eq!(Error::ListSliceRange.to_string(), "list slice out of range");
}

#[test]
fn child_failures_precede_default_count_domain_and_final_range_checks() {
    let zero = json!({"kind":"divide","left":integer(1),"right":integer(0)});
    let overflow = add(integer(i64::MAX), integer(1));
    let empty = json!({"kind":"list","value":[]});
    for (operand, lower, upper, expected) in [
        (
            zero.clone(),
            Some(overflow.clone()),
            Some(J::Null),
            Error::DivisionByZero,
        ),
        (integer(0), Some(zero.clone()), None, Error::DivisionByZero),
        (integer(0), None, Some(zero.clone()), Error::DivisionByZero),
        (
            empty.clone(),
            Some(integer(-1)),
            Some(overflow.clone()),
            Error::Overflow,
        ),
        (empty, Some(zero), Some(overflow), Error::DivisionByZero),
        (integer(0), Some(integer(0)), None, Error::ListSliceOperands),
    ] {
        fails(slice(operand, lower, upper), expected);
    }
}

#[test]
fn actual_alternate_changes_only_missing_lower_and_callbacks_are_once_in_order() {
    for (driver, default) in drivers() {
        for has_lower in [false, true] {
            for has_upper in [false, true] {
                // Renamed fields prove the public names array is honored.
                let mut node = json!({"input":0});
                if has_lower {
                    node["start"] = json!(1);
                }
                if has_upper {
                    node["end"] = json!(2);
                }
                let mut trace = Vec::new();
                let result = runtime::slice_with_driver(
                    &node,
                    ["input", "start", "end"],
                    &mut EvaluationContext::default(),
                    &mut |child, context| {
                        trace.push(child.clone());
                        context.visit(|_| match child.as_i64() {
                            Some(0) => Ok(list(vec![V::Integer(7), V::Integer(8), V::Integer(9)])),
                            Some(value) => Ok(V::Integer(value)),
                            None => Err(Error::Kind),
                        })
                    },
                    runtime::list_slice,
                    driver,
                )
                .expect("compiled/reference control");
                let lower = if has_lower { 1 } else { default };
                let upper = if has_upper { 2 } else { 3 };
                let expected = [7, 8, 9][lower as usize..upper]
                    .iter()
                    .copied()
                    .map(V::Integer)
                    .collect();
                assert_eq!(result, list(expected));
                let mut expected_trace = vec![json!(0)];
                if has_lower {
                    expected_trace.push(json!(1));
                }
                if has_upper {
                    expected_trace.push(json!(2));
                }
                assert_eq!(trace, expected_trace);
            }
        }
    }
}

fn spine(levels: usize) -> J {
    let mut child = integer(0);
    for _ in 0..levels {
        child = add(child, integer(0));
    }
    child
}

#[test]
fn selected_evaluator_proves_free_defaults_single_operand_entry_and_exact_visits() {
    // Each selected spine(10) costs 2047, list root costs 1, slice root costs 1.
    let operand = json!({"kind":"listExpression","elements":[spine(10), spine(10)]});
    let exact = slice(operand.clone(), None, None);
    for result in both(exact, true) {
        assert_eq!(
            result.expect("exactly 4096 entries; no default or repeated operand visits"),
            json!([0, 0])
        );
    }
    for (lower, upper) in [(Some(integer(0)), None), (None, Some(integer(2)))] {
        for result in both(slice(operand.clone(), lower, upper), true) {
            assert!(matches!(result, Err(Error::EvaluationVisits)));
        }
    }
    // With normal add wiring this AST is tiny: selected evaluation must propagate.
    for result in both(slice(operand, None, Some(integer(2))), false) {
        assert_eq!(result.expect("default evaluator"), json!([0, 0]));
    }
    // 2047 + 1023 + 511 + 511 + list + slice + lower + upper = 4096.
    let operand =
        json!({"kind":"listExpression","elements":[spine(10), spine(9), spine(8), spine(8)]});
    for result in both(
        slice(operand.clone(), Some(integer(0)), Some(integer(0))),
        true,
    ) {
        assert_eq!(result.expect("both endpoints at exactly 4096"), json!([]));
    }
    for result in both(
        slice(operand, Some(integer(0)), Some(add(integer(0), integer(0)))),
        true,
    ) {
        assert!(matches!(result, Err(Error::EvaluationVisits)));
    }
}

#[test]
fn exact_depth_counts_slice_roots_but_not_defaults_and_restores_siblings() {
    for wrappers in [63, 64] {
        let mut entry = json!({"kind":"list","value":[]});
        for _ in 0..wrappers {
            entry = slice(entry, None, None);
        }
        for result in both(entry, false) {
            if wrappers == 63 {
                assert_eq!(result.expect("64 active entries"), json!([]));
            } else {
                assert!(matches!(result, Err(Error::EvaluationDepth)));
            }
        }
    }
    for wrappers in [62, 63] {
        let mut endpoint = integer(0);
        for _ in 0..wrappers {
            endpoint = json!({"kind":"negate","operand":endpoint});
        }
        for result in both(
            slice(
                json!({"kind":"list","value":[]}),
                Some(endpoint.clone()),
                Some(endpoint),
            ),
            false,
        ) {
            if wrappers == 62 {
                assert_eq!(result.expect("each sibling reaches depth 64"), json!([]));
            } else {
                assert!(matches!(result, Err(Error::EvaluationDepth)));
            }
        }
    }
}

#[test]
fn immutable_mixed_children_outlive_callback_inputs_and_retain_certificates() {
    let text: Arc<str> = Arc::from("owned é🙂");
    let child =
        V::Record(ClosedRecord::new(vec![("x".into(), V::Text(text.clone()))]).expect("record"));
    let parent = list(vec![child.clone()]);
    let node = slice(integer(0), None, None);
    let retained = runtime::slice(
        &node,
        ["operand", "lower", "upper"],
        &mut EvaluationContext::default(),
        |_, _| Ok(parent.clone()),
    )
    .expect("owned result");
    let selected = runtime::list_at(&retained, &V::Integer(0)).expect("child");
    let (V::Record(a), V::Record(b)) = (&selected, &child) else {
        panic!("records")
    };
    assert!(std::ptr::eq(a.as_map(), b.as_map()));
    drop(node);
    drop(parent);
    drop(child);
    drop(selected);
    drop(text);
    assert_eq!(
        runtime::encode_json(retained).expect("surviving storage"),
        json!([{"x":"owned é🙂"}])
    );

    let shared = V::Record(
        ClosedRecord::new((0..64).map(|i| (format!("f{i}"), V::Null)).collect()).expect("65 nodes"),
    );
    let boundary = list(vec![shared; 63]); // Exactly 4096 occurrence nodes.
    let retained = runtime::slice_interpreted(
        &slice(integer(0), None, None),
        ["operand", "lower", "upper"],
        &mut EvaluationContext::default(),
        |_, _| Ok(boundary.clone()),
        &export("slice-control"),
    )
    .expect("recertified slice");
    assert!(matches!(
        ClosedList::new(vec![retained]),
        Err(Error::ListNodes)
    ));
}
