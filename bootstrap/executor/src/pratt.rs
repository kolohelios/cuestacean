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
    #[serde(default)]
    target: Target,
    body: Vec<Statement>,
}
#[derive(Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
enum Target {
    #[default]
    Parser,
    List,
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
    HasItem {},
    ItemKindIs {
        kind: crate::unification::Kind,
    },
    HasOperator {},
    HasIndex {},
    LeftAssociative {},
    GroupOpen {},
    HasBound {},
    HasString {},
    HasList {},
    HasListElement {},
    HasIdentifier {},
    HasCall {},
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
    FirstItem {},
    LastItem {},
    EvaluateItem {},
    AppendItem {},
    NextItem {},
    PreviousItem {},
    RejectItem {},
    FinishList {},
    Return {},
    Primary {},
    AfterOperand {},
    AfterWordOperator {},
    LookupOperator {},
    AdvanceOperator {},
    ParseRight {},
    BuildBinary {},
    ScanIndex {},
    AdvanceIndex {},
    ParseIndex {
        minimum: Number,
    },
    CloseIndex {},
    Whitespace {},
    ScanGroup {},
    AdvanceGroup {},
    CloseGroup {},
    ScanBound {},
    AdvanceBound {},
    ParseBound {},
    ScanString {},
    ParseString {},
    ScanList {},
    AdvanceList {},
    ProbeListEnd {},
    ParseListElement {
        minimum: Number,
    },
    AppendListElement {},
    AdvanceListTail {},
    BuildList {},
    ScanIdentifier {},
    AdvanceCall {},
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
    validate_family(&program.body, program.target)?;
    definite_return(&program.body)?;
    Ok(program)
}

fn validate_family(body: &[Statement], target: Target) -> Result<(), Error> {
    fn condition_valid(condition: &Condition, target: Target) -> bool {
        match condition {
            Condition::Boolean { .. } => true,
            Condition::And { left, right } => {
                condition_valid(left, target) && condition_valid(right, target)
            }
            Condition::HasItem {} | Condition::ItemKindIs { .. } => target == Target::List,
            _ => target == Target::Parser,
        }
    }
    for statement in body {
        let valid = match statement {
            Statement::Return {} => true,
            Statement::If {
                condition,
                then,
                otherwise,
            } => {
                validate_family(then, target)?;
                validate_family(otherwise, target)?;
                condition_valid(condition, target)
            }
            Statement::While { condition, body } => {
                validate_family(body, target)?;
                condition_valid(condition, target)
            }
            Statement::FirstItem {}
            | Statement::LastItem {}
            | Statement::EvaluateItem {}
            | Statement::AppendItem {}
            | Statement::NextItem {}
            | Statement::PreviousItem {}
            | Statement::RejectItem {}
            | Statement::FinishList {} => target == Target::List,
            _ => target == Target::Parser,
        };
        if !valid {
            return ProgramSnafu {
                reason: "operation does not match control target",
            }
            .fail();
        }
    }
    Ok(())
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
            Self::HasItem {} | Self::ItemKindIs { .. } => unreachable!("validated parser target"),
            Self::HasOperator {} => frame.has_operator(),
            Self::HasIndex {} => frame.has_index()?,
            Self::LeftAssociative {} => frame.left_associative()?,
            Self::GroupOpen {} => frame.group_open()?,
            Self::HasBound {} => frame.has_bound()?,
            Self::HasString {} => frame.has_string()?,
            Self::HasList {} => frame.has_list()?,
            Self::HasListElement {} => frame.has_list_element()?,
            Self::HasIdentifier {} => frame.has_identifier()?,
            Self::HasCall {} => frame.has_call()?,
            Self::HasNegation {} => frame.has_negation()?,
            Self::HasNot {} => frame.has_not()?,
            Self::Boolean { value } => *value,
            Self::AtLeast { left, right } => left.value(frame)? >= right.value(frame)?,
            Self::And { left, right } => left.test(frame)? && right.test(frame)?,
        })
    }
}
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
                    Statement::ScanIndex {} => frame.scan_index()?,
                    Statement::AdvanceIndex {} => frame.advance_index()?,
                    Statement::ParseIndex { minimum } => {
                        frame.parse_index(minimum.value(frame)?)?
                    }
                    Statement::CloseIndex {} => frame.close_index()?,
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
                    Statement::ScanList {} => frame.scan_list()?,
                    Statement::AdvanceList {} => frame.advance_list()?,
                    Statement::ProbeListEnd {} => frame.probe_list_end()?,
                    Statement::ParseListElement { minimum } => {
                        frame.parse_list_element(minimum.value(frame)?)?
                    }
                    Statement::AppendListElement {} => frame.append_list_element()?,
                    Statement::AdvanceListTail {} => frame.advance_list_tail()?,
                    Statement::BuildList {} => frame.build_list()?,
                    Statement::ScanIdentifier {} => frame.scan_identifier()?,
                    Statement::AdvanceCall {} => frame.advance_call()?,
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
                    _ => unreachable!("validated parser target"),
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
    if program.target != Target::Parser {
        return Err(crate::parser::Error::ControlState {
            reason: "parser control target required",
        });
    }
    execute(&program.body, frame).map(|_| ())
}
/// Execute validated list control using the reference VM.
pub fn interpret_list(
    input: &str,
    frame: &mut crate::runtime::ListFrame<'_>,
) -> Result<(), crate::Error> {
    let program = parse(input).map_err(|source| crate::Error::ControlProgram { source })?;
    if program.target != Target::List {
        return Err(crate::Error::ListState {
            reason: "list control target required",
        });
    }
    execute_list(&program.body, frame).map(|_| ())
}

