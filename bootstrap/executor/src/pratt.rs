//! Executable expression-parser control flow. Frame operations are native;
//! the loop, binding-power comparison and associativity policy are CUE-authored.
use serde::Deserialize;
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("invalid Pratt control JSON: {source}"))]
    Json { source: serde_json::Error },
    #[snafu(display("invalid Pratt control: {reason}"))]
    Program { reason: &'static str },
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Program {
    format_version: u32,
    body: Vec<Statement>,
}
#[derive(Clone, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase", deny_unknown_fields)]
enum Number {
    Minimum {},
    TokenLength {},
    Precedence {},
    Integer {
        value: u8,
    },
    Add {
        left: Box<Number>,
        right: Box<Number>,
    },
}
#[derive(Clone, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase", deny_unknown_fields)]
enum Condition {
    HasOperator {},
    LeftAssociative {},
    GroupOpen {},
    HasBound {},
    HasIdentifier {},
    HasNegation {},
    Boolean {
        value: bool,
    },
    AtLeast {
        left: Number,
        right: Number,
    },
    And {
        left: Box<Condition>,
        right: Box<Condition>,
    },
}
#[derive(Clone, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase", deny_unknown_fields)]
enum Statement {
    Primary {},
    AfterOperand {},
    AfterWordOperator {},
    LookupOperator {},
    AdvanceOperator {},
    ParseRight {},
    BuildBinary {},
    Whitespace {},
    ScanGroup {},
    AdvanceGroup {},
    CloseGroup {},
    ScanBound {},
    AdvanceBound {},
    ParseBound {},
    ScanIdentifier {},
    ParseAtom {},
    ParseInteger {},
    ParseGroup {
        minimum: Number,
    },
    ScanNegation {},
    AdvanceNegation {},
    BuildNegation {},
    ParseNegation {
        minimum: Number,
    },
    SetBindingPower {
        value: Number,
    },
    While {
        condition: Condition,
        body: Vec<Statement>,
    },
    If {
        condition: Condition,
        then: Vec<Statement>,
        #[serde(rename = "else", default)]
        otherwise: Vec<Statement>,
    },
}
fn limits(value: &serde_json::Value, depth: usize, nodes: &mut usize) -> Result<(), Error> {
    *nodes += 1;
    if depth > 32 || *nodes > 512 {
        return ProgramSnafu {
            reason: "exceeds 32 JSON levels or 512 JSON nodes",
        }
        .fail();
    }
    match value {
        serde_json::Value::Array(values) => {
            for value in values {
                limits(value, depth + 1, nodes)?;
            }
        }
        serde_json::Value::Object(values) => {
            for value in values.values() {
                limits(value, depth + 1, nodes)?;
            }
        }
        _ => {}
    }
    Ok(())
}
pub(crate) fn parse(input: &str) -> Result<Program, Error> {
    let value: serde_json::Value = serde_json::from_str(input).context(JsonSnafu)?;
    limits(&value, 0, &mut 0)?;
    let program: Program = serde_json::from_value(value).context(JsonSnafu)?;
    if program.format_version != 1 {
        return ProgramSnafu {
            reason: "formatVersion 1 required",
        }
        .fail();
    }
    Ok(program)
}
impl Number {
    fn rust(&self) -> String {
        match self {
            Self::Minimum {} => "_frame.minimum()".into(),
            Self::TokenLength {} => "_frame.token_length()?".into(),
            Self::Precedence {} => "_frame.precedence()?".into(),
            Self::Integer { value } => format!("{value}u8"),
            Self::Add { left, right } => {
                format!("_frame.binding_add({}, {})?", left.rust(), right.rust())
            }
        }
    }
    #[cfg(feature = "source")]
    fn value(&self, frame: &crate::parser::Frame<'_, '_>) -> Result<u8, crate::parser::Error> {
        Ok(match self {
            Self::Minimum {} => frame.minimum(),
            Self::TokenLength {} => frame.token_length()?,
            Self::Precedence {} => frame.precedence()?,
            Self::Integer { value } => *value,
            Self::Add { left, right } => {
                frame.binding_add(left.value(frame)?, right.value(frame)?)?
            }
        })
    }
}
impl Condition {
    fn rust(&self) -> String {
        match self {
            Self::HasOperator {} => "_frame.has_operator()".into(),
            Self::LeftAssociative {} => "_frame.left_associative()?".into(),
            Self::GroupOpen {} => "_frame.group_open()?".into(),
            Self::HasBound {} => "_frame.has_bound()?".into(),
            Self::HasIdentifier {} => "_frame.has_identifier()?".into(),
            Self::HasNegation {} => "_frame.has_negation()?".into(),
            Self::Boolean { value } => value.to_string(),
            Self::AtLeast { left, right } => format!("{} >= {}", left.rust(), right.rust()),
            Self::And { left, right } => format!("{} && {}", left.rust(), right.rust()),
        }
    }
    #[cfg(feature = "source")]
    fn test(&self, frame: &crate::parser::Frame<'_, '_>) -> Result<bool, crate::parser::Error> {
        Ok(match self {
            Self::HasOperator {} => frame.has_operator(),
            Self::LeftAssociative {} => frame.left_associative()?,
            Self::GroupOpen {} => frame.group_open()?,
            Self::HasBound {} => frame.has_bound()?,
            Self::HasIdentifier {} => frame.has_identifier()?,
            Self::HasNegation {} => frame.has_negation()?,
            Self::Boolean { value } => *value,
            Self::AtLeast { left, right } => left.value(frame)? >= right.value(frame)?,
            Self::And { left, right } => left.test(frame)? && right.test(frame)?,
        })
    }
}
#[cfg(feature = "source")]
fn execute(
    body: &[Statement],
    frame: &mut crate::parser::Frame<'_, '_>,
) -> Result<(), crate::parser::Error> {
    for statement in body {
        match statement {
            Statement::While { condition, body } => {
                while {
                    frame.step()?;
                    condition.test(frame)?
                } {
                    execute(body, frame)?;
                }
            }
            _ => {
                frame.step()?;
                match statement {
                    Statement::Primary {} => frame.primary()?,
                    Statement::AfterOperand {} => frame.after_operand()?,
                    Statement::AfterWordOperator {} => frame.after_word_operator()?,
                    Statement::LookupOperator {} => frame.lookup_operator()?,
                    Statement::AdvanceOperator {} => frame.advance_operator()?,
                    Statement::ParseRight {} => frame.parse_right()?,
                    Statement::BuildBinary {} => frame.build_binary()?,
                    Statement::Whitespace {} => frame.whitespace()?,
                    Statement::ScanGroup {} => frame.scan_group()?,
                    Statement::ScanNegation {} => frame.scan_negation()?,
                    Statement::AdvanceNegation {} => frame.advance_negation()?,
                    Statement::BuildNegation {} => frame.build_negation()?,
                    Statement::ParseNegation { minimum } => {
                        frame.parse_negation(minimum.value(frame)?)?
                    }
                    Statement::AdvanceGroup {} => frame.advance_group()?,
                    Statement::CloseGroup {} => frame.close_group()?,
                    Statement::ScanBound {} => frame.scan_bound()?,
                    Statement::AdvanceBound {} => frame.advance_bound()?,
                    Statement::ParseBound {} => frame.parse_bound()?,
                    Statement::ScanIdentifier {} => frame.scan_identifier()?,
                    Statement::ParseAtom {} => frame.parse_atom()?,
                    Statement::ParseInteger {} => frame.parse_integer()?,
                    Statement::ParseGroup { minimum } => {
                        frame.parse_group(minimum.value(frame)?)?
                    }
                    Statement::SetBindingPower { value } => {
                        frame.set_binding_power(value.value(frame)?)?
                    }
                    Statement::If {
                        condition,
                        then,
                        otherwise,
                    } => execute(
                        if condition.test(frame)? {
                            then
                        } else {
                            otherwise
                        },
                        frame,
                    )?,
                    Statement::While { .. } => unreachable!(),
                }
            }
        }
    }
    Ok(())
}
#[cfg(feature = "source")]
pub(crate) fn interpret(
    program: &Program,
    frame: &mut crate::parser::Frame<'_, '_>,
) -> Result<(), crate::parser::Error> {
    execute(&program.body, frame)
}
fn emit(body: &[Statement], output: &mut String) {
    for statement in body {
        if let Statement::While { condition, body } = statement {
            output.push_str(&format!(
                "while {{ _frame.step()?; {} }} {{\n",
                condition.rust()
            ));
            emit(body, output);
            output.push_str("}\n");
            continue;
        }
        output.push_str("_frame.step()?;\n");
        match statement {
            Statement::Primary {} => output.push_str("_frame.primary()?;\n"),
            Statement::AfterOperand {} => output.push_str("_frame.after_operand()?;\n"),
            Statement::AfterWordOperator {} => output.push_str("_frame.after_word_operator()?;\n"),
            Statement::LookupOperator {} => output.push_str("_frame.lookup_operator()?;\n"),
            Statement::AdvanceOperator {} => output.push_str("_frame.advance_operator()?;\n"),
            Statement::ParseRight {} => output.push_str("_frame.parse_right()?;\n"),
            Statement::BuildBinary {} => output.push_str("_frame.build_binary()?;\n"),
            Statement::Whitespace {} => output.push_str("_frame.whitespace()?;\n"),
            Statement::ScanGroup {} => output.push_str("_frame.scan_group()?;\n"),
            Statement::ScanNegation {} => output.push_str("_frame.scan_negation()?;\n"),
            Statement::AdvanceNegation {} => output.push_str("_frame.advance_negation()?;\n"),
            Statement::BuildNegation {} => output.push_str("_frame.build_negation()?;\n"),
            Statement::ParseNegation { minimum } => {
                output.push_str(&format!("_frame.parse_negation({})?;\n", minimum.rust()))
            }
            Statement::AdvanceGroup {} => output.push_str("_frame.advance_group()?;\n"),
            Statement::CloseGroup {} => output.push_str("_frame.close_group()?;\n"),
            Statement::ScanBound {} => output.push_str("_frame.scan_bound()?;\n"),
            Statement::AdvanceBound {} => output.push_str("_frame.advance_bound()?;\n"),
            Statement::ParseBound {} => output.push_str("_frame.parse_bound()?;\n"),
            Statement::ScanIdentifier {} => output.push_str("_frame.scan_identifier()?;\n"),
            Statement::ParseAtom {} => output.push_str("_frame.parse_atom()?;\n"),
            Statement::ParseInteger {} => output.push_str("_frame.parse_integer()?;\n"),
            Statement::ParseGroup { minimum } => {
                output.push_str(&format!("_frame.parse_group({})?;\n", minimum.rust()))
            }
            Statement::SetBindingPower { value } => {
                output.push_str(&format!("_frame.set_binding_power({})?;\n", value.rust()))
            }
            Statement::If {
                condition,
                then,
                otherwise,
            } => {
                output.push_str(&format!("if {} {{\n", condition.rust()));
                emit(then, output);
                if !otherwise.is_empty() {
                    output.push_str("} else {\n");
                    emit(otherwise, output);
                }
                output.push_str("}\n");
            }
            Statement::While { .. } => unreachable!(),
        }
    }
}
/// Lower parser control statements to Rust calls and control flow, not a native Pratt loop.
pub fn generate(input: &str) -> Result<String, Error> {
    let program = parse(input)?;
    let mut output = String::from(
        "// Generated by bootstrap-pratt-codegen. Do not edit by hand.\nuse crate::parser::{Error, Frame};\npub fn execute(_frame: &mut Frame<'_, '_>) -> Result<(), Error> {\n",
    );
    emit(&program.body, &mut output);
    output.push_str("Ok(())\n}\n");
    Ok(output)
}
