use super::*;

#[derive(Clone, Copy)]
enum Stage {
    Start,
    Seeded,
    Ready(usize),
    Evaluated(usize),
    Unified(usize),
    Exhausted,
    Done,
}

pub type Unifier = fn(Value, Value) -> Result<Value, Error>;
pub type ConjunctionDriver = fn(&mut ConjunctionFrame<'_>) -> Result<(), Error>;

/// Checked fold protocol; CUE owns seeding, traversal and merge selection.
pub struct ConjunctionFrame<'a> {
    items: &'a [JsonValue],
    context: &'a mut EvaluationContext,
    evaluate: &'a mut dyn FnMut(&JsonValue, &mut EvaluationContext) -> Result<Value, Error>,
    unify: Unifier,
    stage: Stage,
    candidate: Option<Value>,
    accumulator: Option<Value>,
    merges: usize,
    fuel: usize,
}

impl ConjunctionFrame<'_> {
    fn invalid<T>(reason: &'static str) -> Result<T, Error> {
        Err(Error::ConjunctionState { reason })
    }

    pub fn step(&mut self) -> Result<(), Error> {
        if self.fuel == 0 {
            return Err(Error::ConjunctionControlBudget);
        }
        self.fuel -= 1;
        Ok(())
    }

    pub fn seed_top(&mut self) -> Result<(), Error> {
        if !matches!(self.stage, Stage::Start) {
            return Self::invalid("conjunction already seeded");
        }
        self.accumulator = Some(Value::Top);
        self.stage = Stage::Seeded;
        Ok(())
    }

    fn select(&mut self, index: Option<usize>) -> Result<(), Error> {
        if !matches!(self.stage, Stage::Seeded) {
            return Self::invalid("cursor selection requires seeded conjunction");
        }
        self.stage = index.map_or(Stage::Exhausted, Stage::Ready);
        Ok(())
    }

    pub fn first_item(&mut self) -> Result<(), Error> {
        self.select(if self.items.is_empty() { None } else { Some(0) })
    }

    pub fn last_item(&mut self) -> Result<(), Error> {
        self.select(self.items.len().checked_sub(1))
    }

    pub fn has_item(&self) -> Result<bool, Error> {
        match self.stage {
            Stage::Ready(_) | Stage::Evaluated(_) | Stage::Unified(_) => Ok(true),
            Stage::Exhausted => Ok(false),
            _ => Self::invalid("conjunction cursor is not active"),
        }
    }

    pub fn evaluate_item(&mut self) -> Result<(), Error> {
        let Stage::Ready(index) = self.stage else {
            return Self::invalid("evaluation requires ready cursor");
        };
        let item = self.items.get(index).ok_or(Error::ConjunctionState {
            reason: "item index out of bounds",
        })?;
        self.candidate = Some((self.evaluate)(item, self.context)?);
        self.stage = Stage::Evaluated(index);
        Ok(())
    }

    pub fn item_kind_is(&self, kind: crate::unification::Kind) -> Result<bool, Error> {
        if !matches!(self.stage, Stage::Evaluated(_)) {
            return Self::invalid("item kind requires evaluated candidate");
        }
        self.candidate
            .as_ref()
            .map(|value| crate::unification::kind_is(value, kind))
            .ok_or(Error::ConjunctionState {
                reason: "missing item candidate",
            })
    }

    pub fn unify_item(&mut self) -> Result<(), Error> {
        let Stage::Evaluated(index) = self.stage else {
            return Self::invalid("merge requires evaluated candidate");
        };
        if self.merges >= 64 || self.merges >= self.items.len() {
            return Self::invalid("too many conjunction merges");
        }
        let accumulator = self.accumulator.as_ref().ok_or(Error::ConjunctionState {
            reason: "missing conjunction accumulator",
        })?;
        let candidate = self.candidate.as_ref().ok_or(Error::ConjunctionState {
            reason: "missing item candidate",
        })?;
        // Owned clones make a failed atomic merge leave both original handles intact.
        let merged = (self.unify)(accumulator.clone(), candidate.clone())?;
        self.accumulator = Some(merged);
        self.candidate = None;
        self.merges += 1;
        self.stage = Stage::Unified(index);
        Ok(())
    }

    fn advance(&mut self, forward: bool) -> Result<(), Error> {
        let Stage::Unified(index) = self.stage else {
            return Self::invalid("movement requires successful merge");
        };
        let next = if forward {
            index
                .checked_add(1)
                .filter(|index| *index < self.items.len())
        } else {
            index.checked_sub(1)
        };
        self.stage = next.map_or(Stage::Exhausted, Stage::Ready);
        Ok(())
    }

    pub fn next_item(&mut self) -> Result<(), Error> {
        self.advance(true)
    }

    pub fn previous_item(&mut self) -> Result<(), Error> {
        self.advance(false)
    }

    pub fn finish_conjunction(&mut self) -> Result<(), Error> {
        if !matches!(self.stage, Stage::Exhausted)
            || self.merges != self.items.len()
            || self.candidate.is_some()
            || self.accumulator.is_none()
        {
            return Self::invalid("finish requires exhausted cursor and exact merge count");
        }
        self.stage = Stage::Done;
        Ok(())
    }
}

