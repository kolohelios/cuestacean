use bootstrap_executor::{Error, execute_json, generated, pratt, runtime};
use runtime::{ConjunctionDriver, ConjunctionFrame, EvaluationContext, Value as V};
use serde_json::{Value as J, json};
use std::{path::Path, sync::Arc};

#[path = "../../generated/evaluation-left-twice.rs"]
mod left_twice;
#[path = "../../generated/conjunction-control.rs"]
mod production;
#[path = "../../generated/conjunction-control-reverse.rs"]
mod reverse;

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
fn conjunction(operands: J) -> J {
    json!({"kind":"conjunction","operands":operands})
}
fn program(entry: J) -> String {
    json!({"formatVersion":1,"entry":entry}).to_string()
}
fn both(entry: J) -> [Result<J, Error>; 2] {
    let input = program(entry);
    [
        execute_json(&input, &export("evaluation")),
        generated::execute_json(&input),
    ]
}
fn forward(frame: &mut ConjunctionFrame<'_>) -> Result<(), Error> {
    pratt::interpret_conjunction(&export("conjunction-control"), frame)
}
fn backward(frame: &mut ConjunctionFrame<'_>) -> Result<(), Error> {
    pratt::interpret_conjunction(&export("tests/valid/conjunction-control-reverse"), frame)
}
fn drivers() -> [(ConjunctionDriver, bool); 4] {
    [
        (production::execute, false),
        (forward, false),
        (reverse::execute, true),
        (backward, true),
    ]
}
fn evaluate(node: &J, context: &mut EvaluationContext) -> Result<V, Error> {
    context.visit(|_| match node["kind"].as_str() {
        Some("integer") => runtime::integer(node, "value"),
        Some("intType") => Ok(V::Int),
        Some("top") => Ok(V::Top),
        Some("divide") => Err(Error::DivisionByZero),
        _ => Err(Error::Kind),
    })
}
fn direct(operands: J, driver: ConjunctionDriver) -> (Result<V, Error>, Vec<J>) {
    let mut trace = Vec::new();
    let result = runtime::conjunction_with_driver(
        &conjunction(operands),
        "operands",
        &mut EvaluationContext::default(),
        &mut |node, context| {
            trace.push(node.clone());
            evaluate(node, context)
        },
        runtime::unify,
        driver,
    );
    (result, trace)
}
fn fails(entry: J, expected: Error) {
    for result in both(entry) {
        let actual = result.expect_err("independently expected error");
        assert_eq!(
            std::mem::discriminant(&actual),
            std::mem::discriminant(&expected)
        );
        assert_eq!(actual.to_string(), expected.to_string());
    }
}

#[test]
fn independent_complete_asts_cover_refinement_scalars_and_exact_extrema() {
    let legacy = program(conjunction(json!([{"kind":"intType"}, integer(7)])));
    assert_eq!(
        left_twice::execute(&legacy).expect("legacy refined integer"),
        7
    );
    for (operands, expected) in [
        (json!([integer(7)]), json!(7)),
        (
            json!([{"kind":"intType"}, {"kind":"lowerBound","value":2},
            {"kind":"upperBound","value":4}, integer(3)]),
            json!(3),
        ),
        (
            json!([{"kind":"boolType"}, {"kind":"top"}, {"kind":"true"}]),
            json!(true),
        ),
        (
            json!([{"kind":"stringType"}, {"kind":"string","value":"é🙂"}]),
            json!("é🙂"),
        ),
        (
            json!([{"kind":"top"}, {"kind":"null"}, {"kind":"null"}]),
            json!(null),
        ),
        (
            json!([{"kind":"lowerBound","value":i64::MIN}, integer(i64::MIN)]),
            json!(i64::MIN),
        ),
        (
            json!([{"kind":"upperBound","value":i64::MAX}, integer(i64::MAX)]),
            json!(i64::MAX),
        ),
        (
            json!([binary("add", integer(2), integer(3)), integer(5)]),
            json!(5),
        ),
    ] {
        for result in both(conjunction(operands)) {
            assert_eq!(result.expect("concrete fold"), expected);
        }
    }
    for (operands, expected) in [
        (json!([]), Error::IncompleteValue),
        (json!([{"kind":"top"}]), Error::IncompleteValue),
        (json!([{"kind":"intType"}]), Error::Incomplete),
        (json!([{"kind":"boolType"}]), Error::IncompleteBool),
        (json!([{"kind":"stringType"}]), Error::IncompleteString),
        (
            json!([{"kind":"lowerBound","value":3}, {"kind":"upperBound","value":3}]),
            Error::IncompleteBounds,
        ),
    ] {
        fails(conjunction(operands), expected);
    }
    for entry in [
        binary("unify", conjunction(json!([])), integer(9)),
        binary("unify", integer(9), conjunction(json!([]))),
        conjunction(json!([conjunction(json!([])), integer(9)])),
    ] {
        for result in both(entry) {
            assert_eq!(result.expect("empty top identity"), json!(9));
        }
    }
}

