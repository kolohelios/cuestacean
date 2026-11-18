use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage {
    Start,
    Operand,
    Lower,
    Upper,
    Done,
}

pub type Slicer = fn(Value, Value, Value) -> Result<Value, Error>;
pub type SliceDriver = fn(&mut SliceFrame<'_>) -> Result<(), Error>;

/// Checked optional-endpoint protocol; CUE owns traversal and defaults.
pub struct SliceFrame<'a> {
    children: [Option<&'a JsonValue>; 3],
    context: &'a mut EvaluationContext,
    evaluate: &'a mut dyn FnMut(&JsonValue, &mut EvaluationContext) -> Result<Value, Error>,
    slicer: Slicer,
    stage: Stage,
    values: [Option<Value>; 3],
    result: Option<Value>,
    fuel: usize,
}

impl SliceFrame<'_> {
    fn require(&self, stage: Stage) -> Result<(), Error> {
        if self.stage != stage {
            return Err(Error::SliceState {
                reason: "slice operation out of order",
            });
        }
        Ok(())
    }

    pub fn step(&mut self) -> Result<(), Error> {
        if self.fuel == 0 {
            return Err(Error::SliceControlBudget);
        }
        self.fuel -= 1;
        Ok(())
    }

    pub fn has_lower(&self) -> Result<bool, Error> {
        self.require(Stage::Operand)?;
        Ok(self.children[1].is_some())
    }

    pub fn has_upper(&self) -> Result<bool, Error> {
        self.require(Stage::Lower)?;
        Ok(self.children[2].is_some())
    }

    pub fn slice_kind_is(&self, kind: crate::unification::Kind) -> Result<bool, Error> {
        self.require(Stage::Lower)?;
        Ok(crate::unification::kind_is(self.value(0)?, kind))
    }

    fn value(&self, index: usize) -> Result<&Value, Error> {
        self.values[index].as_ref().ok_or(Error::SliceState {
            reason: "missing evaluated slice value",
        })
    }

    fn evaluate_child(&mut self, index: usize, before: Stage, after: Stage) -> Result<(), Error> {
        self.require(before)?;
        let child = self.children[index].ok_or(Error::SliceState {
            reason: "cannot evaluate absent slice child",
        })?;
        let value = (self.evaluate)(child, self.context)?;
        self.values[index] = Some(value);
        self.stage = after;
        Ok(())
    }

    pub fn evaluate_slice_operand(&mut self) -> Result<(), Error> {
        self.evaluate_child(0, Stage::Start, Stage::Operand)
    }

    pub fn evaluate_slice_lower(&mut self) -> Result<(), Error> {
        self.evaluate_child(1, Stage::Operand, Stage::Lower)
    }

    pub fn evaluate_slice_upper(&mut self) -> Result<(), Error> {
        self.evaluate_child(2, Stage::Lower, Stage::Upper)
    }

    pub fn default_slice_lower(&mut self, value: i64) -> Result<(), Error> {
        self.require(Stage::Operand)?;
        if self.children[1].is_some() {
            return Err(Error::SliceState {
                reason: "cannot default present slice lower",
            });
        }
        self.values[1] = Some(Value::Integer(value));
        self.stage = Stage::Lower;
        Ok(())
    }

    pub fn default_slice_upper(&mut self) -> Result<(), Error> {
        self.require(Stage::Lower)?;
        if self.children[2].is_some() {
            return Err(Error::SliceState {
                reason: "cannot default present slice upper",
            });
        }
        let upper = list_count(self.value(0)?)?;
        self.values[2] = Some(upper);
        self.stage = Stage::Upper;
        Ok(())
    }

    pub fn reject_slice_operand(&self) -> Result<(), Error> {
        self.require(Stage::Lower)?;
        Err(Error::ListSliceOperands)
    }

    pub fn finish_slice(&mut self) -> Result<(), Error> {
        self.require(Stage::Upper)?;
        // A failing selected policy must not consume any original handle.
        let result = (self.slicer)(
            self.value(0)?.clone(),
            self.value(1)?.clone(),
            self.value(2)?.clone(),
        )?;
        self.result = Some(result);
        self.values = [None, None, None];
        self.stage = Stage::Done;
        Ok(())
    }
}

/// Run selected control without charging a parent AST entry.
pub fn slice_with_driver(
    node: &JsonValue,
    names: [&str; 3],
    context: &mut EvaluationContext,
    evaluate: &mut dyn FnMut(&JsonValue, &mut EvaluationContext) -> Result<Value, Error>,
    slicer: Slicer,
    driver: SliceDriver,
) -> Result<Value, Error> {
    run(node, names, context, evaluate, slicer, driver)
}

fn run(
    node: &JsonValue,
    names: [&str; 3],
    context: &mut EvaluationContext,
    evaluate: &mut dyn FnMut(&JsonValue, &mut EvaluationContext) -> Result<Value, Error>,
    slicer: Slicer,
    driver: impl FnOnce(&mut SliceFrame<'_>) -> Result<(), Error>,
) -> Result<Value, Error> {
    let mut frame = SliceFrame {
        children: [
            Some(field(node, names[0])?),
            node.get(names[1]),
            node.get(names[2]),
        ],
        context,
        evaluate,
        slicer,
        stage: Stage::Start,
        values: [None, None, None],
        result: None,
        fuel: 4096,
    };
    driver(&mut frame)?;
    frame.require(Stage::Done)?;
    frame.result.ok_or(Error::SliceState {
        reason: "missing slice result",
    })
}

#[cfg(feature = "semantics")]
#[path = "../../../generated/slice-control.rs"]
mod generated_slice_control;

#[cfg(feature = "semantics")]
pub fn slice(
    node: &JsonValue,
    names: [&str; 3],
    context: &mut EvaluationContext,
    mut evaluate: impl FnMut(&JsonValue, &mut EvaluationContext) -> Result<Value, Error>,
) -> Result<Value, Error> {
    run(
        node,
        names,
        context,
        &mut evaluate,
        super::list_slice,
        generated_slice_control::execute,
    )
}

/// Group CUE-selected field names for the generated evaluator's ordinary call.
/// No evaluation/default/range policy lives in this adapter.
#[cfg(feature = "semantics")]
pub fn slice_fields(
    node: &JsonValue,
    operand: &str,
    lower: &str,
    upper: &str,
    context: &mut EvaluationContext,
    mut evaluate: impl FnMut(&JsonValue, &mut EvaluationContext) -> Result<Value, Error>,
) -> Result<Value, Error> {
    run(
        node,
        [operand, lower, upper],
        context,
        &mut evaluate,
        super::list_slice,
        generated_slice_control::execute,
    )
}

#[cfg(feature = "semantics")]
pub fn slice_interpreted(
    node: &JsonValue,
    names: [&str; 3],
    context: &mut EvaluationContext,
    mut evaluate: impl FnMut(&JsonValue, &mut EvaluationContext) -> Result<Value, Error>,
    program: &str,
) -> Result<Value, Error> {
    run(
        node,
        names,
        context,
        &mut evaluate,
        super::list_slice,
        |frame| crate::pratt::interpret_slice(program, frame),
    )
}

#[cfg(test)]
#[path = "optional_slice_tests.rs"]
mod tests;
