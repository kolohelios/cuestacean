//! Executable expression-parser control flow. Frame operations are native;
//! the loop, binding-power comparison and associativity policy are CUE-authored.
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("invalid Pratt control JSON: {source}"))]
    Json { source: serde_json::Error },
    #[snafu(display("{source}"))]
    Lowering { source: crate::lowering::Error },
    #[snafu(display("invalid Pratt control: {reason}"))]
    Program { reason: &'static str },
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Program {
    format_version: u32,
    body: Vec<Statement>,
}
#[derive(Clone, Deserialize, Serialize)]
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
#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "op", rename_all = "camelCase", deny_unknown_fields)]
enum Condition {
    HasOperator {},
    LeftAssociative {},
    GroupOpen {},
    HasBound {},
    HasString {},
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
#[derive(Clone, Deserialize, Serialize)]
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
    ScanString {},
    ParseString {},
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
    let program: Program = serde_json::from_str(input).context(JsonSnafu)?;
    if program.format_version != 1 {
        return ProgramSnafu {
            reason: "formatVersion 1 required",
        }
        .fail();
    }
    Ok(program)
}
impl Number {
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
    #[cfg(feature = "source")]
    fn test(&self, frame: &crate::parser::Frame<'_, '_>) -> Result<bool, crate::parser::Error> {
        Ok(match self {
            Self::HasOperator {} => frame.has_operator(),
            Self::LeftAssociative {} => frame.left_associative()?,
            Self::GroupOpen {} => frame.group_open()?,
            Self::HasBound {} => frame.has_bound()?,
            Self::HasString {} => frame.has_string()?,
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
                    Statement::ScanString {} => frame.scan_string()?,
                    Statement::ParseString {} => frame.parse_string()?,
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
/// Compile parser control through the shared CUE-owned lowering backend.
pub fn generate(input: &str) -> Result<String, Error> {
    #[cfg(feature = "compiled-lowering")]
    {
        lower(parse(input)?, &crate::compiled_pratt_lowering::Compiled)
    }
    #[cfg(not(feature = "compiled-lowering"))]
    {
        generate_with_policy(
            input,
            include_str!(concat!(
                env!("CUESTACEAN_BOOTSTRAP_JSON"),
                "/pratt-lowering.json"
            )),
        )
    }
}

/// Use a custom trusted compiler policy after validating the complete control IR.
pub fn generate_with_policy(input: &str, policy: &str) -> Result<String, Error> {
    let program = parse(input)?;
    let policy =
        crate::lowering::Policy::parse(policy).map_err(|source| Error::Lowering { source })?;
    lower(program, &policy)
}

fn lower(program: Program, driver: &dyn crate::lowering::Driver) -> Result<String, Error> {
    let node = serde_json::json!({"op":"program", "body":program.body});
    crate::lowering::Context::run(&node, driver).map_err(|source| Error::Lowering { source })
}
