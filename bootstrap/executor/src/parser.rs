//! Narrow bootstrap Pratt parser. Whitespace and decimal scanning are CUE-generated;
//! validation, conversion, other lexing and Pratt parsing remain native scaffolding.

use std::collections::BTreeSet;

use serde::Deserialize;
use serde_json::{Map, Value};
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
    Source { offset: usize, reason: &'static str },
    #[snafu(display("source byte {offset}: signed 64-bit integer required: {source}"))]
    Integer {
        offset: usize,
        source: std::num::ParseIntError,
    },
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
    Ok(Value::Object(Map::from_iter([
        ("formatVersion".into(), 1.into()),
        ("entry".into(), entry),
    ])))
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

    fn whitespace(&mut self) -> Result<bool, Error> {
        let scan = crate::scanner::whitespace(self.source, self.offset).context(ScannerSnafu)?;
        self.offset = scan.offset;
        Ok(scan.flag)
    }

    fn peek(&self) -> Option<u8> {
        self.source.as_bytes().get(self.offset).copied()
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
        let mut left = if self.peek() == Some(self.grammar.group.open.as_bytes()[0]) {
            self.offset += 1;
            let value = self.expression(0, depth + 1)?;
            self.whitespace()?;
            if self.peek() != Some(self.grammar.group.close.as_bytes()[0]) {
                return self.error("expected closing parenthesis");
            }
            self.offset += 1;
            value
        } else if let Some(bound) = self
            .grammar
            .bounds
            .iter()
            .find(|bound| self.source[self.offset..].starts_with(&bound.token))
            .cloned()
        {
            self.offset += bound.token.len();
            self.whitespace()?;
            let value = self.decimal()?;
            self.node()?;
            Value::Object(Map::from_iter([
                ("kind".into(), bound.kind.into()),
                (bound.value_field, value.into()),
            ]))
        } else if self
            .peek()
            .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        {
            self.atom()?
        } else {
            self.integer()?
        };
        loop {
            // CUE inserts a comma after an operand at a line break. Do not
            // silently turn separate CUE declarations into one expression.
            if self.whitespace()? && self.peek().is_some() {
                return self.error("line break after operand is unsupported");
            }
            let op = self
                .grammar
                .operators
                .iter()
                .find(|op| Some(op.token.as_bytes()[0]) == self.peek())
                .cloned();
            let Some(op) = op else {
                break;
            };
            if op.precedence < minimum {
                break;
            }
            self.offset += 1;
            let next = match op.associativity {
                Associativity::Left => op.precedence + 1,
                Associativity::Right => op.precedence,
            };
            let right = self.expression(next, depth + 1)?;
            self.node()?;
            left = Value::Object(Map::from_iter([
                ("kind".into(), op.kind.into()),
                (op.left_field, left),
                (op.right_field, right),
            ]));
        }
        Ok(left)
    }

    fn atom(&mut self) -> Result<Value, Error> {
        let start = self.offset;
        while self
            .peek()
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            self.offset += 1;
        }
        let token = &self.source[start..self.offset];
        let kind = self
            .grammar
            .atoms
            .iter()
            .find(|atom| atom.token == token)
            .map(|atom| atom.kind.clone())
            .ok_or(Error::Source {
                offset: start,
                reason: "unknown atom",
            })?;
        self.node()?;
        Ok(Value::Object(Map::from_iter([(
            "kind".into(),
            kind.into(),
        )])))
    }

    fn decimal(&mut self) -> Result<i64, Error> {
        let start = self.offset;
        let scan = crate::scanner::decimal(self.source, start).context(ScannerSnafu)?;
        self.offset = scan.offset;
        let digits = start + usize::from(scan.flag);
        if self.offset <= digits {
            return self.error("expected signed decimal integer");
        }
        if self.offset - digits > 1 && self.source.as_bytes()[digits] == b'0' {
            return self.error("leading zeroes are unsupported");
        }
        self.source[start..self.offset]
            .parse::<i64>()
            .context(IntegerSnafu { offset: start })
    }

    fn integer(&mut self) -> Result<Value, Error> {
        let value = self.decimal()?;
        self.node()?;
        Ok(Value::Object(Map::from_iter([
            ("kind".into(), self.grammar.integer.kind.clone().into()),
            (self.grammar.integer.value_field.clone(), value.into()),
        ])))
    }
}
