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
    pub(super) target: Target,
    pub(super) body: Vec<Statement>,
}
#[derive(Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) enum Target {
    #[default]
    Parser,
    List,
    Record,
    Conjunction,
    Slice,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "op", rename_all = "camelCase", deny_unknown_fields)]
pub(super) enum Number {
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
pub(super) enum Condition {
    FileBoundary {},
    HasFileField {},
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
    HasListEllipsis {},
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
pub(super) enum Statement {
    ProbeFile {},
    StartFile {},
    ParseFileLabel {},
    ParseFileColon {},
    ParseFileValue {},
    AppendFileField {},
    AdvanceFileTail {},
    BuildFile {},
    ParseFileExpression {},
    FileAfterOperand {},
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
    ProbeListEllipsis {},
    AdvanceListEllipsis {},
    CloseListEllipsis {},
    BuildTypedList {},
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
pub(super) fn definite_return(body: &[Statement]) -> Result<bool, Error> {
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
pub(super) fn project_body(body: &[Statement], tail: bool) -> serde_json::Value {
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

pub(super) fn lower(
    program: Program,
    driver: &dyn crate::lowering::Driver,
) -> Result<String, Error> {
    let returns = definite_return(&program.body)?;
    let body = project_body(&program.body, returns);
    let tail = if returns { "return" } else { "value" };
    let node =
        serde_json::json!({"op":"program", "body":body, "tail":tail, "target":program.target});
    crate::lowering::Context::run(&node, driver).map_err(|source| Error::Lowering { source })
}
