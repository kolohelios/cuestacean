use super::*;
use serde_json::json;

#[derive(Clone, Copy)]
enum Parent {
    List,
    Record,
}

impl Parent {
    fn other(self) -> Self {
        match self {
            Self::List => Self::Record,
            Self::Record => Self::List,
        }
    }

    fn build(self, children: Vec<Value>) -> Result<Value, Error> {
        match self {
            Self::List => ClosedList::new(children).map(Value::List),
            Self::Record => ClosedRecord::new(
                children
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| (index.to_string(), value))
                    .collect(),
            )
            .map(Value::Record),
        }
    }

    fn assert_error(self, result: Result<Value, Error>, expected: ShapeError) {
        let error = result.expect_err("invalid prospective parent");
        assert!(matches!(
            (self, expected, error),
            (Self::List, ShapeError::Element, Error::ListElement { .. })
                | (Self::List, ShapeError::Depth, Error::ListDepth)
                | (Self::List, ShapeError::Nodes, Error::ListNodes)
                | (
                    Self::Record,
                    ShapeError::Element,
                    Error::RecordElement { .. }
                )
                | (Self::Record, ShapeError::Depth, Error::RecordDepth)
                | (Self::Record, ShapeError::Nodes, Error::RecordNodes)
        ));
    }
}

fn alternating(parent: Parent, depth: usize) -> Result<Value, Error> {
    if depth == 1 {
        parent.build(vec![])
    } else {
        parent.build(vec![alternating(parent.other(), depth - 1)?])
    }
}

fn certificate(value: &Value) -> ValueShape {
    match value {
        Value::List(list) => list.shape,
        Value::Record(record) => record.shape,
        _ => ValueShape::EMPTY,
    }
}

// Independent test-only traversal, including every repeated child occurrence.
fn measured(value: &Value) -> (usize, usize) {
    let children: Vec<_> = match value {
        Value::List(list) => list.as_slice().iter().collect(),
        Value::Record(record) => record.as_map().values().collect(),
        _ => return (1, 1),
    };
    children.into_iter().fold((1, 1), |(depth, nodes), child| {
        let (child_depth, child_nodes) = measured(child);
        (depth.max(1 + child_depth), nodes + child_nodes)
    })
}

#[test]
fn alternating_depth_and_mixed_occurrence_boundaries() -> Result<(), Error> {
    for parent in [Parent::List, Parent::Record] {
        let deepest = alternating(parent.other(), 32)?;
        assert_eq!(measured(&deepest), (32, 32));
        assert_eq!(
            certificate(&deepest),
            ValueShape {
                depth: 32,
                nodes: 32
            }
        );
        parent.assert_error(parent.build(vec![deepest]), ShapeError::Depth);
        let boundary = alternating(parent, 32)?;
        assert_eq!(measured(&boundary), (32, 32));
        // Mix both kinds in one prefix, sharing each 65-node child repeatedly.
        let left = parent.build(vec![Value::Null; 64])?;
        let right = parent.other().build(vec![Value::Null; 64])?;
        let mut prefix: Vec<_> = (0..63)
            .map(|index| {
                if index % 2 == 0 {
                    left.clone()
                } else {
                    right.clone()
                }
            })
            .collect();
        let full = parent.build(prefix.clone())?;
        assert_eq!(measured(&full), (3, 4096));
        assert_eq!(
            certificate(&full),
            ValueShape {
                depth: 3,
                nodes: 4096
            }
        );
        for (candidate, expected) in [
            (Value::Top, ShapeError::Element),
            (alternating(parent.other(), 32)?, ShapeError::Depth),
            (Value::Null, ShapeError::Nodes),
        ] {
            let mut entries = prefix.clone();
            entries.push(candidate);
            parent.assert_error(parent.build(entries), expected);
        }
        prefix.extend([Value::Top, Value::Top]);
        assert!(matches!(
            (parent, parent.build(prefix)),
            (Parent::List, Err(Error::ListCapacity)) | (Parent::Record, Err(Error::RecordCapacity))
        ));
    }
    let deep = alternating(Parent::List, 32)?;
    assert!(matches!(
        ClosedRecord::new(vec![("".into(), deep.clone())]),
        Err(Error::RecordName { index: 0 })
    ));
    assert!(matches!(
        ClosedRecord::new(vec![("x".into(), Value::Null), ("x".into(), deep)]),
        Err(Error::RecordDuplicate { .. })
    ));
    Ok(())
}

