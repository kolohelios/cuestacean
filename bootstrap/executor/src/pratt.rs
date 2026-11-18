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
    Record,
    Conjunction,
    Slice,
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
    HasLower {},
    HasUpper {},
    SliceKindIs {
        kind: crate::unification::Kind,
    },
    ItemKindIs {
        kind: crate::unification::Kind,
    },
    HasOperator {},
    HasIndex {},
    HasSlice {},
    HasPostfix {},
    HasSelectorString {},
    LeftAssociative {},
    GroupOpen {},
    HasBound {},
    HasString {},
    HasList {},
    HasListElement {},
    HasIdentifier {},
    HasCall {},
    HasRecord {},
    HasConjunction {},
    HasRecordField {},
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
    FinishRecord {},
    EvaluateSliceOperand {},
    EvaluateSliceLower {},
    EvaluateSliceUpper {},
    DefaultSliceLower {
        value: i64,
    },
    DefaultSliceUpper {},
    RejectSliceOperand {},
    FinishSlice {},
    SeedTop {},
    UnifyItem {},
    FinishConjunction {},
    Return {},
    Primary {},
    AfterOperand {},
    AfterWordOperator {},
    LookupOperator {},
    AdvanceOperator {},
    ParseRight {},
    BuildBinary {},
    ScanSelector {},
    AdvanceSelector {},
    SelectorWhitespace {},
    ScanSelectorString {},
    ParseSelectorString {},
    ParseSelectorIdentifier {},
    BuildSelector {},
    ScanSlice {},
    AdvanceSlice {},
    ParseSliceUpper {
        minimum: Number,
    },
    CloseSlice {},
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
    Conjunction {},
    AdvanceConjunction {},
    OpenConjunctionCall {},
    OpenConjunctionList {},
    CloseConjunctionCall {},
    BuildConjunction {},
    Record {},
    AdvanceRecord {},
    OpenRecordCall {},
    RecordWhitespace {},
    OpenRecordBody {},
    ProbeRecordEnd {},
    RequireRecordLabel {},
    ParseRecordLabel {},
    RecordLabelTrivia {},
    RecordColon {},
    ParseRecordValue {
        minimum: Number,
    },
    AppendRecordField {},
    AdvanceRecordTail {},
    CloseRecordCall {},
    BuildRecord {},
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
            Condition::HasItem {} | Condition::ItemKindIs { .. } => {
                matches!(target, Target::List | Target::Record | Target::Conjunction)
            }
            Condition::HasLower {} | Condition::HasUpper {} | Condition::SliceKindIs { .. } => {
                target == Target::Slice
            }
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
            | Statement::NextItem {}
            | Statement::PreviousItem {} => {
                matches!(target, Target::List | Target::Record | Target::Conjunction)
            }
            Statement::AppendItem {} | Statement::RejectItem {} => {
                matches!(target, Target::List | Target::Record)
            }
            Statement::SeedTop {} | Statement::UnifyItem {} | Statement::FinishConjunction {} => {
                target == Target::Conjunction
            }
            Statement::EvaluateSliceOperand {}
            | Statement::EvaluateSliceLower {}
            | Statement::EvaluateSliceUpper {}
            | Statement::DefaultSliceLower { .. }
            | Statement::DefaultSliceUpper {}
            | Statement::RejectSliceOperand {}
            | Statement::FinishSlice {} => target == Target::Slice,
            Statement::FinishList {} => target == Target::List,
            Statement::FinishRecord {} => target == Target::Record,
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
            Self::HasItem {}
            | Self::ItemKindIs { .. }
            | Self::HasLower {}
            | Self::HasUpper {}
            | Self::SliceKindIs { .. } => unreachable!("validated parser target"),
            Self::HasOperator {} => frame.has_operator(),
            Self::HasIndex {} => frame.has_index()?,
            Self::HasSlice {} => frame.has_slice()?,
            Self::HasPostfix {} => frame.has_postfix()?,
            Self::HasSelectorString {} => frame.has_selector_string()?,
            Self::LeftAssociative {} => frame.left_associative()?,
            Self::GroupOpen {} => frame.group_open()?,
            Self::HasBound {} => frame.has_bound()?,
            Self::HasString {} => frame.has_string()?,
            Self::HasList {} => frame.has_list()?,
            Self::HasListElement {} => frame.has_list_element()?,
            Self::HasIdentifier {} => frame.has_identifier()?,
            Self::HasCall {} => frame.has_call()?,
            Self::HasRecord {} => frame.has_record()?,
            Self::HasConjunction {} => frame.has_conjunction()?,
            Self::HasRecordField {} => frame.has_record_field()?,
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

