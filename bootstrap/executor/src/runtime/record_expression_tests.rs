use super::*;
use serde_json::json;

fn run(node: &JsonValue, driver: RecordDriver) -> Result<Value, Error> {
    record_expression_with_driver(
        node,
        "fields",
        &mut EvaluationContext::default(),
        &mut |_, context| context.visit(|_| Ok(Value::Int)),
        driver,
    )
}

#[test]
fn protocol_and_rejected_append_preserve_pending_state() -> Result<(), Error> {
    let node = json!({"fields": [{"name": "x", "value": {}}]});
    let result = run(&node, |frame| {
        assert!(matches!(
            frame.append_item(),
            Err(Error::RecordState { .. })
        ));
        assert!(matches!(
            frame.finish_record(),
            Err(Error::RecordState { .. })
        ));
        frame.first_item()?;
        assert!(matches!(frame.first_item(), Err(Error::RecordState { .. })));
        frame.evaluate_item()?;
        assert!(matches!(frame.next_item(), Err(Error::RecordState { .. })));
        assert!(matches!(
            frame.append_item(),
            Err(Error::RecordElement { .. })
        ));
        assert!(matches!(frame.stage, RecordStage::Evaluated(0)));
        assert_eq!(frame.name.as_deref(), Some("x"));
        assert_eq!(frame.candidate, Some(Value::Int));
        assert!(frame.values.is_empty());
        assert_eq!(frame.shape, ValueShape::EMPTY);
        frame.candidate = Some(Value::Integer(7));
        frame.values.insert("x".to_owned(), Value::Null);
        assert!(matches!(
            frame.append_item(),
            Err(Error::RecordDuplicate { .. })
        ));
        assert_eq!(frame.candidate, Some(Value::Integer(7)));
        assert_eq!(frame.name.as_deref(), Some("x"));
        frame.values.clear();
        frame.name = Some(String::new());
        assert!(matches!(
            frame.append_item(),
            Err(Error::RecordName { index: 0 })
        ));
        assert_eq!(frame.candidate, Some(Value::Integer(7)));
        frame.name = Some("x".to_owned());
        for index in 0..64 {
            frame.values.insert(index.to_string(), Value::Null);
        }
        assert!(matches!(frame.append_item(), Err(Error::RecordCapacity)));
        assert_eq!(frame.name.as_deref(), Some("x"));
        assert_eq!(frame.candidate, Some(Value::Integer(7)));
        frame.values.clear();
        frame.append_item()?;
        assert_eq!(frame.shape, ValueShape { depth: 2, nodes: 2 });
        assert!(frame.name.is_none());
        assert!(frame.candidate.is_none());
        frame.next_item()?;
        frame.finish_record()
    })?;
    let Value::Record(record) = result else {
        panic!("expected record");
    };
    assert_eq!(record.as_map().get("x"), Some(&Value::Integer(7)));
    Ok(())
}

#[test]
fn budget_is_exact_and_incomplete_results_never_escape() -> Result<(), Error> {
    let empty = json!({"fields": []});
    assert!(matches!(
        run(&empty, |_| Ok(())),
        Err(Error::RecordState { .. })
    ));
    run(&empty, |frame| {
        for _ in 0..4096 {
            frame.step()?;
        }
        assert!(matches!(frame.step(), Err(Error::RecordControlBudget)));
        frame.first_item()?;
        frame.finish_record()
    })?;
    let node = json!({"fields": [{"name": "x", "value": {}}]});
    assert!(matches!(
        run(&node, |frame| {
            frame.first_item()?;
            frame.evaluate_item()
        }),
        Err(Error::RecordState { .. })
    ));
    Ok(())
}

#[test]
fn finish_revalidates_the_opaque_constructor() {
    let node = json!({"fields": [{"name":"x","value":{}}]});
    let result = run(&node, |frame| {
        frame.stage = RecordStage::Exhausted;
        for (name, value) in [("x", Value::Int), ("", Value::Null)] {
            frame.values.insert(name.to_owned(), value);
            let error = frame.finish_record().expect_err("final invariant defense");
            assert!(matches!(
                error,
                Error::RecordElement { .. } | Error::RecordName { .. }
            ));
            assert!(frame.result.is_none());
            assert!(matches!(frame.stage, RecordStage::Exhausted));
        }
        Ok(())
    });
    assert!(matches!(result, Err(Error::RecordState { .. })));
}

#[test]
fn capacity_and_names_precede_callbacks_and_duplicate_child_lookup() -> Result<(), Error> {
    let oversized = json!({"fields": vec![json!({}); 65]});
    assert!(matches!(
        run(&oversized, |_| panic!("capacity must precede driver")),
        Err(Error::RecordCapacity)
    ));
    let mut calls = 0;
    let node = json!({"fields": [{"name": "x", "value": {}}, {"name": "x"}]});
    let result = record_expression_with_driver(
        &node,
        "fields",
        &mut EvaluationContext::default(),
        &mut |_, context| {
            calls += 1;
            context.visit(|_| Ok(Value::Null))
        },
        |frame| {
            frame.first_item()?;
            frame.evaluate_item()?;
            frame.append_item()?;
            frame.next_item()?;
            frame.evaluate_item()
        },
    );
    assert!(matches!(result, Err(Error::RecordDuplicate { .. })));
    assert_eq!(calls, 1);
    let invalid = json!({"fields": [{"name": ""}]});
    assert!(matches!(
        run(&invalid, |frame| {
            frame.first_item()?;
            frame.evaluate_item()
        }),
        Err(Error::RecordName { index: 0 })
    ));
    Ok(())
}