fn list_prefix(frame: &mut ListFrame<'_>) -> Result<(), Error> {
    frame.first_item()?;
    for _ in 0..63 {
        frame.evaluate_item()?;
        frame.append_item()?;
        frame.next_item()?;
    }
    frame.evaluate_item()?;
    let buffer = frame.buffer.clone();
    let shape = frame.shape;
    assert_eq!(shape.nodes, 4096);
    for _ in 0..2 {
        assert!(matches!(frame.append_item(), Err(Error::ListNodes)));
        assert_eq!(frame.buffer, buffer);
        assert_eq!(frame.shape, shape);
        assert_eq!(frame.candidate, Some(Value::Null));
        assert!(matches!(frame.stage, ListStage::Evaluated(63)));
        assert!(frame.result.is_none());
    }
    // A faulty internal driver cannot evade the final constructor with a stale cache.
    frame.buffer.push(Value::Null);
    frame.shape = ValueShape::EMPTY;
    frame.stage = ListStage::Exhausted;
    assert!(matches!(frame.finish_list(), Err(Error::ListNodes)));
    assert!(frame.result.is_none());
    Err(Error::ListNodes)
}

fn record_prefix(frame: &mut RecordFrame<'_>) -> Result<(), Error> {
    frame.first_item()?;
    for _ in 0..63 {
        frame.evaluate_item()?;
        frame.append_item()?;
        frame.next_item()?;
    }
    frame.evaluate_item()?;
    let values = frame.values.clone();
    let name = frame.name.clone();
    let shape = frame.shape;
    assert_eq!(shape.nodes, 4096);
    for _ in 0..2 {
        assert!(matches!(frame.append_item(), Err(Error::RecordNodes)));
        assert_eq!(frame.values, values);
        assert_eq!(frame.shape, shape);
        assert_eq!(frame.name, name);
        assert_eq!(frame.candidate, Some(Value::Null));
        assert!(matches!(frame.stage, RecordStage::Evaluated(63)));
        assert!(frame.result.is_none());
    }
    frame.values.insert("63".into(), Value::Null);
    frame.shape = ValueShape::EMPTY;
    frame.stage = RecordStage::Exhausted;
    assert!(matches!(frame.finish_record(), Err(Error::RecordNodes)));
    assert!(frame.result.is_none());
    Err(Error::RecordNodes)
}

#[test]
fn mixed_prefix_append_is_transactional_and_stops_later_callbacks() -> Result<(), Error> {
    let list = Parent::List.build(vec![Value::Null; 64])?;
    let record = Parent::Record.build(vec![Value::Null; 64])?;
    // The rejected scalar is node 4097 after a 4096-node mixed prefix.
    for parent in [Parent::List, Parent::Record] {
        let mut calls = 0;
        let mut evaluate = |_: &JsonValue, _: &mut EvaluationContext| {
            calls += 1;
            Ok(if calls == 64 {
                Value::Null
            } else if calls % 2 == 0 {
                list.clone()
            } else {
                record.clone()
            })
        };
        let mut context = EvaluationContext::default();
        let result = match parent {
            Parent::List => list_expression_with_driver(
                &json!({"items": vec![JsonValue::Null; 64]}),
                "items",
                &mut context,
                &mut evaluate,
                list_prefix,
            ),
            Parent::Record => record_expression_with_driver(
                &json!({"items": (0..64).map(|i| json!({"name":i.to_string(),"value":null})).collect::<Vec<_>>()}),
                "items",
                &mut context,
                &mut evaluate,
                record_prefix,
            ),
        };
        parent.assert_error(result, ShapeError::Nodes);
        assert_eq!(calls, 64);
    }
    Ok(())
}

