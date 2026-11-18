//! Checked selector transitions; CUE owns postfix choice, trivia and labels.
use super::*;

pub(super) enum SelectorState {
    Scanned(Option<usize>),
    Advanced,
    TriviaReady,
    StringScanned(Option<usize>),
    Label(String),
}

impl Frame<'_, '_> {
    pub(super) fn selector_error<T>() -> Result<T, Error> {
        Err(Error::ControlState {
            reason: "invalid selector stage",
        })
    }

    pub(super) fn selector_pending(&self) -> bool {
        !matches!(
            self.selector_state,
            None | Some(SelectorState::Scanned(None))
        )
    }

    fn selector_phase(&self) -> Result<(), Error> {
        if self.in_primary
            || self.left.is_none()
            || self.right.is_some()
            || self.consumed
            || self.operator.is_some()
            || self.slice_upper.is_some()
            || !matches!(self.index_state, Some(IndexState::Scanned(None)))
        {
            return Self::selector_error();
        }
        Ok(())
    }

    pub(super) fn selector_span(&self, end: usize) -> Result<&str, Error> {
        self.parser
            .source
            .get(self.parser.offset..end)
            .filter(|_| end > self.parser.offset)
            .ok_or(Error::ControlState {
                reason: "invalid selector progress",
            })
    }

    /// Candidate presence only; CUE decides which postfix action to execute.
    pub fn has_postfix(&self) -> Result<bool, Error> {
        if self.in_primary
            || self.left.is_none()
            || self.right.is_some()
            || self.consumed
            || self.operator.is_some()
            || self.slice_upper.is_some()
        {
            return Self::selector_error();
        }
        match (&self.index_state, &self.selector_state) {
            (Some(IndexState::Scanned(Some(_))), None) => Ok(true),
            (Some(IndexState::Scanned(None)), Some(SelectorState::Scanned(candidate))) => {
                Ok(candidate.is_some())
            }
            _ => Self::selector_error(),
        }
    }

    pub fn scan_selector(&mut self) -> Result<(), Error> {
        self.selector_phase()?;
        if self.selector_state.is_some() {
            return Self::selector_error();
        }
        let scan = crate::scanner::selector_open(self.parser.source, self.parser.offset)?;
        if scan.flag {
            self.selector_span(scan.offset)?;
        } else if scan.offset != self.parser.offset {
            return Self::selector_error();
        }
        self.selector_state = Some(SelectorState::Scanned(
            (scan.flag && self.parser.grammar.selector.is_some()).then_some(scan.offset),
        ));
        Ok(())
    }

    pub fn advance_selector(&mut self) -> Result<(), Error> {
        self.selector_phase()?;
        let Some(SelectorState::Scanned(Some(end))) = self.selector_state else {
            return Self::selector_error();
        };
        if self.parser.grammar.selector.is_none() {
            return Self::selector_error();
        }
        self.selector_span(end)?;
        self.parser.offset = end;
        self.selector_state = Some(SelectorState::Advanced);
        Ok(())
    }

    pub fn selector_whitespace(&mut self) -> Result<(), Error> {
        self.selector_phase()?;
        if !matches!(self.selector_state, Some(SelectorState::Advanced)) {
            return Self::selector_error();
        }
        self.parser.whitespace()?;
        self.selector_state = Some(SelectorState::TriviaReady);
        Ok(())
    }

    pub fn scan_selector_string(&mut self) -> Result<(), Error> {
        self.selector_phase()?;
        if !matches!(self.selector_state, Some(SelectorState::TriviaReady)) {
            return Self::selector_error();
        }
        let scan = crate::scanner::string_literal(self.parser.source, self.parser.offset)?;
        if scan.flag {
            self.selector_span(scan.offset)?;
        } else if scan.offset != self.parser.offset {
            return Self::selector_error();
        }
        self.selector_state = Some(SelectorState::StringScanned(
            scan.flag.then_some(scan.offset),
        ));
        Ok(())
    }

    pub fn has_selector_string(&self) -> Result<bool, Error> {
        self.selector_phase()?;
        match self.selector_state {
            Some(SelectorState::StringScanned(candidate)) => Ok(candidate.is_some()),
            _ => Self::selector_error(),
        }
    }

    pub fn parse_selector_string(&mut self) -> Result<(), Error> {
        self.selector_phase()?;
        let Some(SelectorState::StringScanned(Some(end))) = self.selector_state else {
            return Self::selector_error();
        };
        let label =
            decode_string_literal(self.selector_span(end)?).ok_or_else(|| Error::Source {
                offset: self.parser.offset,
                reason: "invalid string literal".to_owned(),
            })?;
        self.parser.offset = end;
        self.selector_state = Some(SelectorState::Label(label));
        Ok(())
    }

    pub fn parse_selector_identifier(&mut self) -> Result<(), Error> {
        self.selector_phase()?;
        if !matches!(
            self.selector_state,
            Some(SelectorState::StringScanned(None))
        ) {
            return Self::selector_error();
        }
        let scan = crate::scanner::selector_identifier(self.parser.source, self.parser.offset)?;
        let label = self.selector_span(scan.offset)?.to_owned();
        self.parser.offset = scan.offset;
        self.selector_state = Some(SelectorState::Label(label));
        Ok(())
    }

    pub fn build_selector(&mut self) -> Result<(), Error> {
        self.selector_phase()?;
        if !matches!(self.selector_state, Some(SelectorState::Label(_))) {
            return Self::selector_error();
        }
        let constructor = self
            .parser
            .grammar
            .selector
            .clone()
            .ok_or(Error::ControlState {
                reason: "selector constructor disabled",
            })?;
        self.parser.node()?;
        let Some(SelectorState::Label(label)) = self.selector_state.take() else {
            return Self::selector_error();
        };
        let operand = self.left.take().ok_or(Error::ControlState {
            reason: "missing selector operand",
        })?;
        self.left = Some(ast::binary(
            constructor.kind,
            constructor.operand_field,
            constructor.label_field,
            operand,
            Value::String(label),
        ));
        self.index_state = None;
        Ok(())
    }
}
