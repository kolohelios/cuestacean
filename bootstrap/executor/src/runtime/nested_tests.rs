use super::*;

fn closed(values: Vec<Value>) -> ClosedList {
    ClosedList::new(values).expect("validated closed list")
}

fn depth_limit() -> ClosedList {
    let mut list = closed(vec![]);
    for _ in 1..32 {
        list = closed(vec![Value::List(list)]);
    }
    list
}

fn node_limit() -> ClosedList {
    let child = closed(vec![Value::Null; 64]);
    closed(vec![Value::List(child); 63])
}

// Independent recursive oracle belongs only in tests, never parent construction.
fn logical_shape(value: &Value) -> (usize, usize) {
    match value {
        Value::List(list) => {
            let mut depth = 1;
            let mut nodes = 1;
            for child in list.as_slice() {
                let (child_depth, child_nodes) = logical_shape(child);
                depth = depth.max(1 + child_depth);
                nodes += child_nodes;
            }
            (depth, nodes)
        }
        Value::Null | Value::Boolean(_) | Value::Integer(_) | Value::Text(_) => (1, 1),
        _ => panic!("abstract value escaped validation"),
    }
}

#[test]
fn cached_shapes_match_logical_occurrences_and_clone_shares_storage() {
    let child = closed(vec![Value::Null, Value::List(closed(vec![]))]);
    let parent = closed(vec![Value::List(child.clone()); 3]);
    assert_eq!(child.shape, ValueShape { depth: 2, nodes: 3 });
    assert_eq!(
        parent.shape,
        ValueShape {
            depth: 3,
            nodes: 10
        }
    );
    let clone = parent.clone();
    assert!(Arc::ptr_eq(&parent.values, &clone.values));
    assert_eq!(clone.shape, parent.shape);
    for value in parent.as_slice() {
        let Value::List(retained) = value else {
            panic!("expected shared child");
        };
        assert!(Arc::ptr_eq(&child.values, &retained.values));
    }
    drop(parent);
    drop(child);
    for list in [closed(vec![]), clone, depth_limit(), node_limit()] {
        assert_eq!(
            logical_shape(&Value::List(list.clone())),
            (list.shape.depth, list.shape.nodes)
        );
    }
    assert_eq!(
        depth_limit().shape,
        ValueShape {
            depth: 32,
            nodes: 32
        }
    );
    assert_eq!(
        node_limit().shape,
        ValueShape {
            depth: 3,
            nodes: 4096
        }
    );
}

#[test]
fn prospective_include_is_pure_and_orders_type_depth_nodes() {
    let shape = ValueShape {
        depth: 32,
        nodes: 4096,
    };
    assert!(matches!(
        shape.list_include(&Value::Top, 7),
        Err(Error::ListElement { index: 7 })
    ));
    assert!(matches!(
        shape.list_include(&Value::List(depth_limit()), 7),
        Err(Error::ListDepth)
    ));
    assert!(matches!(
        shape.list_include(&Value::Null, 7),
        Err(Error::ListNodes)
    ));
    assert_eq!(
        shape,
        ValueShape {
            depth: 32,
            nodes: 4096
        }
    );
    assert_eq!(
        ValueShape::EMPTY
            .list_include(&Value::Null, 0)
            .expect("scalar"),
        ValueShape { depth: 2, nodes: 2 }
    );
    assert_eq!(ValueShape::EMPTY, ValueShape { depth: 1, nodes: 1 });
}

#[test]
fn successful_append_commits_cached_shape_and_consumes_candidate() {
    fn inspect(frame: &mut ListFrame<'_>) -> Result<(), Error> {
        frame.first_item()?;
        for expected_nodes in [4, 7] {
            frame.evaluate_item()?;
            frame.append_item()?;
            assert_eq!(
                frame.shape,
                ValueShape {
                    depth: 3,
                    nodes: expected_nodes
                }
            );
            assert!(frame.candidate.is_none());
            assert!(matches!(frame.stage, ListStage::Appended(_)));
            frame.next_item()?;
        }
        frame.finish_list()?;
        let Some(Value::List(list)) = frame.result.as_ref() else {
            panic!("expected finished list");
        };
        assert_eq!(list.shape, frame.shape);
        Ok(())
    }

    let child = closed(vec![Value::Null, Value::Null]);
    let result = list_expression_with_driver(
        &serde_json::json!({"elements": [null, null]}),
        "elements",
        &mut EvaluationContext::default(),
        &mut |_, _| Ok(Value::List(child.clone())),
        inspect,
    )
    .expect("validated appends");
    assert_eq!(logical_shape(&result), (3, 7));
}