#[test]
fn closed_collections_keep_existing_unification_and_conflict_errors() {
    let list = json!({"kind":"listExpression","elements":[
        {"kind":"recordExpression","fields":[{"name":"x","value":integer(i64::MAX)}]},
        {"kind":"string","value":"owned 🦀"}
    ]});
    let record = json!({"kind":"recordExpression","fields":[{"name":"items","value":list}]});
    for (child, expected) in [
        (list.clone(), json!([{"x":i64::MAX}, "owned 🦀"])),
        (
            record.clone(),
            json!({"items":[{"x":i64::MAX}, "owned 🦀"]}),
        ),
    ] {
        for result in both(conjunction(json!([{"kind":"top"}, child.clone(), child]))) {
            assert_eq!(result.expect("closed equality"), expected);
        }
    }
    for (operands, expected) in [
        (
            json!([integer(1), integer(2)]),
            Error::Conflict { left: 1, right: 2 },
        ),
        (
            json!([{"kind":"true"}, {"kind":"false"}]),
            Error::BooleanConflict {
                left: true,
                right: false,
            },
        ),
        (
            json!([{"kind":"string","value":"a"}, {"kind":"string","value":"b"}]),
            Error::StringConflict {
                left: "a".into(),
                right: "b".into(),
            },
        ),
        (
            json!([{"kind":"intType"}, {"kind":"true"}]),
            Error::TypeConflict,
        ),
        (
            json!([{"kind":"lowerBound","value":4}, {"kind":"upperBound","value":3}]),
            Error::BoundsConflict,
        ),
        (
            json!([list, {"kind":"list","value":[]}]),
            Error::ListConflict,
        ),
        (
            json!([record, {"kind":"record","fields":[]}]),
            Error::RecordConflict,
        ),
    ] {
        fails(conjunction(operands), expected);
    }
}

#[test]
fn array_shape_and_capacity_are_checked_before_any_callbacks() {
    fails(
        json!({"kind":"conjunction"}),
        Error::MissingField {
            field: "operands".into(),
        },
    );
    for operands in [json!(null), json!({}), json!(1), json!(true)] {
        fails(
            conjunction(operands),
            Error::Array {
                field: "operands".into(),
            },
        );
    }
    fails(
        conjunction(json!(vec![json!({}); 65])),
        Error::ConjunctionCapacity,
    );
    for (driver, _) in drivers() {
        let (result, trace) = direct(json!(vec![json!({}); 65]), driver);
        assert!(matches!(result, Err(Error::ConjunctionCapacity)));
        assert!(trace.is_empty());
        let (result, trace) = direct(json!(vec![integer(7); 64]), driver);
        assert_eq!(result.expect("64 merges"), V::Integer(7));
        assert_eq!(trace, vec![integer(7); 64]);
        assert_eq!(direct(json!([]), driver).0.expect("empty"), V::Top);
    }
    for result in both(conjunction(json!(vec![integer(7); 64]))) {
        assert_eq!(result.expect("64 AST children"), json!(7));
    }
}

