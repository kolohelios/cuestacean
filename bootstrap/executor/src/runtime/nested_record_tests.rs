use super::*;
use serde_json::json;

fn fields(count: usize, value: Value) -> Vec<(String, Value)> {
    (0..count)
        .map(|index| (index.to_string(), value.clone()))
        .collect()
}

fn chain(depth: usize) -> Result<Value, Error> {
    let mut value = Value::Record(ClosedRecord::new(vec![])?);
    for _ in 1..depth {
        value = Value::Record(ClosedRecord::new(vec![("x".into(), value)])?);
    }
    Ok(value)
}

#[test]
fn certified_depth_and_occurrence_weighted_nodes() -> Result<(), Error> {
    let empty = ClosedRecord::new(vec![])?;
    assert_eq!(empty.shape, RecordShape { depth: 1, nodes: 1 });
    let scalar = ClosedRecord::new(fields(1, Value::Null))?;
    assert_eq!(scalar.shape, RecordShape { depth: 2, nodes: 2 });
    let deepest = chain(32)?;
    assert!(matches!(
        ClosedRecord::new(fields(1, deepest)),
        Err(Error::RecordDepth)
    ));
    let child = ClosedRecord::new(fields(64, Value::Null))?;
    assert_eq!(
        child.shape,
        RecordShape {
            depth: 2,
            nodes: 65
        }
    );
    let parent = ClosedRecord::new(fields(63, Value::Record(child.clone())))?;
    assert_eq!(
        parent.shape,
        RecordShape {
            depth: 3,
            nodes: 4096
        }
    );
    let mut too_many = fields(63, Value::Record(child.clone()));
    too_many.push(("last".into(), Value::Null));
    assert!(matches!(
        ClosedRecord::new(too_many),
        Err(Error::RecordNodes)
    ));
    let left = parent.as_map().get("0").expect("first child");
    let right = parent.as_map().get("1").expect("repeated child");
    let (Value::Record(left), Value::Record(right)) = (left, right) else {
        panic!("expected shared records");
    };
    assert!(Arc::ptr_eq(&left.values, &right.values));
    assert_eq!(left.shape, child.shape);
    let selected = record_get(&Value::Record(parent.clone()), &Value::Text(Arc::from("0")))?;
    drop(parent);
    let Value::Record(selected) = selected else {
        panic!("expected selected record")
    };
    assert!(Arc::ptr_eq(&selected.values, &child.values));
    assert_eq!(selected.len(), 64);
    Ok(())
}

#[test]
fn constructor_precedence_and_unchanged_flat_codec() -> Result<(), Error> {
    let deep = chain(32)?;
    assert!(matches!(
        ClosedRecord::new(fields(65, deep.clone())),
        Err(Error::RecordCapacity)
    ));
    assert!(matches!(
        ClosedRecord::new(vec![("".into(), deep.clone())]),
        Err(Error::RecordName { index: 0 })
    ));
    assert!(matches!(
        ClosedRecord::new(vec![("a".into(), Value::Null), ("a".into(), deep.clone())]),
        Err(Error::RecordDuplicate { .. })
    ));
    let child = Value::Record(ClosedRecord::new(fields(64, Value::Null))?);
    let full = fields(63, child);
    for (last, expected) in [
        (Value::Top, "type"),
        (deep, "depth"),
        (Value::Null, "nodes"),
    ] {
        let mut entries = full.clone();
        entries.push(("last".into(), last));
        let error = ClosedRecord::new(entries).expect_err("prospective shape rejection");
        assert!(matches!(
            (expected, error),
            ("type", Error::RecordElement { .. })
                | ("depth", Error::RecordDepth)
                | ("nodes", Error::RecordNodes)
        ));
    }
    for value in [json!({}), json!({"fields": []}), json!([])] {
        assert!(matches!(
            record(&json!({"fields":[{"name":"x","value":value}]}), "fields"),
            Err(Error::RecordElement { .. })
        ));
    }
    let Value::Record(decoded) = record(&json!({"fields":[{"name":"x","value":null}]}), "fields")?
    else {
        panic!("expected decoded record");
    };
    assert_eq!(decoded.shape, RecordShape { depth: 2, nodes: 2 });
    Ok(())
}

