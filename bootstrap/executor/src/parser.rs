//! Narrow bootstrap Pratt parser. Token scanning and lexical policies are
//! CUE-generated, as are AST construction and the Pratt expression loop.
//! Primary dispatch, grammar lookup, conversion and frame safety remain native.

use std::collections::BTreeSet;
use std::sync::Arc;

#[path = "../../generated/pratt.rs"]
mod generated;

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
    atoms: Vec<Atom>,
    #[serde(default)]
    bounds: Vec<BoundPrefix>,
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
            if !matches!(op.token.as_str(), "+" | "*" | "/" | "&")
                || !tokens.insert(&op.token)
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
    parse_with_driver(source, grammar, generated::execute)
}

/// Trusted native driver hook for comparing compiled CUE control programs.
pub type Driver = for<'p, 's> fn(&mut Frame<'p, 's>) -> Result<(), Error>;

#[derive(Clone)]
enum Engine {
    Compiled(Driver),
    Interpreted(Arc<crate::pratt::Program>),
}

pub fn parse_with_driver(source: &str, grammar: &str, driver: Driver) -> Result<Value, Error> {
    parse_engine(source, grammar, Engine::Compiled(driver))
}

pub fn parse_interpreted(source: &str, grammar: &str, program: &str) -> Result<Value, Error> {
    let program = crate::pratt::parse(program).context(ControlSnafu)?;
    parse_engine(source, grammar, Engine::Interpreted(Arc::new(program)))
}

fn parse_engine(source: &str, grammar: &str, engine: Engine) -> Result<Value, Error> {
    let grammar: Grammar = serde_json::from_str(grammar).context(JsonSnafu)?;
    grammar.validate()?;
    let mut parser = Parser {
        source,
        grammar: &grammar,
        offset: 0,
        nodes: 0,
        engine,
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

    fn primary(&mut self, depth: usize) -> Result<Value, Error> {
        self.whitespace()?;
        let group = crate::scanner::group_open(self.source, self.offset)?;
        let value = if group.flag {
            self.offset = group.offset;
            let value = self.expression(0, depth + 1)?;
            let close = crate::scanner::group_close(self.source, self.offset)?;
            self.offset = close.offset;
            value
        } else {
            let prefix = crate::scanner::bound_prefix(self.source, self.offset)?;
            let spelling = &self.source[self.offset..prefix.offset];
            let bound = self
                .grammar
                .bounds
                .iter()
                .find(|bound| prefix.flag && bound.token == spelling)
                .cloned();
            if let Some(bound) = bound {
                self.offset = prefix.offset;
                self.whitespace()?;
                let value = self.decimal()?;
                self.node()?;
                ast::integer(bound.kind, bound.value_field, value)
            } else {
                let identifier = crate::scanner::identifier(self.source, self.offset)?;
                if identifier.flag {
                    self.atom(identifier.offset)?
                } else {
                    self.integer()?
                }
            }
        };
        Ok(value)
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
    pub fn set_binding_power(&mut self, value: u8) {
        self.binding = Some(value);
    }
    pub fn primary(&mut self) -> Result<(), Error> {
        if self.left.is_some() {
            return Err(Error::ControlState {
                reason: "primary already parsed",
            });
        }
        self.left = Some(self.parser.primary(self.depth)?);
        Ok(())
    }
    pub fn after_operand(&mut self) -> Result<(), Error> {
        if self.left.is_none() {
            return Err(Error::ControlState {
                reason: "operand not parsed",
            });
        }
        self.parser.after_operand()
    }
    pub fn lookup_operator(&mut self) -> Result<(), Error> {
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
    pub fn parse_right(&mut self) -> Result<(), Error> {
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
