//! Checked bracket stages only; CUE selects slices and endpoint binding power.
use super::*;

impl Frame<'_, '_> {
    fn slice_phase(&self) -> Result<(), Error> {
        self.index_phase()?;
        if self.right.is_none() {
            return self.index_state_error();
        }
        Ok(())
    }

    pub fn scan_slice(&mut self) -> Result<(), Error> {
        self.slice_phase()?;
        if !matches!(self.index_state, Some(IndexState::Parsed)) || self.slice_upper.is_some() {
            return self.index_state_error();
        }
        let scan = crate::scanner::slice_separator(self.parser.source, self.parser.offset)?;
        if scan.flag {
            self.selector_span(scan.offset)?;
        } else if scan.offset != self.parser.offset {
            return self.index_state_error();
        }
        self.index_state = Some(IndexState::SliceScanned(
            (scan.flag && self.parser.grammar.slice.is_some()).then_some(scan.offset),
        ));
        Ok(())
    }

    pub fn has_slice(&self) -> Result<bool, Error> {
        self.slice_phase()?;
        match self.index_state {
            Some(IndexState::SliceScanned(candidate)) => Ok(candidate.is_some()),
            _ => self.index_state_error(),
        }
    }

    pub fn advance_slice(&mut self) -> Result<(), Error> {
        self.slice_phase()?;
        let Some(IndexState::SliceScanned(Some(end))) = self.index_state else {
            return self.index_state_error();
        };
        if self.slice_upper.is_some() || self.parser.grammar.slice.is_none() {
            return self.index_state_error();
        }
        self.advance_index_cursor(end)?;
        self.index_state = Some(IndexState::SliceAdvanced);
        Ok(())
    }

    pub fn parse_slice_upper(&mut self, minimum: u8) -> Result<(), Error> {
        self.slice_phase()?;
        if !matches!(self.index_state, Some(IndexState::SliceAdvanced))
            || self.slice_upper.is_some()
        {
            return self.index_state_error();
        }
        self.slice_upper = Some(Box::new(self.parser.expression(minimum, self.depth + 1)?));
        self.index_state = Some(IndexState::SliceParsed);
        Ok(())
    }

    pub fn close_slice(&mut self) -> Result<(), Error> {
        self.slice_phase()?;
        if !matches!(self.index_state, Some(IndexState::SliceParsed)) || self.slice_upper.is_none()
        {
            return self.index_state_error();
        }
        let constructor = self
            .parser
            .grammar
            .slice
            .clone()
            .ok_or(Error::ControlState {
                reason: "slice constructor disabled",
            })?;
        let scan = crate::scanner::index_close(self.parser.source, self.parser.offset)?;
        self.advance_index_cursor(scan.offset)?;
        // Charge before taking any operand, including the separately boxed upper.
        self.parser.node()?;
        let operand = self.left.take().ok_or(Error::ControlState {
            reason: "missing slice operand",
        })?;
        let lower = self.right.take().ok_or(Error::ControlState {
            reason: "missing slice lower operand",
        })?;
        let upper = self.slice_upper.take().ok_or(Error::ControlState {
            reason: "missing slice upper operand",
        })?;
        self.left = Some(ast::ternary(
            constructor.kind,
            constructor.operand_field,
            constructor.lower_field,
            constructor.upper_field,
            operand,
            lower,
            *upper,
        ));
        self.index_state = None;
        self.selector_state = None;
        Ok(())
    }
}
