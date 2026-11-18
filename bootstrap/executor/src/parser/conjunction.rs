//! Checked conjunction stages; the selected CUE driver owns operand traversal.
use super::*;

impl Frame<'_, '_> {
    fn conjunction_candidate(&self) -> Result<Option<usize>, Error> {
        if !self.in_primary || self.record_state.is_some() {
            return self.primary_state_error();
        }
        let Some(PrimaryState::IdentifierScanned(end)) = self.primary_state else {
            return self.primary_state_error();
        };
        let Some(end) = end else { return Ok(None) };
        let Some(token) = self
            .parser
            .source
            .get(self.parser.offset..end)
            .filter(|_| end > self.parser.offset)
        else {
            return self.primary_state_error();
        };
        Ok(self
            .parser
            .grammar
            .conjunction
            .as_ref()
            .filter(|constructor| constructor.token == token)
            .map(|_| end))
    }

    pub fn has_conjunction(&self) -> Result<bool, Error> {
        Ok(self.conjunction_candidate()?.is_some())
    }

    /// Dispatch on the same frame, retaining its fuel and captured name span.
    pub fn conjunction(&mut self) -> Result<(), Error> {
        let Some(end) = self.conjunction_candidate()? else {
            return self.primary_state_error();
        };
        if self.left.is_some() || !self.list_elements.is_empty() {
            return self.primary_state_error();
        }
        self.primary_state = Some(PrimaryState::ConjunctionSelected(end));
        match self.parser.conjunction_engine.clone() {
            Engine::Compiled(driver) => driver(self)?,
            Engine::Interpreted(program) => crate::pratt::interpret(&program, self)?,
        }
        if !matches!(self.primary_state, Some(PrimaryState::Done)) || self.left.is_none() {
            return self.primary_state_error();
        }
        Ok(())
    }

    pub fn advance_conjunction(&mut self) -> Result<(), Error> {
        let Some(PrimaryState::ConjunctionSelected(end)) = self.primary_state else {
            return self.primary_state_error();
        };
        self.advance_primary(end)?;
        self.primary_state = Some(PrimaryState::ConjunctionNamed);
        Ok(())
    }

    pub fn open_conjunction_call(&mut self) -> Result<(), Error> {
        if !matches!(self.primary_state, Some(PrimaryState::ConjunctionNamed)) {
            return self.primary_state_error();
        }
        let scan = crate::scanner::call_open(self.parser.source, self.parser.offset)?;
        self.advance_primary(scan.offset)?;
        self.primary_state = Some(PrimaryState::ConjunctionCallOpened);
        Ok(())
    }

    pub fn open_conjunction_list(&mut self) -> Result<(), Error> {
        if !matches!(
            self.primary_state,
            Some(PrimaryState::ConjunctionCallOpened)
        ) {
            return self.primary_state_error();
        }
        let scan = crate::scanner::conjunction_open(self.parser.source, self.parser.offset)?;
        self.advance_primary(scan.offset)?;
        self.primary_state = Some(PrimaryState::ConjunctionListReady);
        Ok(())
    }

    pub fn close_conjunction_call(&mut self) -> Result<(), Error> {
        if !matches!(
            self.primary_state,
            Some(PrimaryState::ConjunctionListClosed)
        ) || self.left.is_some()
        {
            return self.primary_state_error();
        }
        let scan = crate::scanner::group_close(self.parser.source, self.parser.offset)?;
        self.advance_primary(scan.offset)?;
        self.primary_state = Some(PrimaryState::ConjunctionCallClosed);
        Ok(())
    }

    pub fn build_conjunction(&mut self) -> Result<(), Error> {
        if !matches!(
            self.primary_state,
            Some(PrimaryState::ConjunctionCallClosed)
        ) || self.left.is_some()
        {
            return self.primary_state_error();
        }
        let constructor = self
            .parser
            .grammar
            .conjunction
            .clone()
            .ok_or(Error::ControlState {
                reason: "missing conjunction constructor",
            })?;
        // Charge before taking ownership so a failed charge retains the buffer.
        self.parser.node()?;
        self.left = Some(ast::unary(
            constructor.kind,
            constructor.operands_field,
            Value::Array(std::mem::take(&mut self.list_elements)),
        ));
        self.primary_state = Some(PrimaryState::Done);
        Ok(())
    }
}
