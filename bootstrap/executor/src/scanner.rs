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
#[path = "../../generated/after-word-operator.rs"]
mod generated_after_word_operator;

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

#[cfg(feature = "source")]
#[path = "../../generated/negation-prefix.rs"]
mod generated_negation_prefix;

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
    #[snafu(display("scanner cannot advance a character at EOF"))]
    AdvanceChar,
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

/// Native boundary: validated cursor, checked advancement, one flag and fuel.
/// Each emitted/interpreted statement or loop test consumes one step.
pub struct Cursor<'a> {
    source: &'a str,
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
            source,
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
        self.source
            .as_bytes()
            .get(self.offset.checked_add(ahead)?)
            .copied()
    }

    pub fn peek_char(&self) -> Option<char> {
        self.source[self.offset..].chars().next()
    }

    pub fn advance_char(&mut self) -> Result<(), Error> {
        let character = self.peek_char().ok_or(Error::AdvanceChar)?;
        self.offset += character.len_utf8();
        Ok(())
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
    #[serde(rename = "charIn")]
    CharIn { characters: Vec<char> },
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
            Self::CharIn { characters } if characters.is_empty() || characters.len() > 128 => {
                ProgramSnafu {
                    reason: "charIn requires 1..=128 characters",
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
            Self::CharIn { characters } => cursor
                .peek_char()
                .is_some_and(|ch| characters.contains(&ch)),
            Self::FlagIs { value } => cursor.flag() == *value,
            Self::HasByte {} => cursor.peek().is_some(),
        }
    }

    fn inverted_rust(&self) -> String {
        match self {
            Self::ByteIn { .. } | Self::CharIn { .. } => format!("!{}", self.rust()),
            Self::FlagIs { value } => Self::FlagIs { value: !value }.rust(),
            Self::HasByte {} => "cursor.peek().is_none()".into(),
        }
    }

    fn rust(&self) -> String {
        let (bytes, ahead) = match self {
            Self::ByteIn { bytes, ahead } => (bytes, ahead),
            Self::CharIn { characters } => {
                let mut characters = characters.clone();
                characters.sort_unstable();
                characters.dedup();
                let patterns = characters
                    .iter()
                    .map(|ch| format!("{ch:?}"))
                    .collect::<Vec<_>>()
                    .join(" | ");
                return format!("matches!(cursor.peek_char(), Some({patterns}))");
            }
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
    #[serde(rename = "advanceChar")]
    AdvanceChar {},
    #[serde(rename = "break")]
    Break {},
    #[serde(rename = "setFlag")]
    SetFlag { value: bool },
    #[serde(rename = "reject")]
    Reject { reason: String },
}

fn validate(
    body: &[Statement],
    depth: usize,
    loops: usize,
    nodes: &mut usize,
) -> Result<(), Error> {
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
                validate(body, depth + 1, loops + 1, nodes)?;
            }
            Statement::If {
                condition,
                then,
                otherwise,
            } => {
                condition.validate()?;
                validate(then, depth + 1, loops, nodes)?;
                validate(otherwise, depth + 1, loops, nodes)?;
            }
            Statement::Reject { reason } if reason.is_empty() => {
                return ProgramSnafu {
                    reason: "reject requires a nonempty reason",
                }
                .fail();
            }
            Statement::Break {} if loops == 0 => {
                return ProgramSnafu {
                    reason: "break requires an enclosing loop",
                }
                .fail();
            }
            Statement::Advance {}
            | Statement::AdvanceChar {}
            | Statement::Break {}
            | Statement::SetFlag { .. }
            | Statement::Reject { .. } => {}
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
    validate(&program.body, 0, 0, &mut 0)?;
    Ok(program)
}

fn execute(body: &[Statement], cursor: &mut Cursor<'_>) -> Result<bool, Error> {
    for statement in body {
        match statement {
            Statement::While { condition, body } => {
                while {
                    cursor.step()?;
                    condition.test(cursor)
                } {
                    if execute(body, cursor)? {
                        break;
                    }
                }
            }
            Statement::If {
                condition,
                then,
                otherwise,
            } => {
                cursor.step()?;
                if execute(
                    if condition.test(cursor) {
                        then
                    } else {
                        otherwise
                    },
                    cursor,
                )? {
                    return Ok(true);
                }
            }
            Statement::Advance {} => {
                cursor.step()?;
                cursor.advance()?;
            }
            Statement::AdvanceChar {} => {
                cursor.step()?;
                cursor.advance_char()?;
            }
            Statement::Break {} => {
                cursor.step()?;
                return Ok(true);
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
    Ok(false)
}

/// Reference execution of a validated JSON statement program.
pub fn interpret(input: &str, source: &str, offset: usize) -> Result<Scan, Error> {
    let program = parse(input)?;
    let mut cursor = Cursor::new(source, offset)?;
    execute(&program.body, &mut cursor)?;
    Ok(cursor.finish())
}

// True only when every path reaching the end of this sequence instead breaks
// the current loop. Nested loops consume their own breaks and may fall through.
fn always_breaks(body: &[Statement]) -> bool {
    body.iter().any(|statement| match statement {
        Statement::Break {} => true,
        Statement::If {
            then, otherwise, ..
        } => always_breaks(then) && always_breaks(otherwise),
        _ => false,
    })
}

fn emit(body: &[Statement], output: &mut String, depth: usize, break_label: Option<usize>) -> bool {
    for statement in body {
        match statement {
            Statement::While { condition, body } => {
                if always_breaks(body) {
                    // A guaranteed break means there is exactly one condition test.
                    output.push_str(&format!(
                        "cursor.step()?;\nif {} {{\n'scan{depth}: {{\n",
                        condition.rust()
                    ));
                    emit(body, output, depth + 1, Some(depth));
                    output.push_str("}\n}\n");
                } else {
                    output.push_str(&format!(
                        "while {{ cursor.step()?; {} }} {{\n",
                        condition.rust()
                    ));
                    emit(body, output, depth + 1, None);
                    output.push_str("}\n");
                }
            }
            Statement::If {
                condition,
                then,
                otherwise,
            } => {
                output.push_str("cursor.step()?;\n");
                let mut then_code = String::new();
                let then_breaks = emit(then, &mut then_code, depth, break_label);
                let mut else_code = String::new();
                let else_breaks = emit(otherwise, &mut else_code, depth, break_label);
                // Predicates are pure reads, so identical branches need no test.
                if then_code == else_code {
                    output.push_str(&then_code);
                } else if then_code.is_empty() {
                    output.push_str(&format!("if {} {{\n", condition.inverted_rust()));
                    output.push_str(&else_code);
                    output.push_str("}\n");
                } else {
                    output.push_str(&format!("if {} {{\n", condition.rust()));
                    output.push_str(&then_code);
                    if !else_code.is_empty() {
                        output.push_str("} else {\n");
                        output.push_str(&else_code);
                    }
                    output.push_str("}\n");
                }
                if then_breaks && else_breaks {
                    return true;
                }
            }
            Statement::Advance {} => output.push_str("cursor.step()?;\ncursor.advance()?;\n"),
            Statement::AdvanceChar {} => {
                output.push_str("cursor.step()?;\ncursor.advance_char()?;\n")
            }
            Statement::Break {} => {
                output.push_str("cursor.step()?;\n");
                if let Some(label) = break_label {
                    output.push_str(&format!("break 'scan{label};\n"));
                } else {
                    output.push_str("break;\n");
                }
                return true;
            }
            Statement::SetFlag { value } => {
                output.push_str(&format!("cursor.step()?;\ncursor.set_flag({value});\n"))
            }
            Statement::Reject { reason } => {
                output.push_str(&format!("cursor.step()?;\ncursor.reject({reason:?})?;\n"))
            }
        }
    }
    false
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
    emit(&program.body, &mut output, 0, None);
    output.push_str("Ok(cursor.finish())\n}\n");
    Ok(output)
}

/// CUE-owned trivia/newline policy selected for word operators by parser control.
#[cfg(feature = "source")]
pub fn after_word_operator(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_after_word_operator::scan(source, offset)
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

/// Recognize minus before a non-digit; signed decimal literals stay atomic.
#[cfg(feature = "source")]
pub fn negation_prefix(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_negation_prefix::scan(source, offset)
}

/// Recognize an ASCII identifier; the flag indicates a valid first byte.
/// An invalid start leaves the cursor unchanged and the flag false.
#[cfg(feature = "source")]
pub fn identifier(source: &str, offset: usize) -> Result<Scan, Error> {
    generated_identifier::scan(source, offset)
}

#[cfg(test)]
mod tests {
    use super::{Cursor, Error, Predicate, Statement, generate, interpret, validate};

    #[test]
    fn unicode_cursor_preserves_boundaries_and_ascii_contract() {
        assert!(matches!(Cursor::new("λ", 1), Err(Error::Offset)));
        let mut cursor = Cursor::new("λ😀a", 0).expect("boundary");
        assert!(matches!(cursor.advance(), Err(Error::Advance)));
        assert_eq!(cursor.peek_char(), Some('λ'));
        cursor.advance_char().expect("lambda");
        cursor.advance_char().expect("emoji");
        assert_eq!(cursor.finish().offset, 6);
        let mut eof = Cursor::new("", 0).expect("EOF boundary");
        assert!(matches!(eof.advance_char(), Err(Error::AdvanceChar)));
    }

    #[test]
    fn char_predicates_validate_and_escape() {
        let program = r#"{"formatVersion":1,"body":[{"op":"if","condition":{"op":"charIn","characters":["λ","\n","'","λ"]},"then":[{"op":"advanceChar"}]}]}"#;
        assert_eq!(interpret(program, "λ!", 0).expect("scan").offset, 2);
        let code = generate(program).expect("generate");
        assert!(code.contains("Some('\\n' | '\\'' | 'λ')"));
        for values in [r#"[]"#, r#"[""]"#, r#"["ab"]"#] {
            let invalid = format!(
                r#"{{"formatVersion":1,"body":[{{"op":"if","condition":{{"op":"charIn","characters":{values}}},"then":[]}}]}}"#
            );
            assert!(generate(&invalid).is_err());
        }
        let values = vec!["x"; 129];
        let invalid = format!(
            r#"{{"formatVersion":1,"body":[{{"op":"if","condition":{{"op":"charIn","characters":{}}},"then":[]}}]}}"#,
            serde_json::to_string(&values).expect("serialize")
        );
        assert!(generate(&invalid).is_err());
    }

    #[test]
    fn break_is_scoped_and_unreachable_suffixes_are_not_emitted() {
        let invalid = r#"{"formatVersion":1,"body":[{"op":"if","condition":{"op":"hasByte"},"then":[{"op":"break"}]}]}"#;
        assert!(generate(invalid).is_err());
        let program = r#"{"formatVersion":1,"body":[{"op":"while","condition":{"op":"hasByte"},"body":[{"op":"while","condition":{"op":"hasByte"},"body":[{"op":"break"},{"op":"advance"}]},{"op":"if","condition":{"op":"charIn","characters":["λ"]},"then":[{"op":"break"}],"else":[{"op":"break"}]},{"op":"advance"}]},{"op":"setFlag","value":true}]}"#;
        let result = interpret(program, "λ", 0).expect("scan");
        assert_eq!(result.offset, 0);
        assert!(result.flag);
        let code = generate(program).expect("generate");
        assert!(!code.contains("cursor.advance()?"));
        assert!(code.contains("cursor.set_flag(true)"));
        assert!(!code.contains("while "));
        assert!(code.contains("break 'scan0;"));
        assert!(code.contains("break 'scan1;"));
        assert!(!code.contains("cursor.peek_char()")); // identical branches
        assert_eq!(code.matches("cursor.step()?").count(), 6);
        let conditional = r#"{"formatVersion":1,"body":[{"op":"while","condition":{"op":"hasByte"},"body":[{"op":"if","condition":{"op":"flagIs","value":true},"then":[{"op":"break"}]},{"op":"advance"}]}]}"#;
        let code = generate(conditional).expect("generate regular loop");
        assert!(code.contains("while "));
        assert!(code.contains("break;"));
        let looping =
            r#"{"formatVersion":1,"body":[{"op":"while","condition":{"op":"hasByte"},"body":[]}]}"#;
        assert!(matches!(interpret(looping, "x", 0), Err(Error::Budget)));
    }

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
        validate(&body, 0, 0, &mut 0).expect("64 statements allowed");
        body.push(Statement::Advance {});
        assert!(
            validate(&body, 0, 0, &mut 0)
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
        validate(&body, 0, 0, &mut 0).expect("16 nesting levels allowed");
        body = vec![Statement::While {
            condition: Predicate::ByteIn {
                bytes: vec![32],
                ahead: 0,
            },
            body,
        }];
        assert!(
            validate(&body, 0, 0, &mut 0)
                .expect_err("nesting cap")
                .to_string()
                .contains("16 nesting levels")
        );
    }
}
