//! Checked source-record storage and transitions; CUE owns traversal and syntax.
use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage {
    Selected,
    Named,
    CallOpened,
    CallTrivia,
    BodyOpened,
    BodyTrivia,
    Ready,
    LabelReady,
    LabelParsed,
    LabelTrivia,
    Colon,
    ValueParsed,
    Appended,
    Comma,
    Closed,
    CallClosed,
}

pub(super) struct SourceRecordState {
    stage: Stage,
    fields: Vec<Value>,
    key: Option<String>,
    child: Option<Value>,
}

impl Frame<'_, '_> {
    fn record_error<T>() -> Result<T, Error> {
        Err(Error::ControlState {
            reason: "invalid source record stage",
        })
    }

    fn record_stage(&self, stage: Stage) -> Result<(), Error> {
        if !self.in_primary
            || !matches!(self.primary_state, Some(PrimaryState::IdentifierScanned(_)))
            || self.left.is_some()
            || !self
                .record_state
                .as_ref()
                .is_some_and(|state| state.stage == stage)
        {
            return Self::record_error();
        }
        Ok(())
    }

    fn record_state_mut(&mut self) -> Result<&mut SourceRecordState, Error> {
        self.record_state.as_mut().ok_or(Error::ControlState {
            reason: "missing source record state",
        })
    }

    fn record_candidate(&self) -> Result<Option<usize>, Error> {
        if !self.in_primary {
            return Self::record_error();
        }
        let Some(PrimaryState::IdentifierScanned(end)) = self.primary_state else {
            return Self::record_error();
        };
        let Some(end) = end else { return Ok(None) };
        let Some(token) = self
            .parser
            .source
            .get(self.parser.offset..end)
            .filter(|_| end > self.parser.offset)
        else {
            return Self::record_error();
        };
        Ok(self
            .parser
            .grammar
            .record
            .as_ref()
            .filter(|record| record.token == token)
            .map(|_| end))
    }

    pub fn has_record(&self) -> Result<bool, Error> {
        if self.record_state.is_some() {
            return Self::record_error();
        }
        Ok(self.record_candidate()?.is_some())
    }

    /// Dispatch on this frame: no fuel refill and no native field loop.
    pub fn record(&mut self) -> Result<(), Error> {
        if !self.has_record()? || self.left.is_some() {
            return Self::record_error();
        }
        self.record_state = Some(SourceRecordState {
            stage: Stage::Selected,
            fields: Vec::new(),
            key: None,
            child: None,
        });
        match self.parser.record_engine.clone() {
            Engine::Compiled(driver) => driver(self)?,
            Engine::Interpreted(program) => crate::pratt::interpret(&program, self)?,
        }
        if !matches!(self.primary_state, Some(PrimaryState::Done))
            || self.left.is_none()
            || self.record_state.is_some()
        {
            return Self::record_error();
        }
        Ok(())
    }

    pub fn advance_record(&mut self) -> Result<(), Error> {
        self.record_stage(Stage::Selected)?;
        let Some(end) = self.record_candidate()? else {
            return Self::record_error();
        };
        self.advance_primary(end)?;
        self.record_state_mut()?.stage = Stage::Named;
        Ok(())
    }

    pub fn open_record_call(&mut self) -> Result<(), Error> {
        self.record_stage(Stage::Named)?;
        let scan = crate::scanner::call_open(self.parser.source, self.parser.offset)?;
        self.advance_primary(scan.offset)?;
        self.record_state_mut()?.stage = Stage::CallOpened;
        Ok(())
    }

    pub fn record_whitespace(&mut self) -> Result<(), Error> {
        let (previous, stage) = match self.record_state.as_ref().map(|state| state.stage) {
            Some(Stage::CallOpened) => (Stage::CallOpened, Stage::CallTrivia),
            Some(stage @ (Stage::BodyOpened | Stage::Comma)) => (stage, Stage::BodyTrivia),
            _ => return Self::record_error(),
        };
        self.record_stage(previous)?;
        self.parser.whitespace()?;
        self.record_state_mut()?.stage = stage;
        Ok(())
    }

    pub fn open_record_body(&mut self) -> Result<(), Error> {
        self.record_stage(Stage::CallTrivia)?;
        let scan = crate::scanner::record_open(self.parser.source, self.parser.offset)?;
        self.advance_primary(scan.offset)?;
        self.record_state_mut()?.stage = Stage::BodyOpened;
        Ok(())
    }