// Keep extended-primary/postfix Result temporaries out of recursive VM frames.
// Dispatches checked intrinsics only; traversal/selection policy remains CUE.
#[cfg(feature = "source")]
fn extended_action(
    statement: &Statement,
    frame: &mut crate::parser::Frame<'_, '_>,
) -> Result<(), crate::parser::Error> {
    match statement {
        Statement::Conjunction {} => frame.conjunction(),
        Statement::AdvanceConjunction {} => frame.advance_conjunction(),
        Statement::OpenConjunctionCall {} => frame.open_conjunction_call(),
        Statement::OpenConjunctionList {} => frame.open_conjunction_list(),
        Statement::CloseConjunctionCall {} => frame.close_conjunction_call(),
        Statement::BuildConjunction {} => frame.build_conjunction(),
        Statement::ScanSlice {} => frame.scan_slice(),
        Statement::AdvanceSlice {} => frame.advance_slice(),
        Statement::ParseSliceUpper { minimum } => frame.parse_slice_upper(minimum.value(frame)?),
        Statement::CloseSlice {} => frame.close_slice(),
        Statement::ScanSelector {} => frame.scan_selector(),
        Statement::AdvanceSelector {} => frame.advance_selector(),
        Statement::SelectorWhitespace {} => frame.selector_whitespace(),
        Statement::ScanSelectorString {} => frame.scan_selector_string(),
        Statement::ParseSelectorString {} => frame.parse_selector_string(),
        Statement::ParseSelectorIdentifier {} => frame.parse_selector_identifier(),
        Statement::BuildSelector {} => frame.build_selector(),
        _ => unreachable!("checked selector action"),
    }
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
                    Statement::Conjunction {}
                    | Statement::AdvanceConjunction {}
                    | Statement::OpenConjunctionCall {}
                    | Statement::OpenConjunctionList {}
                    | Statement::CloseConjunctionCall {}
                    | Statement::BuildConjunction {}
                    | Statement::ScanSlice {}
                    | Statement::AdvanceSlice {}
                    | Statement::ParseSliceUpper { .. }
                    | Statement::CloseSlice {}
                    | Statement::ScanSelector {}
                    | Statement::AdvanceSelector {}
                    | Statement::SelectorWhitespace {}
                    | Statement::ScanSelectorString {}
                    | Statement::ParseSelectorString {}
                    | Statement::ParseSelectorIdentifier {}
                    | Statement::BuildSelector {} => extended_action(statement, frame)?,
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
                    Statement::Record {} => frame.record()?,
                    Statement::AdvanceRecord {} => frame.advance_record()?,
                    Statement::OpenRecordCall {} => frame.open_record_call()?,
                    Statement::RecordWhitespace {} => frame.record_whitespace()?,
                    Statement::OpenRecordBody {} => frame.open_record_body()?,
                    Statement::ProbeRecordEnd {} => frame.probe_record_end()?,
                    Statement::RequireRecordLabel {} => frame.require_record_label()?,
                    Statement::ParseRecordLabel {} => frame.parse_record_label()?,
                    Statement::RecordLabelTrivia {} => frame.record_label_trivia()?,
                    Statement::RecordColon {} => frame.record_colon()?,
                    Statement::ParseRecordValue { minimum } => {
                        frame.parse_record_value(minimum.value(frame)?)?
                    }
                    Statement::AppendRecordField {} => frame.append_record_field()?,
                    Statement::AdvanceRecordTail {} => frame.advance_record_tail()?,
                    Statement::CloseRecordCall {} => frame.close_record_call()?,
                    Statement::BuildRecord {} => frame.build_record()?,
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
    execute_items(&program.body, &mut ItemFrame::List(frame)).map(|_| ())
}

