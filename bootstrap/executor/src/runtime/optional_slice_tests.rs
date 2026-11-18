use super::*;
use serde_json::json;

const NAMES: [&str; 3] = ["operand", "lower", "upper"];

// Test-only selected traversal and slicer; neither requires generated semantics.
fn selected(frame: &mut SliceFrame<'_>) -> Result<(), Error> {
    frame.evaluate_slice_operand()?;
    if frame.has_lower()? {
        frame.evaluate_slice_lower()?;
    } else {
        frame.default_slice_lower(7)?;
    }
    if frame.has_upper()? {
        frame.evaluate_slice_upper()?;
    } else {
        frame.default_slice_upper()?;
    }
    frame.finish_slice()
}

fn take_operand(operand: Value, _: Value, _: Value) -> Result<Value, Error> {
    Ok(operand)
}

#[test]
fn protocol_rejects_wrong_stages_and_present_defaults() -> Result<(), Error> {
    slice_with_driver(
        &json!({"operand": null, "lower": null, "upper": null}),
        NAMES,
        &mut EvaluationContext::default(),
        &mut |_, _| Ok(Value::Null),
        take_operand,
        |frame| {
            assert!(frame.has_lower().is_err());
            assert!(frame.has_upper().is_err());
            assert!(frame.evaluate_slice_lower().is_err());
            assert!(frame.evaluate_slice_upper().is_err());
            assert!(frame.default_slice_lower(0).is_err());
            assert!(frame.default_slice_upper().is_err());
            assert!(frame.reject_slice_operand().is_err());
            assert!(frame.finish_slice().is_err());
            frame.evaluate_slice_operand()?;
            assert!(frame.has_lower()?);
            assert!(frame.evaluate_slice_operand().is_err());
            assert!(frame.default_slice_lower(0).is_err());
            frame.evaluate_slice_lower()?;
            assert!(frame.has_upper()?);
            assert!(frame.has_lower().is_err());
            assert!(frame.slice_kind_is(crate::unification::Kind::Null)?);
            assert!(matches!(
                frame.reject_slice_operand(),
                Err(Error::ListSliceOperands)
            ));
            assert!(frame.default_slice_upper().is_err());
            frame.evaluate_slice_upper()?;
            assert!(frame.slice_kind_is(crate::unification::Kind::Null).is_err());
            frame.finish_slice()?;
            assert!(frame.finish_slice().is_err());
            assert!(frame.evaluate_slice_operand().is_err());
            Ok(())
        },
    )?;
    Ok(())
}

#[test]
fn presence_is_not_null_and_defaults_do_not_enter_ast_context() -> Result<(), Error> {
    for lower in [false, true] {
        for upper in [false, true] {
            let mut node = json!({"source": null});
            if lower {
                node["lo"] = json!(null);
            }
            if upper {
                node["hi"] = json!(null);
            }
            let mut context = EvaluationContext::default();
            let mut calls = 0;
            run(
                &node,
                ["source", "lo", "hi"],
                &mut context,
                &mut |child, context| {
                    assert!(child.is_null());
                    calls += 1;
                    context.visit(|_| Ok(Value::List(ClosedList::new(vec![])?)))
                },
                take_operand,
                |frame| {
                    selected(frame)?;
                    assert!(frame.values.iter().all(Option::is_none));
                    Ok(())
                },
            )?;
            let expected = 1 + usize::from(lower) + usize::from(upper);
            assert_eq!(calls, expected);
            assert_eq!(context.remaining, 4096 - expected);
            assert_eq!(context.depth, 0);
        }
    }
    Ok(())
}

#[test]
fn injected_slicer_receives_cue_literal_and_checked_list_count() -> Result<(), Error> {
    let operand = Value::List(ClosedList::new(vec![
        Value::Null,
        Value::Integer(i64::MAX),
    ])?);
    let result = slice_with_driver(
        &json!({"operand": null}),
        NAMES,
        &mut EvaluationContext::default(),
        &mut |_, _| Ok(operand.clone()),
        |operand, lower, upper| {
            assert_eq!(lower, Value::Integer(7));
            assert_eq!(upper, Value::Integer(2));
            Ok(operand)
        },
        selected,
    )?;
    assert_eq!(result, operand);
    Ok(())
}

#[test]
fn absent_evaluation_and_failed_count_preserve_stage() -> Result<(), Error> {
    run(
        &json!({"operand": null}),
        NAMES,
        &mut EvaluationContext::default(),
        &mut |_, _| Ok(Value::Null),
        take_operand,
        |frame| {
            frame.evaluate_slice_operand()?;
            assert!(frame.evaluate_slice_lower().is_err());
            frame.default_slice_lower(19)?;
            assert_eq!(frame.values[1], Some(Value::Integer(19)));
            assert!(frame.evaluate_slice_upper().is_err());
            assert!(matches!(
                frame.default_slice_upper(),
                Err(Error::ListLength)
            ));
            assert!(frame.stage == Stage::Lower);
            assert!(frame.values[2].is_none());
            Err(Error::ListSliceOperands)
        },
    )
    .expect_err("invalid operand must not finish");
    Ok(())
}

