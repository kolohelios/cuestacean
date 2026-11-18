use super::*;
use serde_json::json;

fn forward(frame: &mut ConjunctionFrame<'_>) -> Result<(), Error> {
    frame.seed_top()?;
    frame.first_item()?;
    while frame.has_item()? {
        frame.evaluate_item()?;
        frame.unify_item()?;
        frame.next_item()?;
    }
    frame.finish_conjunction()
}

// Test-only atomic kernel: no dependency on generated semantics.
fn take_right(_: Value, right: Value) -> Result<Value, Error> {
    Ok(right)
}

fn check(node: &JsonValue, driver: ConjunctionDriver) -> Result<Value, Error> {
    conjunction_with_driver(
        node,
        "operands",
        &mut EvaluationContext::default(),
        &mut |_, context| context.visit(|_| Ok(Value::Null)),
        take_right,
        driver,
    )
}

#[test]
fn protocol_requires_seed_evaluate_merge_move_and_exact_completion() -> Result<(), Error> {
    check(&json!({"operands": [null, null]}), |frame| {
        assert!(frame.first_item().is_err());
        assert!(frame.has_item().is_err());
        assert!(frame.unify_item().is_err());
        assert!(frame.finish_conjunction().is_err());
        frame.seed_top()?;
        assert!(frame.seed_top().is_err());
        frame.first_item()?;
        assert!(frame.last_item().is_err());
        assert!(frame.next_item().is_err());
        frame.evaluate_item()?;
        assert!(frame.evaluate_item().is_err());
        assert!(frame.previous_item().is_err());
        frame.unify_item()?;
        assert!(frame.unify_item().is_err());
        frame.next_item()?;
        frame.evaluate_item()?;
        frame.unify_item()?;
        frame.next_item()?;
        frame.merges = 1;
        assert!(frame.finish_conjunction().is_err());
        frame.merges = 2;
        frame.finish_conjunction()?;
        assert!(frame.finish_conjunction().is_err());
        assert!(frame.has_item().is_err());
        Ok(())
    })?;
    Ok(())
}

#[test]
fn failed_merge_preserves_owned_candidate_accumulator_and_stage() -> Result<(), Error> {
    let text: Arc<str> = Arc::from("retained");
    let child = Value::Text(text.clone());
    let result = run(
        &json!({"operands": [null]}),
        "operands",
        &mut EvaluationContext::default(),
        &mut |_, _| Ok(child.clone()),
        |_, _| Err(Error::TypeConflict),
        |frame| {
            frame.seed_top()?;
            frame.last_item()?;
            frame.evaluate_item()?;
            assert!(matches!(frame.unify_item(), Err(Error::TypeConflict)));
            assert_eq!(frame.accumulator, Some(Value::Top));
            assert_eq!(frame.candidate, Some(child.clone()));
            assert!(matches!(frame.stage, Stage::Evaluated(0)));
            assert_eq!(frame.merges, 0);
            assert!(frame.finish_conjunction().is_err());
            frame.unify = take_right;
            frame.unify_item()?;
            assert!(frame.candidate.is_none());
            frame.previous_item()?;
            frame.finish_conjunction()
        },
    )?;
    drop(child);
    assert_eq!(result, Value::Text(text.clone()));
    assert_eq!(Arc::strong_count(&text), 2);
    Ok(())
}

#[test]
fn fuel_is_exact_and_early_returns_cannot_publish_partial_results() -> Result<(), Error> {
    let empty = json!({"operands": []});
    assert!(matches!(
        check(&empty, |_| Ok(())),
        Err(Error::ConjunctionState { .. })
    ));
    assert_eq!(
        check(&empty, |frame| {
            for _ in 0..4096 {
                frame.step()?;
            }
            assert!(matches!(frame.step(), Err(Error::ConjunctionControlBudget)));
            forward(frame)
        })?,
        Value::Top
    );
    assert!(matches!(
        check(&json!({"operands": [null]}), |frame| {
            frame.seed_top()?;
            frame.first_item()?;
            frame.evaluate_item()?;
            frame.unify_item()
        }),
        Err(Error::ConjunctionState { .. })
    ));
    Ok(())
}