/// Execute validated record control using the same structured reference VM.
pub fn interpret_record(
    input: &str,
    frame: &mut crate::runtime::RecordFrame<'_>,
) -> Result<(), crate::Error> {
    let program = parse(input).map_err(|source| crate::Error::ControlProgram { source })?;
    if program.target != Target::Record {
        return Err(crate::Error::RecordState {
            reason: "record control target required",
        });
    }
    execute_items(&program.body, &mut ItemFrame::Record(frame)).map(|_| ())
}

/// Execute validated conjunction control using the shared structured VM.
pub fn interpret_conjunction(
    input: &str,
    frame: &mut crate::runtime::ConjunctionFrame<'_>,
) -> Result<(), crate::Error> {
    let program = parse(input).map_err(|source| crate::Error::ControlProgram { source })?;
    if program.target != Target::Conjunction {
        return Err(crate::Error::ConjunctionState {
            reason: "conjunction control target required",
        });
    }
    execute_items(&program.body, &mut ItemFrame::Conjunction(frame)).map(|_| ())
}

/// Execute validated optional-slice control through the shared structured VM.
pub fn interpret_slice(
    input: &str,
    frame: &mut crate::runtime::SliceFrame<'_>,
) -> Result<(), crate::Error> {
    let program = parse(input).map_err(|source| crate::Error::ControlProgram { source })?;
    if program.target != Target::Slice {
        return Err(crate::Error::SliceState {
            reason: "slice control target required",
        });
    }
    execute_items(&program.body, &mut ItemFrame::Slice(frame)).map(|_| ())
}

// Dispatch only checked frame primitives, never traversal/default selection policy.
enum ItemFrame<'a, 'b> {
    List(&'a mut crate::runtime::ListFrame<'b>),
    Record(&'a mut crate::runtime::RecordFrame<'b>),
    Conjunction(&'a mut crate::runtime::ConjunctionFrame<'b>),
    Slice(&'a mut crate::runtime::SliceFrame<'b>),
}
macro_rules! item_call {
    ($frame:expr, $method:ident($($arg:expr),*)) => {
        match $frame {
            ItemFrame::List(frame) => frame.$method($($arg),*),
            ItemFrame::Record(frame) => frame.$method($($arg),*),
            ItemFrame::Conjunction(frame) => frame.$method($($arg),*),
            ItemFrame::Slice(_) => unreachable!("validated item target"),
        }
    };
}

fn item_step(frame: &mut ItemFrame<'_, '_>) -> Result<(), crate::Error> {
    match frame {
        ItemFrame::Slice(frame) => frame.step(),
        _ => item_call!(frame, step()),
    }
}

fn item_test(condition: &Condition, frame: &ItemFrame<'_, '_>) -> Result<bool, crate::Error> {
    Ok(match condition {
        Condition::Boolean { value } => *value,
        Condition::And { left, right } => item_test(left, frame)? && item_test(right, frame)?,
        Condition::HasLower {} => match frame {
            ItemFrame::Slice(frame) => frame.has_lower()?,
            _ => unreachable!("validated slice target"),
        },
        Condition::HasUpper {} => match frame {
            ItemFrame::Slice(frame) => frame.has_upper()?,
            _ => unreachable!("validated slice target"),
        },
        Condition::SliceKindIs { kind } => match frame {
            ItemFrame::Slice(frame) => frame.slice_kind_is(*kind)?,
            _ => unreachable!("validated slice target"),
        },
        Condition::HasItem {} => item_call!(frame, has_item())?,
        Condition::ItemKindIs { kind } => item_call!(frame, item_kind_is(*kind))?,
        _ => unreachable!("validated collection target"),
    })
}