/// Run trusted control without a parent AST charge; callbacks own child entry.
pub fn conjunction_with_driver(
    node: &JsonValue,
    name: &str,
    context: &mut EvaluationContext,
    evaluate: &mut dyn FnMut(&JsonValue, &mut EvaluationContext) -> Result<Value, Error>,
    unify: Unifier,
    driver: ConjunctionDriver,
) -> Result<Value, Error> {
    run(node, name, context, evaluate, unify, driver)
}

fn run(
    node: &JsonValue,
    name: &str,
    context: &mut EvaluationContext,
    evaluate: &mut dyn FnMut(&JsonValue, &mut EvaluationContext) -> Result<Value, Error>,
    unify: Unifier,
    driver: impl FnOnce(&mut ConjunctionFrame<'_>) -> Result<(), Error>,
) -> Result<Value, Error> {
    let items = field(node, name)?.as_array().ok_or_else(|| Error::Array {
        field: name.to_owned(),
    })?;
    if items.len() > 64 {
        return Err(Error::ConjunctionCapacity);
    }
    let mut frame = ConjunctionFrame {
        items,
        context,
        evaluate,
        unify,
        stage: Stage::Start,
        candidate: None,
        accumulator: None,
        merges: 0,
        fuel: 4096,
    };
    driver(&mut frame)?;
    if !matches!(frame.stage, Stage::Done) {
        return Err(Error::ConjunctionState {
            reason: "unfinished conjunction result",
        });
    }
    frame.accumulator.ok_or(Error::ConjunctionState {
        reason: "missing conjunction result",
    })
}

#[cfg(feature = "semantics")]
#[path = "../../../generated/conjunction-control.rs"]
mod generated_conjunction_control;

#[cfg(feature = "semantics")]
pub fn conjunction(
    node: &JsonValue,
    name: &str,
    context: &mut EvaluationContext,
    mut evaluate: impl FnMut(&JsonValue, &mut EvaluationContext) -> Result<Value, Error>,
) -> Result<Value, Error> {
    conjunction_with_driver(
        node,
        name,
        context,
        &mut evaluate,
        super::unify,
        generated_conjunction_control::execute,
    )
}

#[cfg(feature = "semantics")]
pub fn conjunction_interpreted(
    node: &JsonValue,
    name: &str,
    context: &mut EvaluationContext,
    mut evaluate: impl FnMut(&JsonValue, &mut EvaluationContext) -> Result<Value, Error>,
    program: &str,
) -> Result<Value, Error> {
    run(node, name, context, &mut evaluate, super::unify, |frame| {
        crate::pratt::interpret_conjunction(program, frame)
    })
}

#[cfg(test)]
#[path = "conjunction_tests.rs"]
mod tests;