fn list_test(
    condition: &Condition,
    frame: &crate::runtime::ListFrame<'_>,
) -> Result<bool, crate::Error> {
    Ok(match condition {
        Condition::Boolean { value } => *value,
        Condition::And { left, right } => list_test(left, frame)? && list_test(right, frame)?,
        Condition::HasItem {} => frame.has_item()?,
        Condition::ItemKindIs { kind } => frame.item_kind_is(*kind)?,
        _ => unreachable!("validated list target"),
    })
}

fn execute_list(
    body: &[Statement],
    frame: &mut crate::runtime::ListFrame<'_>,
) -> Result<Flow, crate::Error> {
    for statement in body {
        if let Statement::While { condition, body } = statement {
            while {
                frame.step()?;
                list_test(condition, frame)?
            } {
                if execute_list(body, frame)? == Flow::Return {
                    return Ok(Flow::Return);
                }
            }
            continue;
        }
        frame.step()?;
        match statement {
            Statement::Return {} => return Ok(Flow::Return),
            Statement::FirstItem {} => frame.first_item()?,
            Statement::LastItem {} => frame.last_item()?,
            Statement::EvaluateItem {} => frame.evaluate_item()?,
            Statement::AppendItem {} => frame.append_item()?,
            Statement::NextItem {} => frame.next_item()?,
            Statement::PreviousItem {} => frame.previous_item()?,
            Statement::RejectItem {} => frame.reject_item()?,
            Statement::FinishList {} => frame.finish_list()?,
            Statement::If {
                condition,
                then,
                otherwise,
            } => {
                let branch = if list_test(condition, frame)? {
                    then
                } else {
                    otherwise
                };
                if execute_list(branch, frame)? == Flow::Return {
                    return Ok(Flow::Return);
                }
            }
            _ => unreachable!("validated list target"),
        }
    }
    Ok(Flow::Continue)
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
                    // Validated unit statements share a canonical category;
                    // CUE alone maps their wire names to checked method names.
                    _ if node.as_object().is_some_and(|fields| fields.len() == 1) => {
                        node["action"] = node["op"].clone();
                        node["op"] = serde_json::json!("action");
                    }
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
    let node =
        serde_json::json!({"op":"program", "body":body, "tail":tail, "target":program.target});
    crate::lowering::Context::run(&node, driver).map_err(|source| Error::Lowering { source })
}

#[cfg(test)]
mod target_tests {
    use super::parse;