// Keep slice-only Result temporaries out of other recursive control frames.
fn slice_action(
    statement: &Statement,
    frame: &mut crate::runtime::SliceFrame<'_>,
) -> Result<(), crate::Error> {
    match statement {
        Statement::EvaluateSliceOperand {} => frame.evaluate_slice_operand(),
        Statement::EvaluateSliceLower {} => frame.evaluate_slice_lower(),
        Statement::EvaluateSliceUpper {} => frame.evaluate_slice_upper(),
        Statement::DefaultSliceLower { value } => frame.default_slice_lower(*value),
        Statement::DefaultSliceUpper {} => frame.default_slice_upper(),
        Statement::RejectSliceOperand {} => frame.reject_slice_operand(),
        Statement::FinishSlice {} => frame.finish_slice(),
        _ => unreachable!("validated slice action"),
    }
}

fn execute_items(body: &[Statement], frame: &mut ItemFrame<'_, '_>) -> Result<Flow, crate::Error> {
    for statement in body {
        if let Statement::While { condition, body } = statement {
            while {
                item_step(frame)?;
                item_test(condition, frame)?
            } {
                if execute_items(body, frame)? == Flow::Return {
                    return Ok(Flow::Return);
                }
            }
            continue;
        }
        item_step(frame)?;
        match statement {
            Statement::Return {} => return Ok(Flow::Return),
            Statement::FirstItem {} => item_call!(&mut *frame, first_item())?,
            Statement::LastItem {} => item_call!(&mut *frame, last_item())?,
            Statement::EvaluateItem {} => item_call!(&mut *frame, evaluate_item())?,
            Statement::EvaluateSliceOperand {}
            | Statement::EvaluateSliceLower {}
            | Statement::EvaluateSliceUpper {}
            | Statement::DefaultSliceLower { .. }
            | Statement::DefaultSliceUpper {}
            | Statement::RejectSliceOperand {}
            | Statement::FinishSlice {} => match frame {
                ItemFrame::Slice(frame) => slice_action(statement, frame)?,
                _ => unreachable!("validated slice target"),
            },
            Statement::SeedTop {} => match frame {
                ItemFrame::Conjunction(frame) => frame.seed_top()?,
                _ => unreachable!("validated conjunction target"),
            },
            Statement::UnifyItem {} => match frame {
                ItemFrame::Conjunction(frame) => frame.unify_item()?,
                _ => unreachable!("validated conjunction target"),
            },
            Statement::FinishConjunction {} => match frame {
                ItemFrame::Conjunction(frame) => frame.finish_conjunction()?,
                _ => unreachable!("validated conjunction target"),
            },
            Statement::AppendItem {} => match frame {
                ItemFrame::List(frame) => frame.append_item()?,
                ItemFrame::Record(frame) => frame.append_item()?,
                _ => unreachable!("validated collection target"),
            },
            Statement::NextItem {} => item_call!(&mut *frame, next_item())?,
            Statement::PreviousItem {} => item_call!(&mut *frame, previous_item())?,
            Statement::RejectItem {} => match frame {
                ItemFrame::List(frame) => frame.reject_item()?,
                ItemFrame::Record(frame) => frame.reject_item()?,
                _ => unreachable!("validated collection target"),
            },
            Statement::FinishList {} => match frame {
                ItemFrame::List(frame) => frame.finish_list()?,
                _ => unreachable!("validated list target"),
            },
            Statement::FinishRecord {} => match frame {
                ItemFrame::Record(frame) => frame.finish_record()?,
                _ => unreachable!("validated record target"),
            },
            Statement::If {
                condition,
                then,
                otherwise,
            } => {
                let branch = if item_test(condition, frame)? {
                    then
                } else {
                    otherwise
                };
                if execute_items(branch, frame)? == Flow::Return {
                    return Ok(Flow::Return);
                }
            }
            _ => unreachable!("validated collection target"),
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
                    Statement::ScanSelector {}
                    | Statement::AdvanceSelector {}
                    | Statement::SelectorWhitespace {}
                    | Statement::ScanSelectorString {}
                    | Statement::ParseSelectorString {}
                    | Statement::ParseSelectorIdentifier {}
                    | Statement::BuildSelector {} => {
                        node["action"] = node["op"].clone();
                        node["op"] = serde_json::json!("selectorAction");
                    }
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
    fn source_record_and_conjunction_operations_are_parser_only_even_in_dead_arms() {
        for op in [
            "conjunction",
            "advanceConjunction",
            "openConjunctionCall",
            "openConjunctionList",
            "closeConjunctionCall",
            "buildConjunction",
            "hasConjunction",
            "record",
            "advanceRecord",
            "openRecordCall",
            "recordWhitespace",
            "openRecordBody",
            "probeRecordEnd",
            "requireRecordLabel",
            "parseRecordLabel",
            "recordLabelTrivia",
            "recordColon",
            "parseRecordValue",
            "appendRecordField",
            "advanceRecordTail",
            "closeRecordCall",
            "buildRecord",
            "hasRecord",
            "hasRecordField",
        ] {
            let mut action = serde_json::json!({"op":op});
            if op == "parseRecordValue" {
                action["minimum"] = serde_json::json!({"op":"integer","value":0});
            }
            if op.starts_with("has") {
                action = serde_json::json!({"op":"if","condition":action,"then":[]});
            }
            let body = serde_json::json!([{"op":"if","condition":{"op":"boolean","value":false},"then":[action]}]);
            let mut input = serde_json::json!({"formatVersion":1,"body":body});
            assert!(parse(&input.to_string()).is_ok(), "{op}");
            for target in ["list", "record", "conjunction"] {
                input["target"] = serde_json::json!(target);
                assert!(parse(&input.to_string()).is_err(), "{target}: {op}");
            }
        }
    }

    #[test]
    fn index_operations_are_parser_only_and_minimum_is_typed() {
        for op in [
            "scanSlice",
            "advanceSlice",
            "parseSliceUpper",
            "closeSlice",
            "scanIndex",
            "advanceIndex",
            "parseIndex",
            "closeIndex",
            "scanSelector",
            "advanceSelector",
            "selectorWhitespace",
            "scanSelectorString",
            "parseSelectorString",
            "parseSelectorIdentifier",
            "buildSelector",
        ] {
            let mut action = serde_json::json!({"op":op});
            if matches!(op, "parseIndex" | "parseSliceUpper") {
                action["minimum"] = serde_json::json!({"op":"integer","value":0});
            }
            let mut input = serde_json::json!({"formatVersion":1,"body":[action]});
            assert!(parse(&input.to_string()).is_ok(), "{op}");
            for target in ["list", "record"] {
                input["target"] = serde_json::json!(target);
                assert!(parse(&input.to_string()).is_err(), "{target}: {op}");
            }
        }
        for condition in ["hasSlice", "hasIndex", "hasPostfix", "hasSelectorString"] {
            let mut input = serde_json::json!({"formatVersion":1,"body":[
                {"op":"if","condition":{"op":condition},"then":[]}
            ]});
            assert!(parse(&input.to_string()).is_ok());
            for target in ["list", "record"] {
                input["target"] = serde_json::json!(target);
                assert!(parse(&input.to_string()).is_err());
            }
        }
        for op in ["parseIndex", "parseSliceUpper"] {
            for minimum in [serde_json::json!(256), serde_json::json!({"op":"hasIndex"})] {
                let input = serde_json::json!({"formatVersion":1,"body":[
                    {"op":op,"minimum":minimum}
                ]});
                assert!(parse(&input.to_string()).is_err());
            }
        }
    }

    #[test]
    fn conjunction_operations_and_dead_arms_respect_target_families() {
        for op in ["seedTop", "unifyItem", "finishConjunction"] {
            let mut input = serde_json::json!({"formatVersion":1,"target":"conjunction","body":[
                {"op":"if","condition":{"op":"boolean","value":false},"then":[{"op":op}]}
            ]});
            assert!(parse(&input.to_string()).is_ok(), "{op}");
            for target in ["parser", "list", "record"] {
                input["target"] = serde_json::json!(target);
                assert!(parse(&input.to_string()).is_err(), "{target}: {op}");
            }
        }
        for op in [
            "appendItem",
            "rejectItem",
            "finishList",
            "finishRecord",
            "primary",
            "scanSlice",
        ] {
            let input = serde_json::json!({"formatVersion":1,"target":"conjunction","body":[
                {"op":"if","condition":{"op":"boolean","value":false},"then":[{"op":op}]}
            ]});
            assert!(parse(&input.to_string()).is_err(), "{op}");
        }
        for condition in [
            serde_json::json!({"op":"hasItem"}),
            serde_json::json!({"op":"itemKindIs","kind":"bounds"}),
        ] {
            let input = serde_json::json!({"formatVersion":1,"target":"conjunction","body":[{"op":"if","condition":condition,"then":[]}]});
            assert!(parse(&input.to_string()).is_ok());
        }
        let malformed = r#"{"formatVersion":1,"target":"conjunction","body":[{"op":"seedTop","op":"unifyItem"}]}"#;
        assert!(parse(malformed).is_err());
        let wire = r#"{"formatVersion":1,"target":"conjunction","body":[{"op":"action","action":"seedTop"}]}"#;
        assert!(parse(wire).is_err());
        let ignored = r#"{"formatVersion":1,"target":"conjunction","body":[{"op":"appendItem"}]}"#;
        assert!(
            super::generate_with_policy(
                ignored,
                r#"{"formatVersion":1,"cases":{"program":{"op":"text","value":"ignored"}}}"#
            )
            .is_err()
        );
    }

    #[test]
    fn optional_slice_wire_is_typed_and_target_checked_in_dead_arms() {
        for op in [
            "evaluateSliceOperand",
            "evaluateSliceLower",
            "evaluateSliceUpper",
            "defaultSliceLower",
            "defaultSliceUpper",
            "rejectSliceOperand",
            "finishSlice",
        ] {
            let mut action = serde_json::json!({"op":op});
            if op == "defaultSliceLower" {
                action["value"] = serde_json::json!(i64::MIN);
            }
            let mut input = serde_json::json!({"formatVersion":1,"target":"slice","body":[{"op":"if","condition":{"op":"boolean","value":false},"then":[action]}]});
            assert!(parse(&input.to_string()).is_ok(), "{op}");
            for target in ["parser", "list", "record", "conjunction"] {
                input["target"] = serde_json::json!(target);
                assert!(parse(&input.to_string()).is_err(), "{target}: {op}");
            }
        }
        for action in [
            serde_json::json!({"op":"firstItem"}),
            serde_json::json!({"op":"seedTop"}),
            serde_json::json!({"op":"parseSliceUpper","minimum":{"op":"integer","value":0}}),
            serde_json::json!({"op":"action","action":"finishSlice"}),
            serde_json::json!({"op":"defaultSliceLower","value":true}),
        ] {
            let input = serde_json::json!({"formatVersion":1,"target":"slice","body":[{"op":"if","condition":{"op":"boolean","value":false},"then":[action]}]});
            assert!(parse(&input.to_string()).is_err());
        }
        for condition in [
            serde_json::json!({"op":"hasItem"}),
            serde_json::json!({"op":"hasSlice"}),
        ] {
            let input = serde_json::json!({"formatVersion":1,"target":"slice","body":[{"op":"if","condition":condition,"then":[]}]});
            assert!(parse(&input.to_string()).is_err());
        }
        assert!(parse(r#"{"formatVersion":1,"target":"slice","body":[{"op":"defaultSliceLower","value":9223372036854775808}]}"#).is_err());
        assert!(parse(r#"{"formatVersion":1,"target":"slice","body":[{"op":"finishSlice","op":"return"}]}"#).is_err());
        assert!(
            super::generate_with_policy(
                r#"{"formatVersion":1,"target":"slice","body":[{"op":"appendItem"}]}"#,
                r#"{"formatVersion":1,"cases":{"program":{"op":"text","value":"ignored"}}}"#
            )
            .is_err()
        );
    }

    #[test]
    fn target_defaults_and_strict_wire_are_preserved() {
        for action in [
            serde_json::json!({"op":"selectorAction","action":"scanSelector"}),
            serde_json::json!({"op":"scanSelector","action":"scanSelector"}),
            serde_json::json!({"op":"parseMinimum","minimum":{"op":"integer","value":0}}),
        ] {
            let input = serde_json::json!({"formatVersion":1,"body":[{"op":"if","condition":{"op":"boolean","value":false},"then":[action]}]});
            assert!(parse(&input.to_string()).is_err());
        }
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
