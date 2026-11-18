use bootstrap_executor::{Error, execute_json, generated, parser, pratt, runtime, unification};
use runtime::{ClosedList, EvaluationContext, ListDriver, ListFrame, Value as V};
use serde_json::{Value as J, json};
use std::{path::Path, sync::Arc};

#[path = "../../generated/list-control-scalars.rs"]
mod historical;
#[path = "../../generated/list-control.rs"]
mod production;
#[path = "../../generated/list-control-reverse.rs"]
mod reverse;

fn export(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CUESTACEAN_BOOTSTRAP_JSON")).join(format!("{name}.json")),
    )
    .expect("fresh export")
}
fn list(values: Vec<V>) -> V {
    V::List(ClosedList::new(values).expect("validated list"))
}
fn singleton(value: V) -> V {
    list(vec![value])
}
fn depth32() -> V {
    let mut value = V::Null;
    for _ in 0..31 {
        value = singleton(value);
    }
    value
}
fn nodes4096() -> V {
    let shared = list(vec![V::Null; 64]); // 65 nodes, depth 2.
    list(vec![shared; 63]) // 1 + 63 * 65, depth 3.
}
fn expression(elements: J) -> J {
    json!({"kind":"listExpression","elements":elements})
}
fn integer(value: i64) -> J {
    json!({"kind":"integer","value":value})
}
fn binary(kind: &str, left: J, right: J) -> J {
    json!({"kind":kind,"left":left,"right":right})
}
fn both(entry: J) -> [Result<J, Error>; 2] {
    let input = json!({"formatVersion":1,"entry":entry}).to_string();
    [
        execute_json(&input, &export("evaluation")),
        generated::execute_json(&input),
    ]
}
fn reference(frame: &mut ListFrame<'_>) -> Result<(), Error> {
    pratt::interpret_list(&export("list-control"), frame)
}
fn reference_reverse(frame: &mut ListFrame<'_>) -> Result<(), Error> {
    pratt::interpret_list(&export("tests/valid/list-control-reverse"), frame)
}
fn reference_historical(frame: &mut ListFrame<'_>) -> Result<(), Error> {
    pratt::interpret_list(&export("tests/valid/list-control-scalars"), frame)
}
// Callback values are already-built immutable DAGs, not enormous repeated ASTs.
// An optional sentinel errors if control reaches it; trace proves early stopping.
fn run(values: &[V], trailing_error: bool, driver: ListDriver) -> (Result<V, Error>, Vec<usize>) {
    let count = values.len() + usize::from(trailing_error);
    let node = expression(json!((0..count).collect::<Vec<_>>()));
    let mut trace = Vec::new();
    let result = runtime::list_expression_with_driver(
        &node,
        "elements",
        &mut EvaluationContext::default(),
        &mut |node, ctx| {
            let index = node.as_u64().expect("callback index") as usize;
            trace.push(index);
            ctx.visit(|_| values.get(index).cloned().ok_or(Error::DivisionByZero))
        },
        driver,
    );
    (result, trace)
}
fn as_list(value: &V) -> &ClosedList {
    match value {
        V::List(value) => value,
        other => panic!("expected list, got {other:?}"),
    }
}

#[test]
fn value_depth_counts_scalar_and_empty_roots_without_public_metrics() {
    for mut value in [V::Null, list(vec![])] {
        // Scalar and empty list each start at depth 1.
        for _ in 0..31 {
            value = singleton(value);
        }
        let error = ClosedList::new(vec![value]).expect_err("depth 33");
        assert!(matches!(error, Error::ListDepth));
        assert_eq!(error.to_string(), "closed list exceeds 32 value levels");
    }
    let value = depth32();
    let mut selected = value;
    for _ in 0..31 {
        selected = runtime::list_at(&selected, &V::Integer(0)).expect("nested index");
    }
    assert_eq!(selected, V::Null);
}

#[test]
fn logical_nodes_charge_shared_occurrences_at_exact_4096_and_4097() {
    let shared = list(vec![V::Null; 64]);
    let accepted = ClosedList::new(vec![shared.clone(); 63]).expect("4096 nodes");
    assert_eq!(accepted.len(), 63);
    for child in accepted.as_slice() {
        assert_eq!(
            as_list(child).as_slice().as_ptr(),
            as_list(&shared).as_slice().as_ptr(),
            "all occurrences retain the same allocation"
        );
    }
    let output = runtime::encode_json(V::List(accepted)).expect("bounded expanded output");
    assert_eq!(output, json!(vec![vec![J::Null; 64]; 63]));
    let mut rejected = vec![shared; 63];
    rejected.push(V::Null); // Width 64, depth 3, exactly 4097 nodes.
    let error = ClosedList::new(rejected).expect_err("4097 logical nodes");
    assert!(matches!(error, Error::ListNodes));
    assert_eq!(
        error.to_string(),
        "closed list exceeds 4096 logical value nodes"
    );
    assert!(matches!(
        ClosedList::new(vec![nodes4096()]),
        Err(Error::ListNodes)
    ));
}

