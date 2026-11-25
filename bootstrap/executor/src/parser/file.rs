//! File-profile control bridge. CUE owns recognition, iteration and boundaries;
//! this module owns checked spans, staging, shared budgets and AST builders.
use super::*;

#[path = "../../../generated/file-boundary.rs"]
mod boundary;
#[path = "../../../generated/file-colon.rs"]
mod colon;
#[path = "../../../generated/file-pratt.rs"]
mod expression;
#[path = "../../../generated/file-label.rs"]
mod label;
#[path = "../../../generated/file-primary.rs"]
mod primary;
#[path = "../../../generated/file-tail.rs"]
mod tail;

macro_rules! program {
    ($name:literal) => {
        include_str!(concat!(
            env!("CUESTACEAN_BOOTSTRAP_JSON"),
            "/",
            $name,
            ".json"
        ))
    };
}
fn interpreted(program: &str) -> Result<Engine, Error> {
    Ok(Engine::Interpreted(Arc::new(
        crate::pratt::parse(program).context(ControlSnafu)?,
    )))
}

/// Parse an explicit file, probing declarations without committing expression input.
pub fn parse_file(source: &str, grammar: &str) -> Result<Value, Error> {
    parse_entry(source, grammar, false)
}

/// Independent control/scanner VM entry for the same file profile.
pub fn parse_file_interpreted(source: &str, grammar: &str) -> Result<Value, Error> {
    parse_entry(source, grammar, true)
}
fn parse_entry(source: &str, grammar: &str, reference: bool) -> Result<Value, Error> {
    let grammar: Grammar = serde_json::from_str(grammar).context(JsonSnafu)?;
    grammar.validate()?;
    let engine = |driver, program| {
        if reference {
            interpreted(program)
        } else {
            Ok(Engine::Compiled(driver))
        }
    };
    let entry_engine = engine(primary::execute as Driver, program!("file-primary"))?;
    let mut parser = Parser {
        source,
        grammar: &grammar,
        offset: 0,
        nodes: 0,
        engine: engine(expression::execute as Driver, program!("file-pratt"))?,
        primary_engine: engine(generated_primary::execute as Driver, PRIMARY_PROGRAM)?,
        record_engine: engine(generated_record::execute as Driver, RECORD_PROGRAM)?,
        conjunction_engine: engine(
            generated_conjunction::execute as Driver,
            CONJUNCTION_PROGRAM,
        )?,
    };
    let entry = parser.expression_control(0, 0, false, entry_engine)?;
    parser.whitespace()?;
    if parser.offset != source.len() {
        return parser.error("unexpected or unsupported syntax");
    }
    Ok(ast::program(entry))
}