#[test]
fn real_reverse_control_changes_callbacks_and_first_error_not_just_commutative_output() {
    let divide = binary("divide", integer(1), integer(0));
    for (driver, reversed) in drivers() {
        let operands = vec![json!({"kind":"intType"}), integer(1), json!({"kind":"top"})];
        let (result, trace) = direct(json!(operands), driver);
        assert_eq!(
            result.expect("abstract operands are not list elements"),
            V::Integer(1)
        );
        let expected: Vec<_> = if reversed {
            operands.into_iter().rev().collect()
        } else {
            operands
        };
        assert_eq!(trace, expected);
        let (result, trace) = direct(json!([integer(1), integer(2), divide]), driver);
        if reversed {
            assert!(matches!(result, Err(Error::DivisionByZero)));
            assert_eq!(trace, vec![divide.clone()]);
        } else {
            assert!(matches!(result, Err(Error::Conflict { left: 1, right: 2 })));
            assert_eq!(trace, vec![integer(1), integer(2)]);
        }
    }
    fails(
        conjunction(json!([integer(1), integer(2), divide])),
        Error::Conflict { left: 1, right: 2 },
    );
    fails(
        conjunction(json!([integer(1), divide, integer(2)])),
        Error::DivisionByZero,
    );
}

#[test]
fn selected_left_twice_and_depth_share_the_existing_ast_context() {
    let mut spine = integer(0);
    for _ in 0..11 {
        spine = binary("add", spine, integer(0));
    }
    // Selected add repeats left: T(11)=4095; conjunction parent is exactly one entry.
    for (operands, success) in [(json!([spine]), true), (json!([spine, integer(0)]), false)] {
        let input = program(conjunction(operands));
        for result in [
            execute_json(&input, &export("tests/valid/evaluation-left-twice")),
            left_twice::execute_json(&input),
        ] {
            if success {
                assert_eq!(result.expect("4096 visits"), json!(0));
            } else {
                assert!(matches!(result, Err(Error::EvaluationVisits)));
            }
        }
    }
    for (wrappers, success) in [(62, true), (63, false)] {
        let mut child = integer(0);
        for _ in 0..wrappers {
            child = json!({"kind":"negate","operand":child});
        }
        for result in both(conjunction(json!([child]))) {
            if success {
                assert_eq!(result.expect("64 active entries"), json!(0));
            } else {
                assert!(matches!(result, Err(Error::EvaluationDepth)));
            }
        }
    }
}

#[test]
fn immutable_children_and_certificates_survive_input_callback_and_accumulator_drop() {
    let key = V::Text(Arc::from("x"));
    let child = runtime::record(
        &json!({"fields":[{"name":"x","value":"é🙂 owned"}]}),
        "fields",
    )
    .expect("certified record");
    let V::Text(text) = runtime::record_get(&child, &key).expect("original text") else {
        panic!("text")
    };
    let node = conjunction(json!([integer(0)]));
    let folded = runtime::conjunction(
        &node,
        "operands",
        &mut EvaluationContext::default(),
        |_, _| Ok(child.clone()),
    )
    .expect("identity preserves child");
    drop(node);
    drop(child);
    let retained = runtime::list_expression_with_driver(
        &json!({"elements":[integer(0)]}),
        "elements",
        &mut EvaluationContext::default(),
        &mut |_, _| Ok(folded.clone()),
        |frame| pratt::interpret_list(&export("list-control"), frame),
    )
    .expect("existing certificate reusable as nested child");
    drop(folded);
    let record = runtime::list_at(&retained, &V::Integer(0)).expect("retained record");
    let V::Text(selected) = runtime::record_get(&record, &key).expect("text") else {
        panic!("text")
    };
    assert!(Arc::ptr_eq(&text, &selected));
    drop(text);
    drop(record);
    drop(retained);
    assert_eq!(&*selected, "é🙂 owned");
}