#[test]
fn constructor_width_and_element_order_remain_independent() {
    let mut oversized = vec![V::Null; 65];
    oversized[0] = V::Top;
    oversized[1] = depth32();
    assert!(matches!(
        ClosedList::new(oversized),
        Err(Error::ListCapacity)
    ));
    for (shape, message) in [
        (depth32(), "closed list exceeds 32 value levels"),
        (nodes4096(), "closed list exceeds 4096 logical value nodes"),
    ] {
        let error = ClosedList::new(vec![V::Int, shape.clone()]).expect_err("type first");
        assert!(matches!(error, Error::ListElement { index: 0 }));
        assert_eq!(
            error.to_string(),
            "closed list element 0 requires a concrete scalar or closed collection"
        );
        let error = ClosedList::new(vec![shape, V::Int]).expect_err("shape first");
        assert_eq!(error.to_string(), message);
    }
    for abstract_value in [
        V::Top,
        V::Int,
        V::Bool,
        V::String,
        V::Bounds {
            lower: Some(1),
            upper: Some(1),
        },
    ] {
        assert!(matches!(
            ClosedList::new(vec![list(vec![]), abstract_value]),
            Err(Error::ListElement { index: 1 })
        ));
    }
}

#[test]
fn prospective_append_shape_rejects_before_later_callback_errors() {
    // Omits CUE admission deliberately: append must still defend representation.
    fn unchecked(frame: &mut ListFrame<'_>) -> Result<(), Error> {
        frame.first_item()?;
        while frame.has_item()? {
            frame.evaluate_item()?;
            frame.append_item()?;
            frame.next_item()?;
        }
        frame.finish_list()
    }
    for driver in [production::execute as ListDriver, reference, unchecked] {
        for (shape, message) in [
            (depth32(), "closed list exceeds 32 value levels"),
            (nodes4096(), "closed list exceeds 4096 logical value nodes"),
        ] {
            let (result, trace) = run(&[shape], true, driver);
            assert_eq!(result.expect_err("prospective parent").to_string(), message);
            assert_eq!(trace, vec![0], "no later error callback runs");
        }
        let (result, trace) = run(&[V::Int], true, driver);
        assert!(matches!(result, Err(Error::ListElement { index: 0 })));
        assert_eq!(trace, vec![0]);
        let (result, trace) = run(&[list(vec![])], true, driver);
        assert!(matches!(result, Err(Error::DivisionByZero)));
        assert_eq!(trace, vec![0, 1]);
        let (result, trace) = run(&vec![V::Top; 65], false, driver);
        assert!(matches!(result, Err(Error::ListCapacity)));
        assert!(trace.is_empty());
    }
}

#[test]
fn frame_exact_occurrence_boundary_matches_constructor() {
    let shared = list(vec![V::Null; 64]);
    for driver in [production::execute as ListDriver, reference] {
        let mut values = vec![shared.clone(); 63];
        let (result, trace) = run(&values, false, driver);
        assert_eq!(result.expect("4096 nodes"), nodes4096());
        assert_eq!(trace, (0..63).collect::<Vec<_>>());
        values.push(V::Null);
        let (result, trace) = run(&values, false, driver);
        assert!(matches!(result, Err(Error::ListNodes)));
        assert_eq!(trace, (0..64).collect::<Vec<_>>());
    }
}

#[test]
fn selected_child_retains_only_its_own_storage_after_parent_drop() {
    let text: Arc<str> = Arc::from("owned é🙂");
    let child = list(vec![V::Text(Arc::clone(&text))]);
    let parent = singleton(child.clone());
    let parent_storage = as_list(&parent).as_slice().as_ptr();
    let selected = runtime::list_at(&parent, &V::Integer(0)).expect("list child");
    assert_eq!(
        as_list(&selected).as_slice().as_ptr(),
        as_list(&child).as_slice().as_ptr()
    );
    assert_ne!(as_list(&selected).as_slice().as_ptr(), parent_storage);
    assert_eq!(
        Arc::strong_count(&text),
        2,
        "container clones do not clone text"
    );
    drop(parent);
    drop(child);
    assert_eq!(
        runtime::encode_json(selected).expect("owned child"),
        json!(["owned é🙂"])
    );
    assert_eq!(Arc::strong_count(&text), 1);
}