    pub fn probe_record_end(&mut self) -> Result<(), Error> {
        self.record_stage(Stage::BodyTrivia)?;
        let scan = crate::scanner::record_close(self.parser.source, self.parser.offset)?;
        if scan.flag {
            self.advance_primary(scan.offset)?;
        }
        self.record_state_mut()?.stage = if scan.flag {
            Stage::Closed
        } else {
            Stage::Ready
        };
        Ok(())
    }

    pub fn has_record_field(&self) -> Result<bool, Error> {
        match self.record_state.as_ref().map(|state| state.stage) {
            Some(stage @ (Stage::Ready | Stage::Comma)) => {
                self.record_stage(stage)?;
                Ok(true)
            }
            Some(Stage::Closed) => {
                self.record_stage(Stage::Closed)?;
                Ok(false)
            }
            _ => Self::record_error(),
        }
    }

    pub fn require_record_label(&mut self) -> Result<(), Error> {
        self.record_stage(Stage::Ready)?;
        let scan = crate::scanner::record_label(self.parser.source, self.parser.offset)?;
        if !scan.flag || scan.offset != self.parser.offset {
            return Self::record_error();
        }
        self.record_state_mut()?.stage = Stage::LabelReady;
        Ok(())
    }

    pub fn parse_record_label(&mut self) -> Result<(), Error> {
        self.record_stage(Stage::LabelReady)?;
        let start = self.parser.offset;
        let scan = crate::scanner::string_literal(self.parser.source, start)?;
        let Some(slice) = self
            .parser
            .source
            .get(start..scan.offset)
            .filter(|_| scan.offset > start)
        else {
            return Self::record_error();
        };
        let key = decode_string_literal(slice).ok_or_else(|| Error::Source {
            offset: start,
            reason: "invalid string literal".to_owned(),
        })?;
        self.advance_primary(scan.offset)?;
        let state = self.record_state_mut()?;
        state.key = Some(key);
        state.stage = Stage::LabelParsed;
        Ok(())
    }

    pub fn record_label_trivia(&mut self) -> Result<(), Error> {
        self.record_stage(Stage::LabelParsed)?;
        self.parser.after_operand()?;
        self.record_state_mut()?.stage = Stage::LabelTrivia;
        Ok(())
    }

    pub fn record_colon(&mut self) -> Result<(), Error> {
        self.record_stage(Stage::LabelTrivia)?;
        let scan = crate::scanner::record_colon(self.parser.source, self.parser.offset)?;
        self.advance_primary(scan.offset)?;
        self.record_state_mut()?.stage = Stage::Colon;
        Ok(())
    }

    pub fn parse_record_value(&mut self, minimum: u8) -> Result<(), Error> {
        self.record_stage(Stage::Colon)?;
        let child = self.parser.expression(minimum, self.depth + 1)?;
        let state = self.record_state_mut()?;
        state.child = Some(child);
        state.stage = Stage::ValueParsed;
        Ok(())
    }

    pub fn append_record_field(&mut self) -> Result<(), Error> {
        self.record_stage(Stage::ValueParsed)?;
        let state = self.record_state_mut()?;
        if state.fields.len() >= 64 {
            return self.parser.error("record exceeds 64 fields");
        }
        if state.key.is_none() || state.child.is_none() {
            return Self::record_error();
        }
        let key = state.key.take().ok_or(Error::ControlState {
            reason: "missing record key",
        })?;
        let child = state.child.take().ok_or(Error::ControlState {
            reason: "missing record child",
        })?;
        state.fields.push(ast::record_field(key, child));
        state.stage = Stage::Appended;
        Ok(())
    }

    pub fn advance_record_tail(&mut self) -> Result<(), Error> {
        self.record_stage(Stage::Appended)?;
        let scan = crate::scanner::record_tail(self.parser.source, self.parser.offset)?;
        self.advance_primary(scan.offset)?;
        self.record_state_mut()?.stage = if scan.flag {
            Stage::Closed
        } else {
            Stage::Comma
        };
        Ok(())
    }