#[test]
fn failed_finish_retains_all_handles_and_can_retry() -> Result<(), Error> {
    let text: Arc<str> = Arc::from("retained");
    let value = Value::Text(text.clone());
    let result = run(
        &json!({"operand": null, "lower": null, "upper": null}),
        NAMES,
        &mut EvaluationContext::default(),
        &mut |_, _| Ok(value.clone()),
        |_, _, _| Err(Error::ListSliceRange),
        |frame| {
            frame.evaluate_slice_operand()?;
            frame.evaluate_slice_lower()?;
            frame.evaluate_slice_upper()?;
            assert!(matches!(frame.finish_slice(), Err(Error::ListSliceRange)));
            assert!(frame.stage == Stage::Upper);
            assert!(frame.result.is_none());
            assert!(
                frame
                    .values
                    .iter()
                    .all(|slot| slot.as_ref() == Some(&value))
            );
            frame.slicer = take_operand;
            frame.finish_slice()
        },
    )?;
    drop(value);
    assert_eq!(result, Value::Text(text.clone()));
    assert_eq!(Arc::strong_count(&text), 2);
    Ok(())
}

#[test]
fn callback_errors_are_retryable_and_keep_shared_context() -> Result<(), Error> {
    let mut context = EvaluationContext::default();
    let mut calls = 0;
    let result = run(
        &json!({"operand": null}),
        NAMES,
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
        take_operand,
        |frame| {
            assert!(matches!(
                frame.evaluate_slice_operand(),
                Err(Error::TypeConflict)
            ));
            assert!(frame.stage == Stage::Start);
            assert!(frame.values[0].is_none());
            frame.evaluate_slice_operand()?;
            frame.default_slice_lower(0)?;
            frame.reject_slice_operand()
        },
    );
    assert!(matches!(result, Err(Error::ListSliceOperands)));
    assert_eq!(calls, 2);
    assert_eq!(context.remaining, 4094);
    assert_eq!(context.depth, 0);
    Ok(())
}

#[cfg(feature = "semantics")]
#[path = "../../../generated/slice-control-tail.rs"]
mod tail_control;

#[cfg(feature = "semantics")]
#[test]
fn compiled_and_reference_charge_presence_defaults_and_tail_return() -> Result<(), Error> {
    fn reference(frame: &mut SliceFrame<'_>) -> Result<(), Error> {
        crate::pratt::interpret_slice(
            include_str!(concat!(
                env!("CUESTACEAN_BOOTSTRAP_JSON"),
                "/slice-control.json"
            )),
            frame,
        )
    }
    fn reference_tail(frame: &mut SliceFrame<'_>) -> Result<(), Error> {
        crate::pratt::interpret_slice(
            include_str!(concat!(
                env!("CUESTACEAN_BOOTSTRAP_JSON"),
                "/tests/valid/slice-control-tail.json"
            )),
            frame,
        )
    }
    for (node, base) in [
        (json!({"operand":[]}), 7),
        (json!({"operand":[],"lower":0}), 7),
        (json!({"operand":[],"upper":0}), 6),
        (json!({"operand":[],"lower":0,"upper":0}), 6),
    ] {
        for (driver, extra) in [
            (generated_slice_control::execute as SliceDriver, 0),
            (reference as SliceDriver, 0),
            (tail_control::execute as SliceDriver, 1),
            (reference_tail as SliceDriver, 1),
        ] {
            for enough in [true, false] {
                let result = run(
                    &node,
                    NAMES,
                    &mut EvaluationContext::default(),
                    &mut |child, _| {
                        if child.is_array() {
                            ClosedList::new(vec![]).map(Value::List)
                        } else {
                            Ok(Value::Integer(0))
                        }
                    },
                    super::super::list_slice,
                    |frame| {
                        frame.fuel = base + extra - usize::from(!enough);
                        let result = driver(frame);
                        assert_eq!(frame.fuel, 0);
                        assert_eq!(frame.stage == Stage::Done, enough || extra == 1);
                        assert_eq!(frame.result.is_some(), enough || extra == 1);
                        result
                    },
                );
                if enough {
                    assert!(matches!(result, Ok(Value::List(_))));
                } else {
                    assert!(matches!(result, Err(Error::SliceControlBudget)));
                }
            }
        }
    }
    Ok(())
}

#[test]
fn fuel_is_exact_and_partial_success_cannot_publish() -> Result<(), Error> {
    let node = json!({"operand": null, "lower": null, "upper": null});
    for partial in [false, true] {
        let result = slice_with_driver(
            &node,
            NAMES,
            &mut EvaluationContext::default(),
            &mut |_, _| Ok(Value::Null),
            take_operand,
            if partial {
                |frame| frame.evaluate_slice_operand()
            } else {
                |_| Ok(())
            },
        );
        assert!(matches!(result, Err(Error::SliceState { .. })));
    }
    slice_with_driver(
        &node,
        NAMES,
        &mut EvaluationContext::default(),
        &mut |_, _| Ok(Value::Null),
        take_operand,
        |frame| {
            for _ in 0..4096 {
                frame.step()?;
            }
            assert!(matches!(frame.step(), Err(Error::SliceControlBudget)));
            selected(frame)
        },
    )?;
    Ok(())
}
