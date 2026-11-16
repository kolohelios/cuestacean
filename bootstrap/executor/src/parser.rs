//! Narrow bootstrap Pratt parser. Token scanning and lexical policies are
//! CUE-generated, as are AST construction and the Pratt expression loop.
//! Primary dispatch and grouping policy also come from CUE. Grammar lookup,
//! conversion, scanner/builder calls and frame safety remain native.

use std::collections::BTreeSet;
use std::sync::Arc;

#[path = "../../generated/pratt.rs"]
mod generated;
#[path = "../../generated/primary.rs"]
mod generated_primary;

const PRIMARY_PROGRAM: &str =
    include_str!(concat!(env!("CUESTACEAN_BOOTSTRAP_JSON"), "/primary.json"));

use serde::Deserialize;
use serde_json::Value;

use crate::ast;
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("{source}"))]
    Control { source: crate::pratt::Error },
    #[snafu(display("invalid parser control state: {reason}"))]
    ControlState { reason: &'static str },
    #[snafu(display("parser control execution budget exhausted"))]
    ControlBudget,
    #[snafu(display("parser binding-power arithmetic overflow"))]
    BindingPower,
    #[snafu(display("scanner failed: {source}"))]
    Scanner { source: crate::scanner::Error },
    #[snafu(display("invalid grammar JSON: {source}"))]
    Json { source: serde_json::Error },
    #[snafu(display("invalid bootstrap grammar: {reason}"))]
    Grammar { reason: &'static str },
    #[snafu(display("source byte {offset}: {reason}"))]
    Source { offset: usize, reason: String },
    #[snafu(display("source byte {offset}: signed 64-bit integer required: {source}"))]
    Integer {
        offset: usize,
        source: std::num::ParseIntError,
    },
}

impl From<crate::scanner::Error> for Error {
    fn from(source: crate::scanner::Error) -> Self {
        match source {
            crate::scanner::Error::Rejected { offset, reason } => Self::Source { offset, reason },
            source => Self::Scanner { source },
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Grammar {
    format_version: u32,
    integer: Integer,
    #[serde(default)]
    text: Option<Integer>,
    #[serde(default)]
    atoms: Vec<Atom>,
    #[serde(default)]
    bounds: Vec<BoundPrefix>,
    #[serde(default)]
    negation: Option<Negation>,
    group: Group,
    operators: Vec<Operator>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Integer {
    kind: String,
    value_field: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Atom {
    token: String,
    kind: String,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct BoundPrefix {
    token: String,
    kind: String,
    value_field: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Negation {
    token: String,
    kind: String,
    operand_field: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Group {
    open: String,
    close: String,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Operator {
    token: String,
    kind: String,
    left_field: String,
    right_field: String,
    precedence: u8,
    associativity: Associativity,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Associativity {
    Left,
    Right,
}

impl Grammar {
    fn validate(&self) -> Result<(), Error> {
        let field = |name: &str| !name.is_empty() && name != "kind";
        if self.format_version != 1 || self.group.open != "(" || self.group.close != ")" {
            return GrammarSnafu {
                reason: "version 1 and parentheses required",
            }
            .fail();
        }
        if self.integer.kind.is_empty()
            || !field(&self.integer.value_field)
            || self.operators.is_empty()
        {
            return GrammarSnafu {
                reason: "integer constructor and operators required",
            }
            .fail();
        }
        if let Some(text) = &self.text
            && (text.kind.is_empty() || !field(&text.value_field))
        {
            return GrammarSnafu {
                reason: "invalid string constructor",
            }
            .fail();
        }
        if let Some(negation) = &self.negation
            && (negation.token != "-"
                || negation.kind.is_empty()
                || !field(&negation.operand_field))
        {
            return GrammarSnafu {
                reason: "invalid negation configuration",
            }
            .fail();
        }
        let mut atoms = BTreeSet::new();
        for atom in &self.atoms {
            let identifier = !atom.token.is_empty()
                && atom.token.bytes().enumerate().all(|(index, byte)| {
                    byte == b'_'
                        || if index == 0 {
                            byte.is_ascii_alphabetic()
                        } else {
                            byte.is_ascii_alphanumeric()
                        }
                });
            if !identifier || atom.kind.is_empty() || !atoms.insert(&atom.token) {
                return GrammarSnafu {
                    reason: "invalid or ambiguous atom configuration",
                }
                .fail();
            }
        }
        let mut bounds = BTreeSet::new();
        for bound in &self.bounds {
            if !matches!(bound.token.as_str(), ">=" | "<=")
                || !bounds.insert(&bound.token)
                || bound.kind.is_empty()
                || !field(&bound.value_field)
            {
                return GrammarSnafu {
                    reason: "invalid or ambiguous bound configuration",
                }
                .fail();
            }
        }
        let mut tokens = BTreeSet::new();
        for op in &self.operators {
            if !matches!(
                op.token.as_str(),
                "+" | "-" | "*" | "/" | "&" | "==" | "!=" | "div" | "mod" | "quo" | "rem"
            ) || !tokens.insert(&op.token)
                || op.kind.is_empty()
                || !field(&op.left_field)
                || !field(&op.right_field)
                || op.left_field == op.right_field
                || !(1..=100).contains(&op.precedence)
            {
                return GrammarSnafu {
                    reason: "invalid or ambiguous operator configuration",
                }
                .fail();
            }
        }
        Ok(())
    }
}

/// Parse the documented decimal-integer subset into a version-1 expression tree.
/// Limits: at most 64 AST nodes and 32 recursive parsing levels.
pub fn parse(source: &str, grammar: &str) -> Result<Value, Error> {
    parse_with_drivers(
        source,
        grammar,
        generated::execute,
        generated_primary::execute,
    )
}

/// Trusted native driver hook for comparing compiled CUE control programs.
pub type Driver = for<'p, 's> fn(&mut Frame<'p, 's>) -> Result<(), Error>;

#[derive(Clone)]
enum Engine {
    Compiled(Driver),
    Interpreted(Arc<crate::pratt::Program>),
}

pub fn parse_with_driver(source: &str, grammar: &str, driver: Driver) -> Result<Value, Error> {
    parse_with_drivers(source, grammar, driver, generated_primary::execute)
}

/// Trusted native hooks for expression and primary control programs.
pub fn parse_with_drivers(
    source: &str,
    grammar: &str,
    expression: Driver,
    primary: Driver,
) -> Result<Value, Error> {
    parse_engine(
        source,
        grammar,
        Engine::Compiled(expression),
        Engine::Compiled(primary),
    )
}

pub fn parse_interpreted(source: &str, grammar: &str, program: &str) -> Result<Value, Error> {
    parse_interpreted_with_primary(source, grammar, program, PRIMARY_PROGRAM)
}

pub fn parse_interpreted_with_primary(
    source: &str,
    grammar: &str,
    expression: &str,
    primary: &str,
) -> Result<Value, Error> {
    let expression = crate::pratt::parse(expression).context(ControlSnafu)?;
    let primary = crate::pratt::parse(primary).context(ControlSnafu)?;
    parse_engine(
        source,
        grammar,
        Engine::Interpreted(Arc::new(expression)),
        Engine::Interpreted(Arc::new(primary)),
    )
}

fn parse_engine(
    source: &str,
    grammar: &str,
    engine: Engine,
    primary_engine: Engine,
) -> Result<Value, Error> {
    let grammar: Grammar = serde_json::from_str(grammar).context(JsonSnafu)?;
    grammar.validate()?;
    let mut parser = Parser {
        source,
        grammar: &grammar,
        offset: 0,
        nodes: 0,
        engine,
        primary_engine,
    };
    let entry = parser.expression(0, 0)?;
    parser.whitespace()?;
    if parser.offset != source.len() {
        return parser.error("unexpected or unsupported syntax");
    }
    Ok(ast::program(entry))
}

struct Parser<'a> {
    source: &'a str,
    grammar: &'a Grammar,
    offset: usize,
    nodes: usize,
    engine: Engine,
    primary_engine: Engine,
}

impl Parser<'_> {
    fn error<T>(&self, reason: &'static str) -> Result<T, Error> {
        SourceSnafu {
            offset: self.offset,
            reason,
        }
        .fail()
    }

    fn whitespace(&mut self) -> Result<(), Error> {
        let scan = crate::scanner::whitespace(self.source, self.offset)?;
        self.offset = scan.offset;
        Ok(())
    }

    fn after_operand(&mut self) -> Result<(), Error> {
        let scan = crate::scanner::after_operand(self.source, self.offset)?;
        self.offset = scan.offset;
        Ok(())
    }

    fn node(&mut self) -> Result<(), Error> {
        self.nodes += 1;
        if self.nodes > 64 {
            return self.error("expression exceeds 64 nodes");
        }
        Ok(())
    }

    fn expression(&mut self, minimum: u8, depth: usize) -> Result<Value, Error> {
        if depth > 32 {
            return self.error("expression exceeds 32 parsing levels");
        }
        let engine = self.engine.clone();
        let mut frame = Frame {
            parser: self,
            minimum,
            depth,
            left: None,
            right: None,
            operator: None,
            token_end: 0,
            consumed: false,
            binding: None,
            fuel: 4096,
            primary_state: None,
            in_primary: false,
        };
        match engine {
            Engine::Compiled(driver) => driver(&mut frame)?,
            Engine::Interpreted(program) => crate::pratt::interpret(&program, &mut frame)?,
        }
        if frame.consumed || frame.right.is_some() {
            return Err(Error::ControlState {
                reason: "unfinished operator at frame return",
            });
        }
        frame.left.ok_or(Error::ControlState {
            reason: "missing expression result",
        })
    }

    fn atom(&mut self, end: usize) -> Result<Value, Error> {
        let start = self.offset;
        self.offset = end;
        let token = &self.source[start..self.offset];
        let kind = self
            .grammar
            .atoms
            .iter()
            .find(|atom| atom.token == token)
            .map(|atom| atom.kind.clone())
            .ok_or_else(|| Error::Source {
                offset: start,
                reason: "unknown atom".into(),
            })?;
        self.node()?;
        Ok(ast::atom(kind))
    }

    fn decimal(&mut self) -> Result<i64, Error> {
        let start = self.offset;
        let scan = crate::scanner::decimal(self.source, start)?;
        self.offset = scan.offset;
        self.source[start..self.offset]
            .parse::<i64>()
            .context(IntegerSnafu { offset: start })
    }

    fn integer(&mut self) -> Result<Value, Error> {
        let value = self.decimal()?;
        self.node()?;
        Ok(ast::integer(
            self.grammar.integer.kind.clone(),
            self.grammar.integer.value_field.clone(),
            value,
        ))
    }
}

#[derive(Clone, Copy)]
enum PrimaryState {
    Start,
    GroupScanned(Option<usize>),
    GroupAdvanced,
    GroupParsed,
    NegationScanned(Option<usize>),
    NegationAdvanced,
    NegationParsed,
    BoundScanned(Option<(usize, usize)>),
    BoundAdvanced(usize),
    StringScanned(Option<usize>),
    IdentifierScanned(Option<usize>),
    Done,
}

/// Native intrinsics for generated/interpreted control programs, not a Pratt loop.
/// Fields are private: control code can only use checked state transitions.
pub struct Frame<'p, 's> {
    parser: &'p mut Parser<'s>,
    minimum: u8,
    depth: usize,
    left: Option<Value>,
    right: Option<Value>,
    operator: Option<Operator>,
    token_end: usize,
    consumed: bool,
    binding: Option<u8>,
    fuel: usize,
    primary_state: Option<PrimaryState>,
    in_primary: bool,
}

impl Frame<'_, '_> {
    pub fn step(&mut self) -> Result<(), Error> {
        self.fuel = self.fuel.checked_sub(1).ok_or(Error::ControlBudget)?;
        Ok(())
    }
    pub fn minimum(&self) -> u8 {
        self.minimum
    }
    pub fn has_operator(&self) -> bool {
        self.operator.is_some()
    }
    fn operator(&self) -> Result<&Operator, Error> {
        self.operator.as_ref().ok_or(Error::ControlState {
            reason: "no current operator",
        })
    }
    pub fn token_length(&self) -> Result<u8, Error> {
        u8::try_from(self.operator()?.token.len()).map_err(|_| Error::ControlState {
            reason: "operator token too long",
        })
    }
    pub fn precedence(&self) -> Result<u8, Error> {
        Ok(self.operator()?.precedence)
    }
    pub fn left_associative(&self) -> Result<bool, Error> {
        Ok(matches!(
            self.operator()?.associativity,
            Associativity::Left
        ))
    }
    pub fn binding_add(&self, left: u8, right: u8) -> Result<u8, Error> {
        left.checked_add(right).ok_or(Error::BindingPower)
    }
    fn expression_phase(&self) -> Result<(), Error> {
        if self.in_primary {
            return Err(Error::ControlState {
                reason: "expression operation during primary dispatch",
            });
        }
        Ok(())
    }
    pub fn set_binding_power(&mut self, value: u8) -> Result<(), Error> {
        self.expression_phase()?;
        self.binding = Some(value);
        Ok(())
    }
    pub fn primary(&mut self) -> Result<(), Error> {
        if self.left.is_some() || self.primary_state.is_some() {
            return Err(Error::ControlState {
                reason: "primary already started or parsed",
            });
        }
        self.primary_state = Some(PrimaryState::Start);
        self.in_primary = true;
        let result = match self.parser.primary_engine.clone() {
            Engine::Compiled(driver) => driver(self),
            Engine::Interpreted(program) => crate::pratt::interpret(&program, self),
        };
        self.in_primary = false;
        result?;
        if !matches!(self.primary_state, Some(PrimaryState::Done)) || self.left.is_none() {
            return Err(Error::ControlState {
                reason: "unfinished primary at return",
            });
        }
        Ok(())
    }
    fn primary_state_error<T>(&self) -> Result<T, Error> {
        Err(Error::ControlState {
            reason: "invalid primary stage",
        })
    }
    fn advance_primary(&mut self, end: usize) -> Result<(), Error> {
        if end <= self.parser.offset
            || end > self.parser.source.len()
            || !self.parser.source.is_char_boundary(end)
        {
            return self.primary_state_error();
        }
        self.parser.offset = end;
        Ok(())
    }
    pub fn whitespace(&mut self) -> Result<(), Error> {
        if !matches!(
            self.primary_state,
            Some(PrimaryState::Start | PrimaryState::BoundAdvanced(_))
        ) {
            return self.primary_state_error();
        }
        self.parser.whitespace()
    }
    pub fn scan_group(&mut self) -> Result<(), Error> {
        if !matches!(self.primary_state, Some(PrimaryState::Start)) {
            return self.primary_state_error();
        }
        let scan = crate::scanner::group_open(self.parser.source, self.parser.offset)?;
        self.primary_state = Some(PrimaryState::GroupScanned(scan.flag.then_some(scan.offset)));
        Ok(())
    }
    pub fn group_open(&self) -> Result<bool, Error> {
        match self.primary_state {
            Some(PrimaryState::GroupScanned(end)) => Ok(end.is_some()),
            _ => self.primary_state_error(),
        }
    }
    pub fn advance_group(&mut self) -> Result<(), Error> {
        let Some(PrimaryState::GroupScanned(Some(end))) = self.primary_state else {
            return self.primary_state_error();
        };
        self.advance_primary(end)?;
        self.primary_state = Some(PrimaryState::GroupAdvanced);
        Ok(())
    }
    pub fn parse_group(&mut self, minimum: u8) -> Result<(), Error> {
        if !matches!(self.primary_state, Some(PrimaryState::GroupAdvanced)) {
            return self.primary_state_error();
        }
        self.left = Some(self.parser.expression(minimum, self.depth + 1)?);
        self.primary_state = Some(PrimaryState::GroupParsed);
        Ok(())
    }
    pub fn close_group(&mut self) -> Result<(), Error> {
        if !matches!(self.primary_state, Some(PrimaryState::GroupParsed)) {
            return self.primary_state_error();
        }
        let scan = crate::scanner::group_close(self.parser.source, self.parser.offset)?;
        self.parser.offset = scan.offset;
        self.primary_state = Some(PrimaryState::Done);
        Ok(())
    }
    pub fn scan_negation(&mut self) -> Result<(), Error> {
        if !matches!(self.primary_state, Some(PrimaryState::GroupScanned(_))) {
            return self.primary_state_error();
        }
        let scan = crate::scanner::negation_prefix(self.parser.source, self.parser.offset)?;
        let candidate =
            (scan.flag && self.parser.grammar.negation.is_some()).then_some(scan.offset);
        self.primary_state = Some(PrimaryState::NegationScanned(candidate));
        Ok(())
    }
    pub fn has_negation(&self) -> Result<bool, Error> {
        match self.primary_state {
            Some(PrimaryState::NegationScanned(end)) => Ok(end.is_some()),
            _ => self.primary_state_error(),
        }
    }
    pub fn advance_negation(&mut self) -> Result<(), Error> {
        let Some(PrimaryState::NegationScanned(Some(end))) = self.primary_state else {
            return self.primary_state_error();
        };
        self.advance_primary(end)?;
        self.primary_state = Some(PrimaryState::NegationAdvanced);
        Ok(())
    }
    pub fn parse_negation(&mut self, minimum: u8) -> Result<(), Error> {
        if !matches!(self.primary_state, Some(PrimaryState::NegationAdvanced)) {
            return self.primary_state_error();
        }
        self.left = Some(self.parser.expression(minimum, self.depth + 1)?);
        self.primary_state = Some(PrimaryState::NegationParsed);
        Ok(())
    }
    pub fn build_negation(&mut self) -> Result<(), Error> {
        if !matches!(self.primary_state, Some(PrimaryState::NegationParsed)) {
            return self.primary_state_error();
        }
        self.parser.node()?;
        let operand = self.left.take().ok_or(Error::ControlState {
            reason: "missing negation operand",
        })?;
        let negation = self
            .parser
            .grammar
            .negation
            .as_ref()
            .ok_or(Error::ControlState {
                reason: "missing negation constructor",
            })?;
        self.left = Some(ast::unary(
            negation.kind.clone(),
            negation.operand_field.clone(),
            operand,
        ));
        self.primary_state = Some(PrimaryState::Done);
        Ok(())
    }
    pub fn scan_bound(&mut self) -> Result<(), Error> {
        if !matches!(
            self.primary_state,
            Some(PrimaryState::GroupScanned(_) | PrimaryState::NegationScanned(_))
        ) {
            return self.primary_state_error();
        }
        let scan = crate::scanner::bound_prefix(self.parser.source, self.parser.offset)?;
        let spelling = &self.parser.source[self.parser.offset..scan.offset];
        let candidate = self
            .parser
            .grammar
            .bounds
            .iter()
            .position(|bound| scan.flag && bound.token == spelling)
            .map(|index| (index, scan.offset));
        self.primary_state = Some(PrimaryState::BoundScanned(candidate));
        Ok(())
    }
    pub fn has_bound(&self) -> Result<bool, Error> {
        match self.primary_state {
            Some(PrimaryState::BoundScanned(candidate)) => Ok(candidate.is_some()),
            _ => self.primary_state_error(),
        }
    }
    pub fn advance_bound(&mut self) -> Result<(), Error> {
        let Some(PrimaryState::BoundScanned(Some((index, end)))) = self.primary_state else {
            return self.primary_state_error();
        };
        self.advance_primary(end)?;
        self.primary_state = Some(PrimaryState::BoundAdvanced(index));
        Ok(())
    }
    pub fn parse_bound(&mut self) -> Result<(), Error> {
        let Some(PrimaryState::BoundAdvanced(index)) = self.primary_state else {
            return self.primary_state_error();
        };
        let bound = self.parser.grammar.bounds[index].clone();
        let value = self.parser.decimal()?;
        self.parser.node()?;
        self.left = Some(ast::integer(bound.kind, bound.value_field, value));
        self.primary_state = Some(PrimaryState::Done);
        Ok(())
    }
    pub fn scan_string(&mut self) -> Result<(), Error> {
        if !matches!(self.primary_state, Some(PrimaryState::BoundScanned(_))) {
            return self.primary_state_error();
        }
        let scan = crate::scanner::string_literal(self.parser.source, self.parser.offset)?;
        let candidate = (scan.flag && self.parser.grammar.text.is_some()).then_some(scan.offset);
        self.primary_state = Some(PrimaryState::StringScanned(candidate));
        Ok(())
    }
    pub fn has_string(&self) -> Result<bool, Error> {
        match self.primary_state {
            Some(PrimaryState::StringScanned(end)) => Ok(end.is_some()),
            _ => self.primary_state_error(),
        }
    }
    pub fn parse_string(&mut self) -> Result<(), Error> {
        let Some(PrimaryState::StringScanned(Some(end))) = self.primary_state else {
            return self.primary_state_error();
        };
        let start = self.parser.offset;
        let Some(slice) = self.parser.source.get(start..end).filter(|_| end > start) else {
            return self.primary_state_error();
        };
        let value = serde_json::from_str::<String>(slice).map_err(|_| Error::Source {
            offset: start,
            reason: "invalid string literal".to_owned(),
        })?;
        let text = self
            .parser
            .grammar
            .text
            .as_ref()
            .ok_or(Error::ControlState {
                reason: "missing string constructor",
            })?;
        let kind = text.kind.clone();
        let field = text.value_field.clone();
        self.parser.node()?;
        self.advance_primary(end)?;
        self.left = Some(ast::text(kind, field, value));
        self.primary_state = Some(PrimaryState::Done);
        Ok(())
    }
    pub fn scan_identifier(&mut self) -> Result<(), Error> {
        if !matches!(
            self.primary_state,
            Some(PrimaryState::BoundScanned(_) | PrimaryState::StringScanned(_))
        ) {
            return self.primary_state_error();
        }
        let scan = crate::scanner::identifier(self.parser.source, self.parser.offset)?;
        self.primary_state = Some(PrimaryState::IdentifierScanned(
            scan.flag.then_some(scan.offset),
        ));
        Ok(())
    }
    pub fn has_identifier(&self) -> Result<bool, Error> {
        match self.primary_state {
            Some(PrimaryState::IdentifierScanned(end)) => Ok(end.is_some()),
            _ => self.primary_state_error(),
        }
    }
    pub fn parse_atom(&mut self) -> Result<(), Error> {
        let Some(PrimaryState::IdentifierScanned(Some(end))) = self.primary_state else {
            return self.primary_state_error();
        };
        self.left = Some(self.parser.atom(end)?);
        self.primary_state = Some(PrimaryState::Done);
        Ok(())
    }
    pub fn parse_integer(&mut self) -> Result<(), Error> {
        if !matches!(self.primary_state, Some(PrimaryState::IdentifierScanned(_))) {
            return self.primary_state_error();
        }
        self.left = Some(self.parser.integer()?);
        self.primary_state = Some(PrimaryState::Done);
        Ok(())
    }
    pub fn after_operand(&mut self) -> Result<(), Error> {
        self.expression_phase()?;
        if self.left.is_none() {
            return Err(Error::ControlState {
                reason: "operand not parsed",
            });
        }
        self.parser.after_operand()
    }
    pub fn lookup_operator(&mut self) -> Result<(), Error> {
        self.expression_phase()?;
        if self.left.is_none() || self.consumed {
            return Err(Error::ControlState {
                reason: "operator lookup out of order",
            });
        }
        let token = crate::scanner::infix(self.parser.source, self.parser.offset)?;
        let spelling = &self.parser.source[self.parser.offset..token.offset];
        self.operator = self
            .parser
            .grammar
            .operators
            .iter()
            .find(|op| token.flag && op.token == spelling)
            .cloned();
        self.token_end = token.offset;
        self.binding = None;
        Ok(())
    }
    pub fn advance_operator(&mut self) -> Result<(), Error> {
        self.expression_phase()?;
        self.operator()?;
        if self.consumed || self.token_end <= self.parser.offset {
            return Err(Error::ControlState {
                reason: "operator already consumed or not advancing",
            });
        }
        self.parser.offset = self.token_end;
        self.consumed = true;
        Ok(())
    }
    pub fn after_word_operator(&mut self) -> Result<(), Error> {
        self.expression_phase()?;
        if !self.consumed || self.right.is_some() {
            return Err(Error::ControlState {
                reason: "operator trivia out of order",
            });
        }
        let scan = crate::scanner::after_word_operator(self.parser.source, self.parser.offset)?;
        self.parser.offset = scan.offset;
        Ok(())
    }
    pub fn parse_right(&mut self) -> Result<(), Error> {
        self.expression_phase()?;
        if !self.consumed || self.right.is_some() {
            return Err(Error::ControlState {
                reason: "right parse out of order",
            });
        }
        let minimum = self.binding.ok_or(Error::ControlState {
            reason: "binding power not set",
        })?;
        self.right = Some(self.parser.expression(minimum, self.depth + 1)?);
        Ok(())
    }
    pub fn build_binary(&mut self) -> Result<(), Error> {
        self.expression_phase()?;
        let op = self.operator.take().ok_or(Error::ControlState {
            reason: "no current operator",
        })?;
        let left = self.left.take().ok_or(Error::ControlState {
            reason: "missing left operand",
        })?;
        let right = self.right.take().ok_or(Error::ControlState {
            reason: "missing right operand",
        })?;
        self.parser.node()?;
        self.left = Some(ast::binary(
            op.kind,
            op.left_field,
            op.right_field,
            left,
            right,
        ));
        self.consumed = false;
        self.binding = None;
        Ok(())
    }
}

#[cfg(test)]
mod string_state_tests {
    use super::*;

    #[test]
    fn checked_stages_and_spans() -> Result<(), Box<dyn std::error::Error>> {
        let grammar: Grammar = serde_json::from_str(
            r#"{
            "formatVersion":1,"integer":{"kind":"integer","valueField":"value"},
            "text":{"kind":"string","valueField":"value"},
            "group":{"open":"(","close":")"},"operators":[]
        }"#,
        )?;
        let mut parser = Parser {
            source: "\"λ\"",
            grammar: &grammar,
            offset: 0,
            nodes: 0,
            engine: Engine::Compiled(generated::execute),
            primary_engine: Engine::Compiled(generated_primary::execute),
        };
        let mut frame = Frame {
            parser: &mut parser,
            minimum: 0,
            depth: 0,
            left: None,
            right: None,
            operator: None,
            token_end: 0,
            consumed: false,
            binding: None,
            fuel: 4096,
            primary_state: Some(PrimaryState::Start),
            in_primary: true,
        };
        assert!(matches!(
            frame.has_string(),
            Err(Error::ControlState { .. })
        ));
        assert!(matches!(
            frame.scan_string(),
            Err(Error::ControlState { .. })
        ));
        assert!(matches!(
            frame.parse_string(),
            Err(Error::ControlState { .. })
        ));
        for end in [0, 2, 5] {
            frame.primary_state = Some(PrimaryState::StringScanned(Some(end)));
            assert!(matches!(
                frame.parse_string(),
                Err(Error::ControlState { .. })
            ));
            assert_eq!(frame.parser.offset, 0);
            assert_eq!(frame.parser.nodes, 0);
        }
        frame.primary_state = Some(PrimaryState::StringScanned(Some(3)));
        assert!(matches!(
            frame.parse_string(),
            Err(Error::Source { offset: 0, .. })
        ));
        assert_eq!(frame.parser.offset, 0);
        assert_eq!(frame.parser.nodes, 0);
        frame.primary_state = Some(PrimaryState::StringScanned(Some(4)));
        assert!(frame.has_string()?);
        frame.parse_string()?;
        assert!(matches!(frame.primary_state, Some(PrimaryState::Done)));
        assert_eq!(frame.parser.nodes, 1);
        assert_eq!(frame.parser.offset, 4);
        assert!(matches!(
            frame.parse_string(),
            Err(Error::ControlState { .. })
        ));
        assert!(matches!(
            frame.lookup_operator(),
            Err(Error::ControlState { .. })
        ));
        Ok(())
    }
}