#[test]
fn nested_expression_output_content_policies_length_and_chained_indices() {
    let nested = expression(json!([
        expression(json!([])),
        expression(json!([
            {"kind":"null"}, {"kind":"true"},
            integer(i64::MIN), integer(i64::MAX),
            {"kind":"string","value":"é🙂"}
        ]))
    ]));
    let exact = r#"[[],[null,true,-9223372036854775808,9223372036854775807,"é🙂"]]"#;
    for result in both(nested.clone()) {
        assert_eq!(result.expect("nested output").to_string(), exact);
    }
    for (kind, expected) in [("equal", true), ("notEqual", false)] {
        for result in both(binary(kind, nested.clone(), nested.clone())) {
            assert_eq!(result.expect("nested content"), json!(expected));
        }
    }
    let different = expression(json!([expression(json!([]))]));
    for result in both(binary("equal", nested.clone(), different.clone())) {
        assert_eq!(result.expect("different shape"), json!(false));
    }
    for result in both(binary("unify", nested.clone(), different)) {
        assert!(matches!(result, Err(Error::ListConflict)));
    }
    for other in [nested.clone(), json!({"kind":"top"})] {
        for result in both(binary("unify", nested.clone(), other)) {
            assert_eq!(result.expect("identity").to_string(), exact);
        }
    }
    for result in both(json!({"kind":"length","operand":nested})) {
        assert_eq!(result.expect("direct element length"), json!(2));
    }
    let grammar = export("grammar");
    for (source, expected) in [
        ("[[]]", json!([[]])),
        ("len([[],[1,2]])", json!(2)),
        ("[[1+2],[4]][0][0]", json!(3)),
        ("[[\"é🙂\"]][0]", json!(["é🙂"])),
        ("[[1]] == [[2]]", json!(false)),
    ] {
        for tree in [
            parser::parse(source, &grammar),
            parser::parse_interpreted(source, &grammar, &export("pratt")),
        ] {
            let input = tree.expect(source).to_string();
            for result in [
                execute_json(&input, &export("evaluation")),
                generated::execute_json(&input),
            ] {
                assert_eq!(result.expect(source), expected, "{source}");
            }
        }
    }
}

#[test]
fn recursive_expression_evaluation_keeps_shape_and_ast_limits_separate() {
    for (wrappers, accepted) in [(31, true), (32, false)] {
        let mut entry = json!({"kind":"null"});
        let mut expected = J::Null;
        for _ in 0..wrappers {
            entry = expression(json!([entry]));
            expected = json!([expected]);
        }
        for result in both(entry) {
            if accepted {
                assert_eq!(result.expect("32 value levels"), expected);
            } else {
                assert!(matches!(result, Err(Error::ListDepth)));
            }
        }
    }
    // Scalar-only recursion exceeds the unchanged AST budget before admission.
    let mut entry = integer(0);
    for _ in 0..63 {
        entry = json!({"kind":"negate","operand":entry});
    }
    for result in both(expression(json!([entry]))) {
        assert!(matches!(result, Err(Error::EvaluationDepth)));
    }
}

#[test]
fn actual_historical_admission_and_reverse_control_have_fixed_outcomes() {
    let values = [list(vec![]), list(vec![V::Integer(7)])];
    for driver in [production::execute as ListDriver, reference] {
        let (result, trace) = run(&values, false, driver);
        assert_eq!(
            runtime::encode_json(result.expect("nested admission")).expect("JSON"),
            json!([[], [7]])
        );
        assert_eq!(trace, vec![0, 1]);
    }
    for driver in [historical::execute as ListDriver, reference_historical] {
        let (result, trace) = run(&values, false, driver);
        assert!(matches!(result, Err(Error::ListElement { index: 0 })));
        assert_eq!(trace, vec![0]);
        let (result, trace) = run(&[V::Integer(1), V::Integer(2)], false, driver);
        assert_eq!(
            runtime::encode_json(result.expect("old scalar profile")).expect("JSON"),
            json!([1, 2])
        );
        assert_eq!(trace, vec![0, 1]);
    }
    for driver in [reverse::execute as ListDriver, reference_reverse] {
        let (result, trace) = run(&values, false, driver);
        assert_eq!(
            runtime::encode_json(result.expect("reverse nested")).expect("JSON"),
            json!([[7], []])
        );
        assert_eq!(trace, vec![1, 0]);
    }
}