    pub fn close_record_call(&mut self) -> Result<(), Error> {
        self.record_stage(Stage::Closed)?;
        let scan = crate::scanner::group_close(self.parser.source, self.parser.offset)?;
        self.advance_primary(scan.offset)?;
        self.record_state_mut()?.stage = Stage::CallClosed;
        Ok(())
    }

    pub fn build_record(&mut self) -> Result<(), Error> {
        self.record_stage(Stage::CallClosed)?;
        let constructor = self
            .parser
            .grammar
            .record
            .clone()
            .ok_or(Error::ControlState {
                reason: "missing record constructor",
            })?;
        let state = self.record_state_mut()?;
        if state.key.is_some() || state.child.is_some() {
            return Self::record_error();
        }
        self.parser.node()?;
        let state = self.record_state.take().ok_or(Error::ControlState {
            reason: "missing source record state",
        })?;
        self.left = Some(ast::unary(
            constructor.kind,
            constructor.fields_field,
            Value::Array(state.fields),
        ));
        self.primary_state = Some(PrimaryState::Done);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const GRAMMAR: &str = include_str!(concat!(env!("CUESTACEAN_BOOTSTRAP_JSON"), "/grammar.json"));

    #[test]
    fn stages_and_label_readiness() -> Result<(), Box<dyn std::error::Error>> {
        fn staged(frame: &mut Frame<'_, '_>) -> Result<(), Error> {
            assert!(frame.record().is_err());
            assert!(frame.build_record().is_err());
            frame.advance_record()?;
            assert!(frame.advance_record().is_err());
            frame.open_record_call()?;
            frame.record_whitespace()?;
            frame.open_record_body()?;
            frame.record_whitespace()?;
            frame.probe_record_end()?;
            assert!(frame.has_record_field()?);
            assert!(frame.parse_record_label().is_err());
            frame.require_record_label()?;
            assert!(frame.require_record_label().is_err());
            frame.parse_record_label()?;
            frame.record_label_trivia()?;
            frame.record_colon()?;
            frame.parse_record_value(0)?;
            assert!(frame.parse_record_value(0).is_err());
            frame.append_record_field()?;
            assert!(frame.append_record_field().is_err());
            frame.advance_record_tail()?;
            assert!(!frame.has_record_field()?);
            frame.close_record_call()?;
            frame.build_record()?;
            assert!(frame.record_state.is_none());
            assert_eq!(frame.parser.nodes, 2);
            Ok(())
        }
        parse_with_three_drivers(
            "close({\"x\":1})",
            GRAMMAR,
            generated::execute,
            generated_primary::execute,
            staged,
        )?;
        Ok(())
    }

    #[test]
    fn shared_fuel_and_unfinished_return() {
        fn identify(frame: &mut Frame<'_, '_>) -> Result<(), Error> {
            frame.whitespace()?;
            frame.scan_group()?;
            frame.scan_not()?;
            frame.scan_negation()?;
            frame.scan_bound()?;
            frame.scan_string()?;
            frame.scan_list()?;
            frame.scan_identifier()
        }
        fn primary(frame: &mut Frame<'_, '_>) -> Result<(), Error> {
            identify(frame)?;
            frame.fuel = 1;
            frame.record()
        }
        fn budget(frame: &mut Frame<'_, '_>) -> Result<(), Error> {
            frame.step()?;
            frame.step()
        }
        fn unfinished(frame: &mut Frame<'_, '_>) -> Result<(), Error> {
            frame.advance_record()
        }
        fn false_done(frame: &mut Frame<'_, '_>) -> Result<(), Error> {
            identify(frame)?;
            frame.record_state = Some(SourceRecordState {
                stage: Stage::Selected,
                fields: Vec::new(),
                key: None,
                child: None,
            });
            frame.left = Some(ast::atom("null".to_owned()));
            frame.primary_state = Some(PrimaryState::Done);
            Ok(())
        }
        let parse = |primary, record| {
            parse_with_three_drivers("close({})", GRAMMAR, generated::execute, primary, record)
        };
        assert!(matches!(parse(primary, budget), Err(Error::ControlBudget)));
        assert!(matches!(
            parse(primary, unfinished),
            Err(Error::ControlState { .. })
        ));
        assert!(matches!(
            parse(false_done, budget),
            Err(Error::ControlState { .. })
        ));
    }
}
