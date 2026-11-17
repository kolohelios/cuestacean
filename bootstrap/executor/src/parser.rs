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
    #[serde(default)]
    logical_not: Option<Negation>,
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
        if let Some(logical_not) = &self.logical_not
            && (logical_not.token != "!"
                || logical_not.kind.is_empty()
                || !field(&logical_not.operand_field))
        {
            return GrammarSnafu {
                reason: "invalid logical not configuration",
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
                "+" | "-"
                    | "*"
                    | "/"
                    | "&"
                    | "&&"
                    | "||"
                    | "=="
                    | "!="
                    | "<"
                    | "<="
                    | ">"
                    | ">="
                    | "div"
                    | "mod"
                    | "quo"
                    | "rem"
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

/// Parse the documented integer-literal subset into a version-1 expression tree.
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

    fn integer_literal(&mut self) -> Result<i64, Error> {
        let start = self.offset;
        let scan = crate::scanner::integer_literal(self.source, start)?;
        self.offset = scan.offset;
        decode_integer_literal(&self.source[start..self.offset])
            .context(IntegerSnafu { offset: start })
    }

    fn integer(&mut self) -> Result<Value, Error> {
        let value = self.integer_literal()?;
        self.node()?;
        Ok(ast::integer(
            self.grammar.integer.kind.clone(),
            self.grammar.integer.value_field.clone(),
            value,
        ))
    }
}

/// Convert a CUE-selected spelling; lexical admission belongs to the scanner.
fn decode_integer_literal(text: &str) -> Result<i64, std::num::ParseIntError> {
    let (negative, unsigned) = match text.strip_prefix('-') {
        Some(unsigned) => (true, unsigned),
        None => (false, text),
    };
    let prefixed = unsigned
        .strip_prefix("0x")
        .or_else(|| unsigned.strip_prefix("0X"))
        .map(|digits| (digits, 16))
        .or_else(|| unsigned.strip_prefix("0b").map(|digits| (digits, 2)))
        .or_else(|| unsigned.strip_prefix("0o").map(|digits| (digits, 8)));
    let Some((digits, radix)) = prefixed else {
        return text.parse::<i64>();
    };
    if negative {
        // Parse the signed digits directly so MIN never needs a positive i64.
        i64::from_str_radix(&format!("-{digits}"), radix)
    } else {
        i64::from_str_radix(digits, radix)
    }
}

#[derive(Clone, Copy)]
enum PrimaryState {
    Start,
    GroupScanned(Option<usize>),
    GroupAdvanced,
    GroupParsed,
    NotScanned(Option<usize>),
    NotAdvanced,
    NotParsed,
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
    pub fn scan_not(&mut self) -> Result<(), Error> {
        if !matches!(self.primary_state, Some(PrimaryState::GroupScanned(_))) {
            return self.primary_state_error();
        }
        let scan = crate::scanner::not_prefix(self.parser.source, self.parser.offset)?;
        let candidate =
            (scan.flag && self.parser.grammar.logical_not.is_some()).then_some(scan.offset);
        self.primary_state = Some(PrimaryState::NotScanned(candidate));
        Ok(())
    }
    pub fn has_not(&self) -> Result<bool, Error> {
        match self.primary_state {
            Some(PrimaryState::NotScanned(end)) => Ok(end.is_some()),
            _ => self.primary_state_error(),
        }
    }
    pub fn advance_not(&mut self) -> Result<(), Error> {
        let Some(PrimaryState::NotScanned(Some(end))) = self.primary_state else {
            return self.primary_state_error();
        };
        self.advance_primary(end)?;
        self.primary_state = Some(PrimaryState::NotAdvanced);
        Ok(())
    }
    pub fn parse_not(&mut self, minimum: u8) -> Result<(), Error> {
        if !matches!(self.primary_state, Some(PrimaryState::NotAdvanced)) {
            return self.primary_state_error();
        }
        self.left = Some(self.parser.expression(minimum, self.depth + 1)?);
        self.primary_state = Some(PrimaryState::NotParsed);
        Ok(())
    }
    pub fn build_not(&mut self) -> Result<(), Error> {
        if !matches!(self.primary_state, Some(PrimaryState::NotParsed)) {
            return self.primary_state_error();
        }
        self.parser.node()?;
        let operand = self.left.take().ok_or(Error::ControlState {
            reason: "missing logical not operand",
        })?;
        let constructor = self
            .parser
            .grammar
            .logical_not
            .as_ref()
            .ok_or(Error::ControlState {
                reason: "missing logical not constructor",
            })?;
        self.left = Some(ast::unary(
            constructor.kind.clone(),
            constructor.operand_field.clone(),
            operand,
        ));
        self.primary_state = Some(PrimaryState::Done);
        Ok(())
    }
    pub fn scan_negation(&mut self) -> Result<(), Error> {
        if !matches!(
            self.primary_state,
            Some(PrimaryState::GroupScanned(_) | PrimaryState::NotScanned(_))
        ) {
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
            Some(
                PrimaryState::GroupScanned(_)
                    | PrimaryState::NotScanned(_)
                    | PrimaryState::NegationScanned(_)
            )
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
        let value = self.parser.integer_literal()?;
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
        let value = decode_string_literal(slice).ok_or_else(|| Error::Source {
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

/// Checked conversion after CUE admission, not source lexical policy. Allocate a
/// JSON-compatible spelling so quote, backslash and control scalars stay escaped;
/// the JSON decoder still validates the complete literal and existing escapes.
fn decode_string_literal(slice: &str) -> Option<String> {
    use std::fmt::Write;

    if !slice.contains("\\U") {
        return serde_json::from_str(slice).ok();
    }
    let mut json = String::new();
    let mut offset = 0usize;
    while offset < slice.len() {
        let ch = slice.get(offset..)?.chars().next()?;
        offset = offset.checked_add(ch.len_utf8())?;
        if ch != '\\' {
            json.push(ch);
            continue;
        }
        // Consume ordinary escapes together, including a literal backslash, so
        // an escaped backslash cannot introduce a wide Unicode conversion.
        let escaped = slice.get(offset..)?.chars().next()?;
        offset = offset.checked_add(escaped.len_utf8())?;
        if escaped != 'U' {
            json.push('\\');
            json.push(escaped);
            continue;
        }
        let end = offset.checked_add(8)?;
        let digits = slice.get(offset..end)?;
        let mut value = 0u32;
        for digit in digits.bytes() {
            if !digit.is_ascii_hexdigit() {
                return None;
            }
            value = value
                .checked_mul(16)?
                .checked_add((digit as char).to_digit(16)?)?;
        }
        let scalar = char::from_u32(value)?;
        let mut units = [0u16; 2];
        for unit in scalar.encode_utf16(&mut units) {
            write!(json, "\\u{unit:04X}").ok()?;
        }
        offset = end;
    }
    serde_json::from_str(&json).ok()
}

#[cfg(test)]
mod integer_codec_tests {
    use super::decode_integer_literal;
    use std::num::IntErrorKind;

    #[test]
    fn fixed_radix_values_and_signed_extrema() -> Result<(), std::num::ParseIntError> {
        for (source, expected) in [
            ("0b101010", 42),
            ("0o52", 42),
            ("0x2a", 42),
            ("0X2A", 42),
            ("-0b101010", -42),
            ("-0o52", -42),
            ("-0x2A", -42),
            ("-0X2a", -42),
            ("0b0", 0),
            ("-0b0", 0),
            ("0o0", 0),
            ("-0o0", 0),
            ("0x0", 0),
            ("-0X0", 0),
            (
                "0b111111111111111111111111111111111111111111111111111111111111111",
                i64::MAX,
            ),
            (
                "-0b1000000000000000000000000000000000000000000000000000000000000000",
                i64::MIN,
            ),
            ("0o777777777777777777777", i64::MAX),
            ("-0o1000000000000000000000", i64::MIN),
            ("0x7fffffffffffffff", i64::MAX),
            ("-0x8000000000000000", i64::MIN),
            ("0X7FFFFFFFFFFFFFFF", i64::MAX),
            ("-0X8000000000000000", i64::MIN),
        ] {
            assert_eq!(decode_integer_literal(source)?, expected, "{source}");
        }
        Ok(())
    }

    #[test]
    fn standard_range_and_digit_errors() {
        for (source, expected) in [
            (
                "0b1000000000000000000000000000000000000000000000000000000000000000",
                IntErrorKind::PosOverflow,
            ),
            (
                "-0b1000000000000000000000000000000000000000000000000000000000000001",
                IntErrorKind::NegOverflow,
            ),
            ("0o1000000000000000000000", IntErrorKind::PosOverflow),
            ("-0o1000000000000000000001", IntErrorKind::NegOverflow),
            ("0x8000000000000000", IntErrorKind::PosOverflow),
            ("-0x8000000000000001", IntErrorKind::NegOverflow),
            ("0X8000000000000000", IntErrorKind::PosOverflow),
            ("-0X8000000000000001", IntErrorKind::NegOverflow),
            ("0b2", IntErrorKind::InvalidDigit),
            ("0o8", IntErrorKind::InvalidDigit),
            ("0xg", IntErrorKind::InvalidDigit),
            ("0B1", IntErrorKind::InvalidDigit),
            ("0O1", IntErrorKind::InvalidDigit),
            ("0x１", IntErrorKind::InvalidDigit),
            ("0b1_0", IntErrorKind::InvalidDigit),
            ("0x", IntErrorKind::Empty),
            ("-0o", IntErrorKind::InvalidDigit),
        ] {
            let error = decode_integer_literal(source).expect_err(source);
            assert_eq!(error.kind(), &expected, "{source}");
        }
    }

    #[test]
    fn decimal_conversion_remains_the_standard_fast_path() {
        // Conversion deliberately does not replace CUE lexical policy.
        for source in [
            "0",
            "-0",
            "42",
            "-42",
            "01",
            "+1",
            "",
            "-",
            "１",
            "1_0",
            "9223372036854775807",
            "-9223372036854775808",
            "9223372036854775808",
            "-9223372036854775809",
        ] {
            assert_eq!(
                decode_integer_literal(source),
                source.parse::<i64>(),
                "{source}"
            );
        }
    }
}

#[cfg(test)]
mod string_codec_tests {
    use super::decode_string_literal;

    #[test]
    fn fixed_scalar_oracles_and_json_sensitive_values() {
        for (source, expected) in [
            (r#""\U0001F980""#, "🦀"),
            (r#""\U00000022""#, "\""),
            (r#""\U0000005C""#, "\\"),
            (r#""\U0000002F""#, "/"),
            (r#""\U00000000\U00000008\U0000000C""#, "\0\u{8}\u{c}"),
            (r#""\U00000009\U0000000A\U0000000D""#, "\t\n\r"),
            (
                r#""\U0000feff\U0000FDD0\U0010ffff""#,
                "\u{feff}\u{fdd0}\u{10ffff}",
            ),
            (r#""λ\u0061\U0001f980\uD83E\uDD80\n""#, "λa🦀🦀\n"),
            (r#""\U000000220\U0000005C\U00000000λ""#, "\"0\\\0λ"),
            // Eight digits are consumed; following hex characters are text.
            (r#""\U000000410""#, "A0"),
        ] {
            assert_eq!(
                decode_string_literal(source).as_deref(),
                Some(expected),
                "{source}"
            );
        }
    }

    #[test]
    fn sampled_planes_and_scalar_boundaries() -> Result<(), Box<dyn std::error::Error>> {
        for plane in 0..=16u32 {
            for low in [0, 1, 0x7f, 0x80, 0x1234, 0xfffe, 0xffff] {
                let value = plane * 0x10000 + low;
                let scalar = char::from_u32(value).ok_or("invalid test scalar")?;
                let source = format!("\"\\U{value:08X}\"");
                assert_eq!(decode_string_literal(&source), Some(scalar.to_string()));
            }
        }
        for value in [0xd7ff, 0xe000, 0xffff, 0x10000, 0x10ffff] {
            let scalar = char::from_u32(value).ok_or("invalid test scalar")?;
            assert_eq!(
                decode_string_literal(&format!("\"\\U{value:08x}\"")),
                Some(scalar.to_string())
            );
        }
        Ok(())
    }

    #[test]
    fn backslash_parity_and_existing_escapes() {
        for count in 1..=8 {
            let source = format!("\"{}U0001F980\"", "\\".repeat(count));
            let suffix = if count % 2 == 0 { "U0001F980" } else { "🦀" };
            let expected = format!("{}{suffix}", "\\".repeat(count / 2));
            assert_eq!(decode_string_literal(&source), Some(expected));
        }
        for source in [
            r#""""#,
            r#""λ🦀""#,
            r#""\"\\\b\f\n\r\t""#,
            r#""\u0000\uFEFF\ud83e\udd80""#,
            r#""\\UFFFFFFFF""#,
            r#""\\U0000000λ""#,
            r#""\"\\U0001F980\"""#,
        ] {
            assert_eq!(
                decode_string_literal(source),
                serde_json::from_str::<String>(source).ok()
            );
        }
    }

    #[test]
    fn rejects_malformed_wide_and_json_context() {
        for digits in [
            "00110000",
            "FFFFFFFF",
            "0000D800",
            "0000DBFF",
            "0000DC00",
            "0000DFFF",
            "0000000g",
            "000000é",
            "００００００００",
        ] {
            let source = format!("\"\\U{digits}\"");
            assert_eq!(decode_string_literal(&source), None, "{source}");
        }
        for count in 0..8 {
            let digits = "0".repeat(count);
            for source in [format!("\"\\U{digits}"), format!("\"\\U{digits}\"")] {
                assert_eq!(decode_string_literal(&source), None, "{source}");
            }
        }
        for source in [
            "",
            "null",
            "42",
            "[]",
            "\"",
            "\"\\",
            "\"\n\"",
            r#""\U00000041"extra"#,
            r#""\U00000041"""#,
            r#""\U00000041" "x""#,
            r#""\U00000041"#,
            r#""\U00000041\q""#,
            r#""\uD800\U0000DC00""#,
            r#""\uD800\U0001F980""#,
            r#""\U00000041\uD800""#,
            r#""\U00000041\uDC00""#,
            r#""\U00000041\u12g4""#,
            "\"\\U00000041\0\"",
        ] {
            assert_eq!(decode_string_literal(source), None, "{source:?}");
        }
    }
}

#[cfg(test)]
mod string_state_tests {
    use super::*;

    #[test]
    fn logical_not_metadata_and_stages() -> Result<(), Box<dyn std::error::Error>> {
        let base = serde_json::json!({
            "formatVersion": 1, "integer": {"kind":"integer", "valueField":"value"},
            "group": {"open":"(", "close":")"},
            "operators": [{"token":"+", "kind":"add", "leftField":"left",
                "rightField":"right", "precedence":10, "associativity":"left"}]
        });
        for invalid in [
            serde_json::json!({"token":"-", "kind":"not", "operandField":"operand"}),
            serde_json::json!({"token":"!", "kind":"", "operandField":"operand"}),
            serde_json::json!({"token":"!", "kind":"not", "operandField":"kind"}),
            serde_json::json!({"token":"!", "kind":"not", "operandField":""}),
        ] {
            let mut value = base.clone();
            value["logicalNot"] = invalid;
            let grammar: Grammar = serde_json::from_value(value)?;
            assert!(matches!(grammar.validate(), Err(Error::Grammar { .. })));
        }
        let mut value = base;
        value["logicalNot"] =
            serde_json::json!({"token":"!", "kind":"not", "operandField":"operand"});
        let grammar: Grammar = serde_json::from_value(value)?;
        grammar.validate()?;
        let mut parser = Parser {
            source: "!λ",
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
        assert!(matches!(frame.has_not(), Err(Error::ControlState { .. })));
        assert!(matches!(frame.scan_not(), Err(Error::ControlState { .. })));
        assert!(matches!(
            frame.advance_not(),
            Err(Error::ControlState { .. })
        ));
        assert!(matches!(
            frame.parse_not(101),
            Err(Error::ControlState { .. })
        ));
        assert!(matches!(frame.build_not(), Err(Error::ControlState { .. })));
        for end in [0, 2, 4] {
            frame.primary_state = Some(PrimaryState::NotScanned(Some(end)));
            assert!(matches!(
                frame.advance_not(),
                Err(Error::ControlState { .. })
            ));
            assert_eq!(frame.parser.offset, 0);
        }
        frame.primary_state = Some(PrimaryState::GroupScanned(None));
        frame.scan_not()?;
        assert!(frame.has_not()?);
        frame.advance_not()?;
        assert_eq!(frame.parser.offset, 1);
        assert!(matches!(frame.build_not(), Err(Error::ControlState { .. })));
        frame.left = Some(serde_json::json!({"kind":"boolean"}));
        frame.primary_state = Some(PrimaryState::NotParsed);
        frame.build_not()?;
        assert_eq!(
            frame.left,
            Some(serde_json::json!({"kind":"not", "operand":{"kind":"boolean"}}))
        );
        assert_eq!(frame.parser.nodes, 1);
        assert!(matches!(frame.build_not(), Err(Error::ControlState { .. })));
        Ok(())
    }

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