#[test]
fn cross_kind_depth_append_and_final_constructor_defense() -> Result<(), Error> {
    let list_child = alternating(Parent::List, 32)?;
    let record_child = alternating(Parent::Record, 32)?;
    let mut calls = 0;
    let result = list_expression_with_driver(
        &json!({"items":[null,null]}),
        "items",
        &mut EvaluationContext::default(),
        &mut |_, _| {
            calls += 1;
            Ok(record_child.clone())
        },
        |frame| {
            frame.first_item()?;
            frame.evaluate_item()?;
            let candidate = frame.candidate.clone();
            for _ in 0..2 {
                assert!(matches!(frame.append_item(), Err(Error::ListDepth)));
                assert_eq!(frame.candidate, candidate);
                assert_eq!(frame.shape, ValueShape::EMPTY);
                assert!(frame.buffer.is_empty());
                assert!(matches!(frame.stage, ListStage::Evaluated(0)));
            }
            // Bypass append internally to exercise the independent final check.
            frame.buffer = vec![candidate.expect("pending child"); 2];
            frame.stage = ListStage::Exhausted;
            frame.finish_list()
        },
    );
    assert!(matches!(result, Err(Error::ListDepth)));
    assert_eq!(calls, 1);
    calls = 0;
    let result = record_expression_with_driver(
        &json!({"items":[{"name":"x","value":null},{"name":"later","value":null}]}),
        "items",
        &mut EvaluationContext::default(),
        &mut |_, _| {
            calls += 1;
            Ok(list_child.clone())
        },
        |frame| {
            frame.first_item()?;
            frame.evaluate_item()?;
            let candidate = frame.candidate.clone();
            for _ in 0..2 {
                assert!(matches!(frame.append_item(), Err(Error::RecordDepth)));
                assert_eq!(frame.candidate, candidate);
                assert_eq!(frame.name.as_deref(), Some("x"));
                assert_eq!(frame.shape, ValueShape::EMPTY);
                assert!(frame.values.is_empty());
                assert!(matches!(frame.stage, RecordStage::Evaluated(0)));
            }
            frame.values = [
                ("x".into(), candidate.expect("pending child")),
                ("later".into(), Value::Null),
            ]
            .into();
            frame.stage = RecordStage::Exhausted;
            frame.finish_record()
        },
    );
    assert!(matches!(result, Err(Error::RecordDepth)));
    assert_eq!(calls, 1);
    Ok(())
}

#[test]
fn selecting_cross_kind_child_releases_parent_storage() -> Result<(), Error> {
    let child = ClosedRecord::new(vec![("x".into(), Value::Null)])?;
    let parent = ClosedList::new(vec![Value::Record(child.clone())])?;
    let weak = Arc::downgrade(&parent.values);
    let selected = list_at(&Value::List(parent), &Value::Integer(0))?;
    assert!(weak.upgrade().is_none());
    let Value::Record(selected) = selected else {
        panic!("record child")
    };
    assert!(Arc::ptr_eq(&selected.values, &child.values));
    assert_eq!(selected.shape, child.shape);
    let child = ClosedList::new(vec![Value::Null])?;
    let parent = ClosedRecord::new(vec![("x".into(), Value::List(child.clone()))])?;
    let weak = Arc::downgrade(&parent.values);
    let selected = record_get(&Value::Record(parent), &Value::Text(Arc::from("x")))?;
    assert!(weak.upgrade().is_none());
    let Value::List(selected) = selected else {
        panic!("list child")
    };
    assert!(Arc::ptr_eq(&selected.values, &child.values));
    assert_eq!(selected.shape, child.shape);
    Ok(())
}
