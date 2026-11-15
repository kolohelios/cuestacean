//! Narrow bootstrap Pratt parser. Token scanning and lexical policies are
//! CUE-generated, as is AST construction. Conversion, grammar validation, lookup
//! and Pratt parsing remain native.

use std::collections::BTreeSet;

use serde::Deserialize;
use serde_json::Value;

use crate::ast;
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
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
    let grammar: Grammar = serde_json::from_str(grammar).context(JsonSnafu)?;
    grammar.validate()?;
    let mut parser = Parser {
        source,
        grammar: &grammar,
        offset: 0,
        nodes: 0,
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
        self.whitespace()?;
        let group = crate::scanner::group_open(self.source, self.offset)?;
        let mut left = if group.flag {
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
        loop {
            self.after_operand()?;
            let token = crate::scanner::infix(self.source, self.offset)?;
            let spelling = &self.source[self.offset..token.offset];
            let op = self
                .grammar
                .operators
                .iter()
                .find(|op| token.flag && op.token == spelling)
                .cloned();
            let Some(op) = op else {
                break;
            };
            if op.precedence < minimum {
                break;
            }
            self.offset = token.offset;
            let next = match op.associativity {
                Associativity::Left => op.precedence + 1,
                Associativity::Right => op.precedence,
            };
            let right = self.expression(next, depth + 1)?;
            self.node()?;
            left = ast::binary(op.kind, op.left_field, op.right_field, left, right);
        }
        Ok(left)
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
