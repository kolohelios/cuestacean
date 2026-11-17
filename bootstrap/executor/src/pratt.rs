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
    HasNot {},
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
    Return {},
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
    ScanNot {},
    AdvanceNot {},
    BuildNot {},
    ParseNot {
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
    definite_return(&program.body)?;
    Ok(program)
}

// Validate every block without folding conditions or assuming loops terminate.
fn definite_return(body: &[Statement]) -> Result<bool, Error> {
    let mut returns = false;
    for statement in body {
        if returns {
            return ProgramSnafu {
                reason: "statements after definite return",
            }
            .fail();
        }
        returns = match statement {
            Statement::Return {} => true,
            Statement::If {
                then, otherwise, ..
            } => {
                let then_returns = definite_return(then)?;
                let otherwise_returns = definite_return(otherwise)?;
                then_returns && otherwise_returns
            }
            Statement::While { body, .. } => {
                definite_return(body)?;
                false
            }
            _ => false,
        };
    }
    Ok(returns)
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
            Self::HasNot {} => frame.has_not()?,
            Self::Boolean { value } => *value,
            Self::AtLeast { left, right } => left.value(frame)? >= right.value(frame)?,
            Self::And { left, right } => left.test(frame)? && right.test(frame)?,
        })
    }
}
#[cfg(feature = "source")]
#[derive(PartialEq, Eq)]
enum Flow {
    Continue,
    Return,
}

#[cfg(feature = "source")]
fn execute(
    body: &[Statement],
    frame: &mut crate::parser::Frame<'_, '_>,
) -> Result<Flow, crate::parser::Error> {
    for statement in body {
        match statement {
            Statement::While { condition, body } => {
                while {
                    frame.step()?;
                    condition.test(frame)?
                } {
                    if execute(body, frame)? == Flow::Return {
                        return Ok(Flow::Return);
                    }
                }
            }
            _ => {
                frame.step()?;
                match statement {
                    Statement::Return {} => return Ok(Flow::Return),
                    Statement::Primary {} => frame.primary()?,
                    Statement::AfterOperand {} => frame.after_operand()?,
                    Statement::AfterWordOperator {} => frame.after_word_operator()?,
                    Statement::LookupOperator {} => frame.lookup_operator()?,
                    Statement::AdvanceOperator {} => frame.advance_operator()?,
                    Statement::ParseRight {} => frame.parse_right()?,
                    Statement::BuildBinary {} => frame.build_binary()?,
                    Statement::Whitespace {} => frame.whitespace()?,
                    Statement::ScanGroup {} => frame.scan_group()?,
                    Statement::ScanNot {} => frame.scan_not()?,
                    Statement::AdvanceNot {} => frame.advance_not()?,
                    Statement::BuildNot {} => frame.build_not()?,
                    Statement::ParseNot { minimum } => frame.parse_not(minimum.value(frame)?)?,
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
                    } => {
                        let body = if condition.test(frame)? {
                            then
                        } else {
                            otherwise
                        };
                        if execute(body, frame)? == Flow::Return {
                            return Ok(Flow::Return);
                        }
                    }
                    Statement::While { .. } => unreachable!(),
                }
            }
        }
    }
    Ok(Flow::Continue)
}
#[cfg(feature = "source")]
pub(crate) fn interpret(
    program: &Program,
    frame: &mut crate::parser::Frame<'_, '_>,
) -> Result<(), crate::parser::Error> {
    execute(&program.body, frame).map(|_| ())
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

// Tail context is supplied only for a checked definitely-returning block.
fn project_body(body: &[Statement], tail: bool) -> serde_json::Value {
    serde_json::Value::Array(
        body.iter()
            .enumerate()
            .map(|(index, statement)| {
                let at_tail = tail && index + 1 == body.len();
                let mut node = serde_json::json!(statement);
                match statement {
                    Statement::Return {} => {
                        node["position"] =
                            serde_json::json!(if at_tail { "tail" } else { "early" });
                    }
                    Statement::If {
                        then, otherwise, ..
                    } => {
                        // At a definite block tail, both arms must return.
                        node["then"] = project_body(then, at_tail);
                        node["else"] = project_body(otherwise, at_tail);
                    }
                    Statement::While { body, .. } => node["body"] = project_body(body, false),
                    _ => {}
                }
                node
            })
            .collect(),
    )
}

fn lower(program: Program, driver: &dyn crate::lowering::Driver) -> Result<String, Error> {
    let returns = definite_return(&program.body)?;
    let body = project_body(&program.body, returns);
    let tail = if returns { "return" } else { "value" };
    let node = serde_json::json!({"op":"program", "body":body, "tail":tail});
    crate::lowering::Context::run(&node, driver).map_err(|source| Error::Lowering { source })
}

#[cfg(all(test, feature = "source"))]
mod return_tests {
    use super::{definite_return, interpret, parse, project_body};
    use crate::parser::{self, Error, Frame};

    const GRAMMAR: &str = include_str!(concat!(env!("CUESTACEAN_BOOTSTRAP_JSON"), "/grammar.json"));

    fn return_after<const STEPS: usize>(frame: &mut Frame<'_, '_>) -> Result<(), Error> {
        for _ in 0..STEPS {
            frame.step()?;
        }
        let program =
            parse(r#"{"formatVersion":1,"body":[{"op":"return"}]}"#).expect("return program");
        interpret(&program, frame)
    }

    fn expression(frame: &mut Frame<'_, '_>) -> Result<(), Error> {
        frame.step()?;
        frame.primary()
    }

    #[test]
    fn checked_positions_distinguish_nested_function_tails_from_loop_returns() {
        let ret = serde_json::json!({"op":"return"});
        let condition = serde_json::json!({"op":"boolean","value":false});
        let mut original = serde_json::json!({"formatVersion":1,"body":[
            {"op":"while","condition":condition,"body":[ret]},
            {"op":"if","condition":condition,"then":[ret],"else":[]},
            {"op":"if","condition":condition,"then":[
                {"op":"if","condition":condition,"then":[ret],"else":[ret]}
            ],"else":[ret]}
        ]});
        let program = parse(&original.to_string()).expect("structurally valid");
        assert!(definite_return(&program.body).expect("checked block"));
        original["body"][0]["body"][0]["position"] = serde_json::json!("early");
        original["body"][1]["then"][0]["position"] = serde_json::json!("early");
        original["body"][2]["then"][0]["then"][0]["position"] = serde_json::json!("tail");
        original["body"][2]["then"][0]["else"][0]["position"] = serde_json::json!("tail");
        original["body"][2]["else"][0]["position"] = serde_json::json!("tail");
        assert_eq!(project_body(&program.body, true), original["body"]);
    }

    #[test]
    fn reference_return_consumes_exactly_one_shared_step() {
        // Match the default expression driver's one charged primary dispatch.
        let exhausted = parser::parse_with_drivers("1", GRAMMAR, expression, return_after::<4095>);
        assert!(matches!(exhausted, Err(Error::ControlBudget)));
        let last_step = parser::parse_with_drivers("1", GRAMMAR, expression, return_after::<4094>);
        assert!(matches!(
            last_step,
            Err(Error::ControlState {
                reason: "unfinished primary at return"
            })
        ));
    }
}
