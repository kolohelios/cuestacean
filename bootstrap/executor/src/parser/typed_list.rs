//! Checked typed-tail stages. CUE owns the branch, recursive minimum and return.
use super::*;

impl Frame<'_, '_> {
    fn ellipsis_scan(&self, close: bool) -> Result<crate::scanner::Scan, Error> {
        let source = self.parser.source;
        let offset = self.parser.offset;
        Ok(match self.parser.primary_engine {
            Engine::Compiled(_) if close => crate::scanner::list_ellipsis_close(source, offset)?,
            Engine::Compiled(_) => crate::scanner::list_ellipsis(source, offset)?,
            Engine::Interpreted(_) => crate::scanner::interpret(
                if close {
                    include_str!(concat!(
                        env!("CUESTACEAN_BOOTSTRAP_JSON"),
                        "/list-ellipsis-close.json"
                    ))
                } else {
                    include_str!(concat!(
                        env!("CUESTACEAN_BOOTSTRAP_JSON"),
                        "/list-ellipsis.json"
                    ))
                },
                source,
                offset,
            )?,
        })
    }

    pub fn probe_list_ellipsis(&mut self) -> Result<(), Error> {
        if !matches!(self.primary_state, Some(PrimaryState::ListReady)) || self.left.is_some() {
            return self.primary_state_error();
        }
        let scan = self.ellipsis_scan(false)?;
        self.primary_state = Some(PrimaryState::ListEllipsisScanned(
            scan.flag.then_some(scan.offset),
        ));
        Ok(())
    }

    pub fn has_list_ellipsis(&self) -> Result<bool, Error> {
        match self.primary_state {
            Some(PrimaryState::ListEllipsisScanned(end)) => Ok(end.is_some()),
            _ => self.primary_state_error(),
        }
    }

    pub fn advance_list_ellipsis(&mut self) -> Result<(), Error> {
        let Some(PrimaryState::ListEllipsisScanned(Some(end))) = self.primary_state else {
            return self.primary_state_error();
        };
        if self.parser.grammar.typed_list.is_none() || self.left.is_some() {
            return self.primary_state_error();
        }
        self.advance_primary(end)?;
        self.primary_state = Some(PrimaryState::ListEllipsisAdvanced);
        Ok(())
    }

    pub fn close_list_ellipsis(&mut self) -> Result<(), Error> {
        if !matches!(self.primary_state, Some(PrimaryState::ListEllipsisParsed))
            || self.left.is_none()
        {
            return self.primary_state_error();
        }
        let scan = self.ellipsis_scan(true)?;
        self.advance_primary(scan.offset)?;
        self.primary_state = Some(PrimaryState::ListEllipsisClosed);
        Ok(())
    }

    pub fn build_typed_list(&mut self) -> Result<(), Error> {
        if !matches!(self.primary_state, Some(PrimaryState::ListEllipsisClosed))
            || self.left.is_none()
        {
            return self.primary_state_error();
        }
        let constructor = self
            .parser
            .grammar
            .typed_list
            .clone()
            .ok_or(Error::ControlState {
                reason: "missing typed list constructor",
            })?;
        self.parser.node()?;
        let tail = self.left.take().ok_or(Error::ControlState {
            reason: "missing list tail",
        })?;
        self.left = Some(ast::binary(
            constructor.kind,
            constructor.elements_field,
            constructor.tail_field,
            Value::Array(std::mem::take(&mut self.list_elements)),
            tail,
        ));
        self.primary_state = Some(PrimaryState::Done);
        Ok(())
    }
}