#[test]
fn shape_failure_is_transactional_and_precedes_later_callbacks() -> Result<(), Error> {
    let node = json!({"fields":[{"name":"x","value":{}},{"name":"later","value":{}}]});
    let mut calls = 0;
    let result = record_expression_with_driver(
        &node,
        "fields",
        &mut EvaluationContext::default(),
        &mut |_, _| {
            calls += 1;
            chain(32)
        },
        |frame| {
            frame.first_item()?;
            frame.evaluate_item()?;
            let candidate = frame.candidate.clone();
            let name = frame.name.clone();
            let shape = frame.shape;
            assert!(matches!(frame.append_item(), Err(Error::RecordDepth)));
            assert_eq!(frame.candidate, candidate);
            assert_eq!(frame.name, name);
            assert_eq!(frame.shape, shape);
            assert!(frame.values.is_empty());
            assert!(matches!(frame.stage, RecordStage::Evaluated(0)));
            frame.append_item()
        },
    );
    assert!(matches!(result, Err(Error::RecordDepth)));
    assert_eq!(calls, 1);
    Ok(())
}

#[test]
fn node_failure_retains_map_and_pending_slots() -> Result<(), Error> {
    let node = json!({"fields":[{"name":"last","value":{}}]});
    let result = record_expression_with_driver(
        &node,
        "fields",
        &mut EvaluationContext::default(),
        &mut |_, _| Ok(Value::Null),
        |frame| {
            frame.first_item()?;
            frame.evaluate_item()?;
            let child = Value::Record(ClosedRecord::new(fields(64, Value::Null))?);
            for (name, value) in fields(63, child) {
                frame.shape = frame.shape.include(&value, &name)?;
                frame.values.insert(name, value);
            }
            let values = frame.values.clone();
            let shape = frame.shape;
            assert!(matches!(frame.append_item(), Err(Error::RecordNodes)));
            assert_eq!(frame.values, values);
            assert_eq!(frame.shape, shape);
            assert_eq!(frame.name.as_deref(), Some("last"));
            assert_eq!(frame.candidate, Some(Value::Null));
            assert!(matches!(frame.stage, RecordStage::Evaluated(0)));
            frame.append_item()
        },
    );
    assert!(matches!(result, Err(Error::RecordNodes)));
    Ok(())
}

#[test]
fn finish_independently_revalidates_shape() -> Result<(), Error> {
    for count in [1, 64] {
        let node = json!({"fields": vec![json!({}); count]});
        let result = record_expression_with_driver(
            &node,
            "fields",
            &mut EvaluationContext::default(),
            &mut |_, _| Ok(Value::Null),
            |frame| {
                let depth_error = frame.items.len() == 1;
                let entries = if depth_error {
                    fields(1, chain(32)?)
                } else {
                    let child = Value::Record(ClosedRecord::new(fields(64, Value::Null))?);
                    let mut entries = fields(63, child);
                    entries.push(("last".into(), Value::Null));
                    entries
                };
                frame.values = entries.into_iter().collect();
                frame.stage = RecordStage::Exhausted;
                assert_eq!(frame.shape, RecordShape::EMPTY);
                let error = frame
                    .finish_record()
                    .expect_err("constructor must ignore frame certificate");
                assert!(matches!(
                    (depth_error, error),
                    (true, Error::RecordDepth) | (false, Error::RecordNodes)
                ));
                assert!(frame.result.is_none());
                assert!(matches!(frame.stage, RecordStage::Exhausted));
                Ok(())
            },
        );
        assert!(matches!(result, Err(Error::RecordState { .. })));
    }
    Ok(())
}