#[derive(Default)]
pub(super) struct Context {
    pub scope: bool,
    boundary: Option<bool>,
    pub state: Option<State>,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage {
    Probed,
    Ready,
    Label,
    Colon,
    Value,
    Appended,
    Closed,
}
pub(super) struct State {
    stage: Stage,
    more: bool,
    start: usize,
    fields: Vec<Value>,
    names: BTreeSet<String>,
    key: Option<String>,
    child: Option<Value>,
}
#[cfg(test)]
#[path = "file_tests.rs"]
mod tests;

enum Scanner {
    Label,
    Colon,
    Boundary,
    Tail,
}
impl Frame<'_, '_> {
    fn file_scan(&self, scanner: Scanner, offset: usize) -> Result<crate::scanner::Scan, Error> {
        let (driver, program): (
            fn(&str, usize) -> Result<crate::scanner::Scan, crate::scanner::Error>,
            &str,
        ) = match scanner {
            Scanner::Label => (label::scan, program!("file-label")),
            Scanner::Colon => (colon::scan, program!("file-colon")),
            Scanner::Boundary => (boundary::scan, program!("file-boundary")),
            Scanner::Tail => (tail::scan, program!("file-tail")),
        };
        Ok(match self.parser.engine {
            Engine::Interpreted(_) => {
                crate::scanner::interpret(program, self.parser.source, offset)?
            }
            Engine::Compiled(_) => driver(self.parser.source, offset)?,
        })
    }
    fn file_stage(&self, stage: Stage) -> Result<(), Error> {
        if self.in_primary
            || self.left.is_some()
            || self.consumed
            || !self
                .file
                .state
                .as_ref()
                .is_some_and(|state| state.stage == stage)
        {
            return Err(Error::ControlState {
                reason: "invalid file stage",
            });
        }
        Ok(())
    }
    fn file_state(&mut self) -> Result<&mut State, Error> {
        self.file.state.as_mut().ok_or(Error::ControlState {
            reason: "missing file state",
        })
    }
    fn file_advance(&mut self, end: usize) -> Result<(), Error> {
        if end <= self.parser.offset || !self.parser.source.is_char_boundary(end) {
            return Err(Error::ControlState {
                reason: "invalid file progress",
            });
        }
        self.parser.offset = end;
        Ok(())
    }
    pub fn probe_file(&mut self) -> Result<(), Error> {
        if self.file.state.is_some()
            || self.left.is_some()
            || self.depth != 0
            || self.parser.offset != 0
        {
            return Err(Error::ControlState {
                reason: "file probe out of order",
            });
        }
        let start = self.file_scan(Scanner::Boundary, 0)?.offset;
        let label = self.file_scan(Scanner::Label, start)?;
        let more = label.flag && self.file_scan(Scanner::Colon, label.offset)?.flag;
        self.file.state = Some(State {
            stage: Stage::Probed,
            more,
            start,
            fields: Vec::new(),
            names: BTreeSet::new(),
            key: None,
            child: None,
        });
        Ok(())
    }
    pub fn has_file_field(&self) -> Result<bool, Error> {
        let state = self.file.state.as_ref().ok_or(Error::ControlState {
            reason: "missing file probe",
        })?;
        if !matches!(state.stage, Stage::Probed | Stage::Ready | Stage::Closed) {
            return Err(Error::ControlState {
                reason: "file field test out of order",
            });
        }
        self.file_stage(state.stage)?;
        Ok(state.more)
    }
    pub fn start_file(&mut self) -> Result<(), Error> {
        self.file_stage(Stage::Probed)?;
        if !self.has_file_field()? {
            return Err(Error::ControlState {
                reason: "file not recognized",
            });
        }
        if self.parser.grammar.record.is_none() {
            return self.parser.error("file record constructor required");
        }
        self.parser.node()?;
        self.parser.offset = self.file_state()?.start;
        self.file_state()?.stage = Stage::Ready;
        Ok(())
    }
    pub fn parse_file_label(&mut self) -> Result<(), Error> {
        self.file_stage(Stage::Ready)?;
        let start = self.parser.offset;
        let scan = self.file_scan(Scanner::Label, start)?;
        if !scan.flag {
            return self.parser.error("expected file label");
        }
        let key = self
            .parser
            .source
            .get(start..scan.offset)
            .ok_or(Error::ControlState {
                reason: "invalid file label span",
            })?
            .to_owned();
        self.file_advance(scan.offset)?;
        if matches!(key.as_str(), "package" | "import") {
            return self.parser.error("unsupported file label");
        }
        if !self.file_state()?.names.insert(key.clone()) {
            return self.parser.error("duplicate file field");
        }
        self.file_state()?.key = Some(key);
        self.file_state()?.stage = Stage::Label;
        Ok(())
    }
    pub fn parse_file_colon(&mut self) -> Result<(), Error> {
        self.file_stage(Stage::Label)?;
        let scan = self.file_scan(Scanner::Colon, self.parser.offset)?;
        if !scan.flag {
            self.parser.offset = scan.offset;
            return self.parser.error("expected file colon");
        }
        self.file_advance(scan.offset)?;
        self.file_state()?.stage = Stage::Colon;
        Ok(())
    }
    pub fn parse_file_value(&mut self) -> Result<(), Error> {
        self.file_stage(Stage::Colon)?;
        let child =
            self.parser
                .expression_control(0, self.depth + 1, true, self.parser.engine.clone())?;
        self.file_state()?.child = Some(child);
        self.file_state()?.stage = Stage::Value;
        Ok(())
    }
    pub fn append_file_field(&mut self) -> Result<(), Error> {
        self.file_stage(Stage::Value)?;
        let state = self.file_state()?;
        if state.fields.len() >= 64 {
            return self.parser.error("record exceeds 64 fields");
        }
        let key = state.key.take().ok_or(Error::ControlState {
            reason: "missing file key",
        })?;
        let child = state.child.take().ok_or(Error::ControlState {
            reason: "missing file child",
        })?;
        state.fields.push(ast::record_field(key, child));
        state.stage = Stage::Appended;
        Ok(())
    }
    pub fn advance_file_tail(&mut self) -> Result<(), Error> {
        self.file_stage(Stage::Appended)?;
        let scan = self.file_scan(Scanner::Tail, self.parser.offset)?;
        if scan.offset < self.parser.offset || !self.parser.source.is_char_boundary(scan.offset) {
            return Err(Error::ControlState {
                reason: "invalid file tail span",
            });
        }
        self.parser.offset = scan.offset;
        let state = self.file_state()?;
        state.more = scan.flag;
        state.stage = if scan.flag {
            Stage::Ready
        } else {
            Stage::Closed
        };
        Ok(())
    }
    pub fn build_file(&mut self) -> Result<(), Error> {
        self.file_stage(Stage::Closed)?;
        let state = self.file.state.take().ok_or(Error::ControlState {
            reason: "missing file result",
        })?;
        if state.fields.is_empty()
            || state.key.is_some()
            || state.child.is_some()
            || self.parser.offset != self.parser.source.len()
        {
            return Err(Error::ControlState {
                reason: "unfinished file result",
            });
        }
        let constructor = self
            .parser
            .grammar
            .record
            .as_ref()
            .ok_or(Error::ControlState {
                reason: "missing file constructor",
            })?;
        self.left = Some(ast::unary(
            constructor.kind.clone(),
            constructor.fields_field.clone(),
            Value::Array(state.fields),
        ));
        Ok(())
    }
    pub fn parse_file_expression(&mut self) -> Result<(), Error> {
        self.file_stage(Stage::Probed)?;
        if self.has_file_field()? {
            return Err(Error::ControlState {
                reason: "cannot retry recognized file",
            });
        }
        self.file.state = None;
        let old = match self.parser.engine {
            Engine::Compiled(_) => Engine::Compiled(generated::execute),
            Engine::Interpreted(_) => interpreted(program!("pratt"))?,
        };
        let selected = std::mem::replace(&mut self.parser.engine, old);
        let result = self.parser.expression(0, 0);
        self.parser.engine = selected;
        self.left = Some(result?);
        Ok(())
    }
    pub fn file_after_operand(&mut self) -> Result<(), Error> {
        self.expression_phase()?;
        if self.left.is_none() {
            return Err(Error::ControlState {
                reason: "operand not parsed",
            });
        }
        self.file.boundary = Some(false);
        if !self.file.scope {
            return self.parser.after_operand();
        }
        let scan = self.file_scan(Scanner::Boundary, self.parser.offset)?;
        let stop = scan.flag && scan.offset < self.parser.source.len();
        self.file.boundary = Some(stop);
        // Preserve the trivia offset across every recursive RHS return, so a
        // parent cannot consume an operator-named declaration as an infix.
        if !stop {
            self.parser.after_operand()?;
        }
        Ok(())
    }
    pub fn file_boundary(&self) -> Result<bool, Error> {
        self.expression_phase()?;
        self.file.boundary.ok_or(Error::ControlState {
            reason: "file boundary not scanned",
        })
    }
}