    #[test]
    fn family_validation_visits_dead_branches_and_nested_conditions() {
        for (target, op) in [("list", "primary"), ("parser", "firstItem")] {
            let input = serde_json::json!({"formatVersion":1,"target":target,"body":[
                {"op":"if","condition":{"op":"boolean","value":false},"then":[{"op":op}]}
            ]});
            assert!(parse(&input.to_string()).is_err());
        }
        for (target, condition) in [
            ("parser", serde_json::json!({"op":"hasItem"})),
            ("list", serde_json::json!({"op":"hasOperator"})),
        ] {
            let input = serde_json::json!({"formatVersion":1,"target":target,"body":[
                {"op":"while","condition":{"op":"and","left":{"op":"boolean","value":false},
                    "right":condition},"body":[]}
            ]});
            assert!(parse(&input.to_string()).is_err());
        }
    }

    #[test]
    fn source_list_operations_are_parser_only_and_minimum_is_typed() {
        for op in [
            "scanList",
            "advanceList",
            "probeListEnd",
            "parseListElement",
            "appendListElement",
            "advanceListTail",
            "buildList",
        ] {
            let mut action = serde_json::json!({"op":op});
            if op == "parseListElement" {
                action["minimum"] = serde_json::json!({"op":"integer","value":100});
            }
            let mut input = serde_json::json!({"formatVersion":1,"body":[action]});
            assert!(parse(&input.to_string()).is_ok(), "{op}");
            input["target"] = serde_json::json!("list");
            assert!(parse(&input.to_string()).is_err(), "{op}");
        }
        for op in ["hasList", "hasListElement"] {
            let mut input = serde_json::json!({"formatVersion":1,"body":[
                {"op":"if","condition":{"op":op},"then":[]}
            ]});
            assert!(parse(&input.to_string()).is_ok());
            input["target"] = serde_json::json!("list");
            assert!(parse(&input.to_string()).is_err());
        }
        for minimum in [
            serde_json::json!({"op":"integer","value":256}),
            serde_json::json!({"op":"integer","value":-1}),
            serde_json::json!({"op":"integer","value":"0"}),
            serde_json::json!({"op":"hasList"}),
        ] {
            let input = serde_json::json!({"formatVersion":1,"body":[
                {"op":"parseListElement","minimum":minimum}
            ]});
            assert!(parse(&input.to_string()).is_err());
        }
    }

    #[test]
    fn index_operations_are_parser_only_and_minimum_is_typed() {
        for op in ["scanIndex", "advanceIndex", "parseIndex", "closeIndex"] {
            let mut action = serde_json::json!({"op":op});
            if op == "parseIndex" {
                action["minimum"] = serde_json::json!({"op":"integer","value":0});
            }
            let mut input = serde_json::json!({"formatVersion":1,"body":[action]});
            assert!(parse(&input.to_string()).is_ok(), "{op}");
            input["target"] = serde_json::json!("list");
            assert!(parse(&input.to_string()).is_err(), "{op}");
        }
        let mut input = serde_json::json!({"formatVersion":1,"body":[
            {"op":"if","condition":{"op":"hasIndex"},"then":[]}
        ]});
        assert!(parse(&input.to_string()).is_ok());
        input["target"] = serde_json::json!("list");
        assert!(parse(&input.to_string()).is_err());
        for minimum in [serde_json::json!(256), serde_json::json!({"op":"hasIndex"})] {
            let input = serde_json::json!({"formatVersion":1,"body":[
                {"op":"parseIndex","minimum":minimum}
            ]});
            assert!(parse(&input.to_string()).is_err());
        }
    }

    #[test]
    fn target_defaults_and_strict_wire_are_preserved() {
        assert!(parse(r#"{"formatVersion":1,"body":[{"op":"primary"}]}"#).is_ok());
        assert!(parse(r#"{"formatVersion":1,"target":"list","body":[{"op":"return"}]}"#).is_ok());
        assert!(parse(r#"{"formatVersion":1,"target":"List","body":[]}"#).is_err());
        assert!(parse(r#"{"formatVersion":1,"target":"list","extra":0,"body":[]}"#).is_err());
    }
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