#[test]
fn failed_append_retains_candidate_buffer_shape_and_cursor() {
    for (candidate, width_full) in [
        (Value::Top, false),
        (Value::List(depth_limit()), false),
        (Value::List(node_limit()), false),
        (Value::Top, true),
    ] {
        let items = [JsonValue::Null, JsonValue::Null, JsonValue::Null];
        let mut context = EvaluationContext::default();
        let mut evaluate = |_: &JsonValue, _: &mut EvaluationContext| Ok(candidate.clone());
        let buffer = if width_full {
            vec![Value::Null; 64]
        } else {
            vec![]
        };
        let shape = if width_full {
            ValueShape {
                depth: 2,
                nodes: 65,
            }
        } else {
            ValueShape::EMPTY
        };
        let mut frame = ListFrame {
            items: &items,
            context: &mut context,
            evaluate: &mut evaluate,
            stage: ListStage::Start,
            candidate: None,
            buffer: buffer.clone(),
            shape,
            result: None,
            fuel: 4096,
        };
        // Reverse selection distinguishes the cursor index from buffer length.
        frame.last_item().expect("select last");
        frame.evaluate_item().expect("evaluate candidate");
        for _ in 0..2 {
            let error = frame.append_item().expect_err("prospective parent rejects");
            match (&candidate, width_full) {
                (_, true) => assert!(matches!(error, Error::ListCapacity)),
                (Value::Top, false) => {
                    assert!(matches!(error, Error::ListElement { index: 2 }));
                }
                (Value::List(list), false) if list.shape.depth == 32 => {
                    assert!(matches!(error, Error::ListDepth));
                }
                _ => assert!(matches!(error, Error::ListNodes)),
            }
            assert_eq!(frame.candidate.as_ref(), Some(&candidate));
            assert_eq!(frame.buffer, buffer);
            assert_eq!(frame.shape, shape);
            assert!(matches!(frame.stage, ListStage::Evaluated(2)));
            assert!(frame.result.is_none());
            assert!(matches!(frame.next_item(), Err(Error::ListState { .. })));
        }
    }
}

#[test]
fn append_shape_failure_precedes_later_callback_even_without_policy_admission() {
    fn permissive(frame: &mut ListFrame<'_>) -> Result<(), Error> {
        frame.first_item()?;
        while frame.has_item()? {
            frame.evaluate_item()?;
            frame.append_item()?;
            frame.next_item()?;
        }
        frame.finish_list()
    }

    for candidate in [
        Value::Top,
        Value::List(depth_limit()),
        Value::List(node_limit()),
    ] {
        let mut calls = 0;
        let mut evaluate = |_: &JsonValue, _: &mut EvaluationContext| {
            calls += 1;
            if calls == 1 {
                Ok(candidate.clone())
            } else {
                Err(Error::EvaluationVisits)
            }
        };
        let result = list_expression_with_driver(
            &serde_json::json!({"elements": [null, null]}),
            "elements",
            &mut EvaluationContext::default(),
            &mut evaluate,
            permissive,
        );
        match candidate {
            Value::Top => assert!(matches!(result, Err(Error::ListElement { index: 0 }))),
            Value::List(list) if list.shape.depth == 32 => {
                assert!(matches!(result, Err(Error::ListDepth)));
            }
            _ => assert!(matches!(result, Err(Error::ListNodes))),
        }
        assert_eq!(calls, 1);
    }
}

#[test]
fn accumulated_node_failure_retains_full_prefix_and_pending_candidate() {
    fn inspect(frame: &mut ListFrame<'_>) -> Result<(), Error> {
        frame.first_item()?;
        for _ in 0..63 {
            frame.evaluate_item()?;
            frame.append_item()?;
            frame.next_item()?;
        }
        frame.evaluate_item()?;
        let before = frame.buffer.clone();
        let shape = frame.shape;
        assert_eq!(shape.nodes, 4096);
        for _ in 0..2 {
            assert!(matches!(frame.append_item(), Err(Error::ListNodes)));
            assert_eq!(frame.buffer, before);
            assert_eq!(frame.shape, shape);
            assert_eq!(frame.candidate, Some(Value::Null));
            assert!(matches!(frame.stage, ListStage::Evaluated(63)));
            assert!(frame.result.is_none());
        }
        Err(Error::ListNodes)
    }
    let child = closed(vec![Value::Null; 64]);
    let mut calls = 0;
    let result = list_expression_with_driver(
        &serde_json::json!({"elements": vec![JsonValue::Null; 64]}),
        "elements",
        &mut EvaluationContext::default(),
        &mut |_, _| {
            calls += 1;
            Ok(if calls <= 63 {
                Value::List(child.clone())
            } else {
                Value::Null
            })
        },
        inspect,
    );
    assert!(matches!(result, Err(Error::ListNodes)));
    assert_eq!(calls, 64);
}

#[test]
fn finish_revalidates_buffer_instead_of_trusting_frame_cache() {
    for candidate in [
        Value::List(closed(vec![Value::Null])),
        Value::Top,
        Value::List(depth_limit()),
        Value::List(node_limit()),
    ] {
        let items = [JsonValue::Null];
        let mut context = EvaluationContext::default();
        let mut evaluate = |_: &JsonValue, _: &mut EvaluationContext| Ok(Value::Null);
        // Simulate a faulty internal buffer/cache; no such mutation is public.
        let mut frame = ListFrame {
            items: &items,
            context: &mut context,
            evaluate: &mut evaluate,
            stage: ListStage::Exhausted,
            candidate: None,
            buffer: vec![candidate.clone()],
            shape: ValueShape::EMPTY,
            result: None,
            fuel: 4096,
        };
        let result = frame.finish_list();
        match candidate {
            Value::List(list) if list.shape.nodes == 2 => {
                result.expect("valid buffer recomputes shape");
                let Some(Value::List(result)) = frame.result else {
                    panic!("expected finished list");
                };
                assert_eq!(result.shape, ValueShape { depth: 3, nodes: 3 });
                assert!(matches!(frame.stage, ListStage::Done));
            }
            candidate => {
                match candidate {
                    Value::Top => assert!(matches!(result, Err(Error::ListElement { index: 0 }))),
                    Value::List(list) if list.shape.depth == 32 => {
                        assert!(matches!(result, Err(Error::ListDepth)));
                    }
                    _ => assert!(matches!(result, Err(Error::ListNodes))),
                }
                assert!(frame.result.is_none());
                assert!(matches!(frame.stage, ListStage::Exhausted));
            }
        }
    }
}
