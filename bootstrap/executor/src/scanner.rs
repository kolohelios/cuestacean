//! Small executable scanner DSL. The interpreter, emitter and safe cursor are
//! native scaffolding; scanner algorithms are CUE-authored statement programs.

use serde::Deserialize;
use snafu::{ResultExt, Snafu};

#[cfg(feature = "source")]
#[path = "../../generated/whitespace.rs"]
mod generated;

#[cfg(feature = "source")]
#[path = "../../generated/decimal.rs"]
mod generated_decimal;

#[cfg(feature = "source")]
#[path = "../../generated/identifier.rs"]
mod generated_identifier;

#[cfg(feature = "source")]
#[path = "../../generated/after-operand.rs"]
mod generated_after_operand;

#[cfg(feature = "source")]
#[path = "../../generated/group-open.rs"]
mod generated_group_open;

#[cfg(feature = "source")]
#[path = "../../generated/group-close.rs"]
mod generated_group_close;

#[cfg(feature = "source")]
#[path = "../../generated/bound-prefix.rs"]
mod generated_bound_prefix;

#[cfg(feature = "source")]
#[path = "../../generated/infix.rs"]
mod generated_infix;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("invalid scanner JSON: {source}"))]
    Json { source: serde_json::Error },
    #[snafu(display("invalid scanner program: {reason}"))]
    Program { reason: &'static str },
    #[snafu(display("scanner offset is not a source character boundary"))]
    Offset,
    #[snafu(display("scanner cannot advance at EOF or over a non-ASCII byte"))]
    Advance,
    #[snafu(display("scanner execution budget exhausted"))]
    Budget,
    #[snafu(display("source byte {offset}: {reason}"))]
    Rejected { offset: usize, reason: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scan {
    pub offset: usize,
    pub flag: bool,
}

/// Native boundary: validated cursor, ASCII-only advancement, one flag and fuel.
/// Each emitted/interpreted statement or loop test consumes one step.
pub struct Cursor<'a> {
    source: &'a [u8],
    offset: usize,
    flag: bool,
    fuel: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(source: &'a str, offset: usize) -> Result<Self, Error> {
        if !source.is_char_boundary(offset) {
            return Err(Error::Offset);
        }
        Ok(Self {
            source: source.as_bytes(),
            offset,
            flag: false,
            fuel: (source.len() - offset)
                .saturating_add(1)
                .saturating_mul(128),
        })
    }

    pub fn peek(&self) -> Option<u8> {
        self.peek_at(0)
    }

    pub fn peek_at(&self, ahead: usize) -> Option<u8> {
        self.source.get(self.offset.checked_add(ahead)?).copied()
    }

    pub fn step(&mut self) -> Result<(), Error> {
        self.fuel = self.fuel.checked_sub(1).ok_or(Error::Budget)?;
        Ok(())
    }

    pub fn advance(&mut self) -> Result<(), Error> {
        if !self.peek().is_some_and(|byte| byte.is_ascii()) {
            return Err(Error::Advance);
        }
        self.offset += 1;
        Ok(())
    }

    pub fn flag(&self) -> bool {
        self.flag
    }

    pub fn set_flag(&mut self, value: bool) {
        self.flag = value;
    }

    pub fn reject(&self, reason: &str) -> Result<(), Error> {
        Err(Error::Rejected {
            offset: self.offset,
            reason: reason.to_owned(),
        })
    }

    pub fn finish(self) -> Scan {
        Scan {
            offset: self.offset,
            flag: self.flag,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Program {
    format_version: u32,
    body: Vec<Statement>,
}

#[derive(Deserialize)]
#[serde(tag = "op", deny_unknown_fields)]
enum Predicate {
    #[serde(rename = "byteIn")]
    ByteIn {
        bytes: Vec<u8>,
        #[serde(default)]
        ahead: usize,
    },
    #[serde(rename = "flagIs")]
    FlagIs { value: bool },
    #[serde(rename = "hasByte")]
    HasByte {},
}

impl Predicate {
    fn validate(&self) -> Result<(), Error> {
        match self {
            Self::ByteIn { bytes, .. }
                if bytes.is_empty()
                    || bytes.len() > 128
                    || bytes.iter().any(|byte| !byte.is_ascii()) =>
            {
                ProgramSnafu {
                    reason: "byteIn requires 1..=128 ASCII bytes",
                }
                .fail()
            }
            Self::ByteIn { ahead, .. } if *ahead > 64 => ProgramSnafu {
                reason: "byteIn lookahead exceeds 64 bytes",
            }
            .fail(),
            _ => Ok(()),
        }
    }

    fn test(&self, cursor: &Cursor<'_>) -> bool {
        match self {
            Self::ByteIn { bytes, ahead } => cursor
                .peek_at(*ahead)
                .is_some_and(|byte| bytes.contains(&byte)),
            Self::FlagIs { value } => cursor.flag() == *value,
            Self::HasByte {} => cursor.peek().is_some(),
        }
    }

    fn inverted_rust(&self) -> String {
        match self {
            Self::ByteIn { .. } => format!("!{}", self.rust()),
            Self::FlagIs { value } => Self::FlagIs { value: !value }.rust(),
            Self::HasByte {} => "cursor.peek().is_none()".into(),
        }
    }

    fn rust(&self) -> String {
        let (bytes, ahead) = match self {
            Self::ByteIn { bytes, ahead } => (bytes, ahead),
            Self::FlagIs { value: true } => return "cursor.flag()".into(),
            Self::FlagIs { value: false } => return "!cursor.flag()".into(),
            Self::HasByte {} => return "cursor.peek().is_some()".into(),
        };
        let mut bytes = bytes.clone();
        bytes.sort_unstable();
        bytes.dedup();
        // Keep arbitrary byte sets, but compact runs to warning-free Rust ranges.
        let mut patterns = Vec::new();
        for run in bytes.chunk_by(|left, right| left.checked_add(1) == Some(*right)) {
            if run.len() >= 3 {
                patterns.push(format!("{}..={}", run[0], run[run.len() - 1]));
            } else {
                patterns.extend(run.iter().map(u8::to_string));
            }
        }
        let pattern = patterns.join(" | ");
        let peek = if *ahead == 0 {
            "cursor.peek()".into()
        } else {
            format!("cursor.peek_at({ahead})")
        };
        format!("matches!({peek}, Some({pattern}))")
    }
}

#[derive(Deserialize)]
#[serde(tag = "op", deny_unknown_fields)]
enum Statement {
    #[serde(rename = "while")]
    While {
        condition: Predicate,
        body: Vec<Statement>,
    },
    #[serde(rename = "if")]
    If {
        condition: Predicate,
        then: Vec<Statement>,
        #[serde(rename = "else", default)]
        otherwise: Vec<Statement>,
    },
    #[serde(rename = "advance")]
    Advance {},
    #[serde(rename = "setFlag")]
    SetFlag { value: bool },
    #[serde(rename = "reject")]
    Reject { reason: String },
}

fn validate(body: &[Statement], depth: usize, nodes: &mut usize) -> Result<(), Error> {
    if depth > 16 {
        return ProgramSnafu {
            reason: "exceeds 16 nesting levels",
        }
        .fail();
    }
    for statement in body {
        *nodes += 1;
        if *nodes > 64 {
            return ProgramSnafu {
                reason: "exceeds 64 statements",
            }
            .fail();
        }
        match statement {
            Statement::While { condition, body } => {
                condition.validate()?;
                validate(body, depth + 1, nodes)?;
            }
            Statement::If {
                condition,
                then,
                otherwise,
            } => {
                condition.validate()?;
                validate(then, depth + 1, nodes)?;
                validate(otherwise, depth + 1, nodes)?;
            }
            Statement::Reject { reason } if reason.is_empty() => {
                return ProgramSnafu {
                    reason: "reject requires a nonempty reason",
                }
                .fail();
            }
            Statement::Advance {} | Statement::SetFlag { .. } | Statement::Reject { .. } => {}
        }
    }
    Ok(())
}

fn parse(input: &str) -> Result<Program, Error> {
    let program: Program = serde_json::from_str(input).context(JsonSnafu)?;
    if program.format_version != 1 {
        return ProgramSnafu {
            reason: "formatVersion 1 required",
        }
        .fail();
    }
    validate(&program.body, 0, &mut 0)?;
    Ok(program)
}

fn execute(body: &[Statement], cursor: &mut Cursor<'_>) -> Result<(), Error> {
    for statement in body {
        match statement {
            Statement::While { condition, body } => {
                while {
                    cursor.step()?;
                    condition.test(cursor)
                } {
                    execute(body, cursor)?;
                }
            }
            Statement::If {
                condition,
                then,
                otherwise,
            } => {
                cursor.step()?;
                execute(
                    if condition.test(cursor) {
                        then
                    } else {
                        otherwise
                    },
                    cursor,
                )?;
            }
            Statement::Advance {} => {
                cursor.step()?;
                cursor.advance()?;
            }
            Statement::SetFlag { value } => {
                cursor.step()?;
                cursor.set_flag(*value);
            }
            Statement::Reject { reason } => {
                cursor.step()?;
                cursor.reject(reason)?;
            }
        }
    }
    Ok(())
}

/// Reference execution of a validated JSON statement program.
pub fn interpret(input: &str, source: &str, offset: usize) -> Result<Scan, Error> {
    let program = parse(input)?;
    let mut cursor = Cursor::new(source, offset)?;
    execute(&program.body, &mut cursor)?;
    Ok(cursor.finish())
}

fn emit(body: &[Statement], output: &mut String) {
    for statement in body {
        match statement {
            Statement::While { condition, body } => {
                output.push_str(&format!(
                    "while {{ cursor.step()?; {} }} {{\n",
                    condition.rust()
                ));
                emit(body, output);
                output.push_str("}\n");
            }
            Statement::If {
                condition,
                then,
                otherwise,
            } => {
                output.push_str("cursor.step()?;\n");
                // Empty branches still consume fuel, but need no Rust no-op.
                if then.is_empty() {
                    if !otherwise.is_empty() {
                        output.push_str(&format!("if {} {{\n", condition.inverted_rust()));
                        emit(otherwise, output);
                        output.push_str("}\n");
                    }
                } else {
                    output.push_str(&format!("if {} {{\n", condition.rust()));
                    emit(then, output);
                    if !otherwise.is_empty() {
                        output.push_str("} else {\n");
                        emit(otherwise, output);
                    }
                    output.push_str("}\n");
                }
            }
            Statement::Advance {} => output.push_str("cursor.step()?;\ncursor.advance()?;\n"),
            Statement::SetFlag { value } => {
                output.push_str(&format!("cursor.step()?;\ncursor.set_flag({value});\n"))
            }
            Statement::Reject { reason } => {
                output.push_str(&format!("cursor.step()?;\ncursor.reject({reason:?})?;\n"))
            }
        }
    }
}

/// Compile scanner control flow to Rust, using only the shared safe cursor.
pub fn generate(input: &str) -> Result<String, Error> {
    let program = parse(input)?;
    let mut output = String::from(
        "// Generated by bootstrap-scan-codegen. Do not edit by hand.\n\
         use crate::scanner::{Cursor, Error, Scan};\n\
         pub fn scan(source: &str, offset: usize) -> Result<Scan, Error> {\n",
    );
    let binding = if program.body.is_empty() {
        "cursor"
    } else {
        "mut cursor"
    };
    output.push_str(&format!("let {binding} = Cursor::new(source, offset)?;\n"));
    emit(&program.body, &mut output);
    output.push_str("Ok(cursor.finish())\n}\n");
    Ok(output)
}

/// Production whitespace scanner, compiled from the CUE statement program.
#[cfg(feature = "source")]
pub fn whitespace(source: &str, offset: usize) -> Result<Scan, Error> {
    generated::scan(source, offset)
}

/// Skip whitespace after an operand, rejecting a newline followed by more input.
#[cfg(feature = "source")]
pub fn after_operand(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_after_operand::scan(source, offset)
}

/// Recognize and consume an opening parenthesis; the flag records a match.
#[cfg(feature = "source")]
pub fn group_open(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_group_open::scan(source, offset)
}

/// Skip whitespace, require a closing parenthesis and consume it.
/// The result flag reports newlines in skipped whitespace, not token recognition.
#[cfg(feature = "source")]
pub fn group_close(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_group_close::scan(source, offset)
}

/// Recognize a complete inclusive-bound prefix; no match leaves offset unchanged.
#[cfg(feature = "source")]
pub fn bound_prefix(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_bound_prefix::scan(source, offset)
}

/// Recognize one byte from the bootstrap infix alphabet, independent of grammar lookup.
#[cfg(feature = "source")]
pub fn infix(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_infix::scan(source, offset)
}

/// Validate decimal spelling (digits required, no leading zeroes), not magnitude.
/// The result flag records whether the optional minus was consumed.
#[cfg(feature = "source")]
pub fn decimal(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_decimal::scan(source, offset)
}

/// Recognize an ASCII identifier; the flag indicates a valid first byte.
/// An invalid start leaves the cursor unchanged and the flag false.
#[cfg(feature = "source")]
pub fn identifier(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_identifier::scan(source, offset)
}

#[cfg(test)]
mod tests {
    use super::{Predicate, Statement, validate};

    #[test]
    fn byte_sets_compact_contiguous_runs_without_changing_sparse_patterns() {
        assert_eq!(
            Predicate::ByteIn {
                bytes: vec![13, 0, 1, 2, 4, 5, 10, 11, 12, 13],
                ahead: 0
            }
            .rust(),
            "matches!(cursor.peek(), Some(0..=2 | 4 | 5 | 10..=13))"
        );
        assert_eq!(
            Predicate::ByteIn {
                bytes: vec![32, 9, 10, 13],
                ahead: 0
            }
            .rust(),
            "matches!(cursor.peek(), Some(9 | 10 | 13 | 32))"
        );
    }

    #[test]
    fn statement_and_nesting_limits_are_checked_before_execution_or_emission() {
        let mut body: Vec<_> = (0..64).map(|_| Statement::Advance {}).collect();
        validate(&body, 0, &mut 0).expect("64 statements allowed");
        body.push(Statement::Advance {});
        assert!(
            validate(&body, 0, &mut 0)
                .expect_err("statement cap")
                .to_string()
                .contains("64 statements")
        );
        let mut body = vec![];
        for _ in 0..16 {
            body = vec![Statement::While {
                condition: Predicate::ByteIn {
                    bytes: vec![32],
                    ahead: 0,
                },
                body,
            }];
        }
        validate(&body, 0, &mut 0).expect("16 nesting levels allowed");
        body = vec![Statement::While {
            condition: Predicate::ByteIn {
                bytes: vec![32],
                ahead: 0,
            },
            body,
        }];
        assert!(
            validate(&body, 0, &mut 0)
                .expect_err("nesting cap")
                .to_string()
                .contains("16 nesting levels")
        );
    }
}