#[cfg(feature = "semantics")]
#[path = "../../../generated/conjunction-control-tail.rs"]
mod tail_control;

#[cfg(feature = "semantics")]
#[test]
fn generated_and_reference_charge_loop_completion_and_tail_return() -> Result<(), Error> {
    fn reference(frame: &mut ConjunctionFrame<'_>) -> Result<(), Error> {
        crate::pratt::interpret_conjunction(
            include_str!(concat!(
                env!("CUESTACEAN_BOOTSTRAP_JSON"),
                "/conjunction-control.json"
            )),
            frame,
        )
    }
    fn reference_tail(frame: &mut ConjunctionFrame<'_>) -> Result<(), Error> {
        crate::pratt::interpret_conjunction(
            include_str!(concat!(
                env!("CUESTACEAN_BOOTSTRAP_JSON"),
                "/tests/valid/conjunction-control-tail.json"
            )),
            frame,
        )
    }
    let empty = json!({"operands": []});
    for (driver, required) in [
        (
            generated_conjunction_control::execute as ConjunctionDriver,
            4,
        ),
        (reference as ConjunctionDriver, 4),
        (tail_control::execute as ConjunctionDriver, 5),
        (reference_tail as ConjunctionDriver, 5),
    ] {
        let result = run(
            &empty,
            "operands",
            &mut EvaluationContext::default(),
            &mut |_, _| panic!("empty fold has no children"),
            take_right,
            |frame| {
                frame.fuel = required;
                driver(frame)?;
                assert_eq!(frame.fuel, 0);
                assert!(matches!(frame.stage, Stage::Done));
                Ok(())
            },
        )?;
        assert_eq!(result, Value::Top);
        let result = run(
            &empty,
            "operands",
            &mut EvaluationContext::default(),
            &mut |_, _| panic!("empty fold has no children"),
            take_right,
            |frame| {
                frame.fuel = required - 1;
                let result = driver(frame);
                assert!(matches!(result, Err(Error::ConjunctionControlBudget)));
                assert_eq!(frame.fuel, 0);
                assert!(frame.accumulator.is_some());
                assert_eq!(matches!(frame.stage, Stage::Done), required == 5);
                result
            },
        );
        assert!(matches!(result, Err(Error::ConjunctionControlBudget)));
    }
    Ok(())
}

#[test]
fn capacity_precedes_driver_and_callbacks_and_64_merges_complete() -> Result<(), Error> {
    assert!(matches!(
        check(&json!({"operands": vec![json!(null); 65]}), |_| {
            panic!("capacity must precede driver")
        }),
        Err(Error::ConjunctionCapacity)
    ));
    assert_eq!(
        check(&json!({"operands": vec![json!(null); 64]}), |frame| {
            forward(frame)?;
            assert_eq!(frame.merges, 64);
            Ok(())
        })?,
        Value::Null
    );
    Ok(())
}

#[test]
fn selected_callback_owns_shared_ast_entries_and_failure_is_retryable() -> Result<(), Error> {
    let mut context = EvaluationContext::default();
    let mut calls = 0;
    let result = conjunction_with_driver(
        &json!({"operands": [null]}),
        "operands",
        &mut context,
        &mut |_, context| {
            calls += 1;
            context.visit(|_| {
                if calls == 1 {
                    Err(Error::TypeConflict)
                } else {
                    Ok(Value::Null)
                }
            })
        },
        take_right,
        |frame| {
            frame.seed_top()?;
            frame.first_item()?;
            assert!(matches!(frame.evaluate_item(), Err(Error::TypeConflict)));
            assert!(matches!(frame.stage, Stage::Ready(0)));
            assert!(frame.candidate.is_none());
            frame.evaluate_item()?;
            frame.unify_item()?;
            frame.next_item()?;
            frame.finish_conjunction()
        },
    )?;
    assert_eq!(result, Value::Null);
    assert_eq!(calls, 2);
    assert_eq!(context.remaining, 4094);
    assert_eq!(context.depth, 0);
    Ok(())
}
