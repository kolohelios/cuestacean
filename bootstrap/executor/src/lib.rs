//! Disposable bootstrap machinery for interpreting exported rule instructions.
//! Node kinds and field names come from data, not Rust expression variants.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use snafu::{ResultExt, Snafu};

pub mod ast;
mod codegen;
pub mod lowering;
pub mod module;
#[cfg(feature = "source")]
pub mod parser;
pub mod pratt;
pub mod runtime;
pub mod scanner;
pub mod unification;

pub use codegen::{generate, generate_with_policy};

#[cfg(any(feature = "semantics", feature = "compiled-lowering"))]
extern crate self as bootstrap_executor;

/// Compiled CUE lowering policy. Compiler-only seeds use its reference engine.
#[cfg(feature = "compiled-lowering")]
#[path = "../../generated/lowering.rs"]
pub mod compiled_lowering;

/// Compiled CUE backend for Pratt/primary control generation.
#[cfg(feature = "compiled-lowering")]
#[path = "../../generated/pratt-lowering.rs"]
pub mod compiled_pratt_lowering;

/// Compiled CUE backend for unification-control generation.
#[cfg(feature = "compiled-lowering")]
#[path = "../../generated/unify-lowering.rs"]
pub mod compiled_unify_lowering;

/// Compiled CUE backend for typed AST-builder generation.
#[cfg(feature = "compiled-lowering")]
#[path = "../../generated/ast-lowering.rs"]
pub mod compiled_ast_lowering;

/// CUE-owned scanner compiler source composition.
#[cfg(feature = "compiled-lowering")]
#[path = "../../generated/scan-lowering.rs"]
pub mod compiled_scan_lowering;

/// CUE-owned complete lowering-driver compiler over checked native metadata.
#[cfg(feature = "compiled-lowering")]
#[path = "../../generated/meta-lowering.rs"]
pub mod compiled_meta_lowering;

/// Default CUE-generated rules, with integer-only and JSON scalar entry points.
/// Compiler-only seeds omit this module and all generated semantics.
#[cfg(feature = "semantics")]
#[path = "../../generated/evaluation.rs"]
pub mod generated;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("invalid {document} JSON: {source}"))]
    Json {
        document: &'static str,
        source: serde_json::Error,
    },
    #[snafu(display("unsupported {document} formatVersion: {version}"))]
    Version {
        document: &'static str,
        version: u32,
    },
    #[snafu(display("evaluation exceeds 4096 AST visits"))]
    EvaluationVisits,
    #[snafu(display("evaluation exceeds 64 active AST nodes"))]
    EvaluationDepth,
    #[snafu(display("expression must have a string kind"))]
    Kind,
    #[snafu(display("no rule for node kind {kind:?}"))]
    MissingRule { kind: String },
    #[snafu(display("missing field {field:?}"))]
    MissingField { field: String },
    #[snafu(display("field {field:?} must be a signed 64-bit integer"))]
    Integer { field: String },
    #[snafu(display("field {field:?} must be a string"))]
    Text { field: String },
    #[snafu(display("field {field:?} must be an array"))]
    Array { field: String },
    #[snafu(display("closed list exceeds 64 elements"))]
    ListCapacity,
    #[snafu(display("conjunction exceeds 64 operands"))]
    ConjunctionCapacity,
    #[snafu(display("invalid conjunction control: {reason}"))]
    ConjunctionState { reason: &'static str },
    #[snafu(display("conjunction control exceeds 4096 steps"))]
    ConjunctionControlBudget,
    #[snafu(display(
        "closed list element {index} requires a concrete scalar or closed collection"
    ))]
    ListElement { index: usize },
    #[snafu(display("closed list exceeds 32 value levels"))]
    ListDepth,
    #[snafu(display("closed list exceeds 4096 logical value nodes"))]
    ListNodes,
    #[snafu(display("invalid list control state: {reason}"))]
    ListState { reason: &'static str },
    #[snafu(display("list control exceeds 4096 steps"))]
    ListControlBudget,
    #[snafu(display("{source}"))]
    ControlProgram { source: pratt::Error },
    #[snafu(display("conflicting closed list values"))]
    ListConflict,
    #[snafu(display("length requires a concrete closed list"))]
    ListLength,
    #[snafu(display("invalid record control state: {reason}"))]
    RecordState { reason: &'static str },
    #[snafu(display("record control exceeds 4096 steps"))]
    RecordControlBudget,
    #[snafu(display("closed record exceeds 64 fields"))]
    RecordCapacity,
    #[snafu(display("closed record field name at index {index} must contain 1..=64 UTF-8 bytes"))]
    RecordName { index: usize },
    #[snafu(display("duplicate closed record field {field:?}"))]
    RecordDuplicate { field: String },
    #[snafu(display(
        "closed record field {field:?} requires a concrete scalar or closed collection"
    ))]
    RecordElement { field: String },
    #[snafu(display("closed record exceeds 32 value levels"))]
    RecordDepth,
    #[snafu(display("closed record exceeds 4096 logical value nodes"))]
    RecordNodes,
    #[snafu(display("conflicting closed record values"))]
    RecordConflict,
    #[snafu(display("length requires a concrete closed record"))]
    RecordLength,
    #[snafu(display("field lookup requires a concrete closed record and string"))]
    RecordIndexOperands,
    #[snafu(display("list slicing requires a concrete closed list and integer endpoints"))]
    ListSliceOperands,
    #[snafu(display("list slice out of range"))]
    ListSliceRange,
    #[snafu(display("missing closed record field {field:?}"))]
    RecordMissing { field: String },
    #[snafu(display("signed 64-bit integer addition overflow"))]
    Overflow,
    #[snafu(display("signed 64-bit integer subtraction overflow"))]
    DifferenceOverflow,
    #[snafu(display("signed 64-bit integer multiplication overflow"))]
    ProductOverflow,
    #[snafu(display("integer division by zero"))]
    DivisionByZero,
    #[snafu(display("signed 64-bit integer division overflow"))]
    DivisionOverflow,
    #[snafu(display("signed 64-bit integer negation overflow"))]
    NegationOverflow,
    #[snafu(display("conflicting integer values {left} and {right}"))]
    Conflict { left: i64, right: i64 },
    #[snafu(display("non-concrete int value"))]
    Incomplete,
    #[snafu(display("non-concrete value"))]
    IncompleteValue,
    #[snafu(display("conflicting boolean values {left} and {right}"))]
    BooleanConflict { left: bool, right: bool },
    #[snafu(display("conflicting value types"))]
    TypeConflict,
    #[snafu(display("non-concrete bool value"))]
    IncompleteBool,
    #[snafu(display("non-concrete string value"))]
    IncompleteString,
    #[snafu(display("conflicting string values {left:?} and {right:?}"))]
    StringConflict { left: String, right: String },
    #[snafu(display("integer output requires an integer value"))]
    IntegerOutput,
    #[snafu(display("output policy returned an unsupported value"))]
    OutputPolicy,
    #[snafu(display("conflicting integer bounds"))]
    BoundsConflict,
    #[snafu(display("non-concrete integer bounds"))]
    IncompleteBounds,
    #[snafu(display("bound threshold requires a concrete integer"))]
    BoundThreshold,
    #[snafu(display("exclusive bound is empty in the signed 64-bit integer domain"))]
    EmptyBound,
    #[snafu(display("signed 64-bit integer endpoint offset overflow"))]
    EndpointOffsetOverflow,
    #[snafu(display("arithmetic requires compatible concrete operands"))]
    Arithmetic,
    #[snafu(display("comparison requires concrete operands"))]
    Comparison,
    #[snafu(display("invalid logical operands"))]
    Logical,
    #[snafu(display("ordering requires matching concrete integers or strings"))]
    Ordering,
    #[snafu(display("length requires a concrete string"))]
    Length,
    #[snafu(display("length requires a concrete string or closed collection"))]
    LengthOperand,
    #[snafu(display(
        "indexing requires a concrete closed list and integer or closed record and string"
    ))]
    CollectionIndexOperands,
    #[snafu(display("indexing requires a concrete closed list and integer"))]
    IndexOperands,
    #[snafu(display("list index out of range"))]
    IndexRange,
    #[snafu(display("string byte length exceeds signed 64-bit integer range"))]
    TextLengthOverflow,
    #[snafu(display("string concatenation exceeds available capacity"))]
    TextCapacity,
    #[snafu(display("{source}"))]
    Lowering { source: lowering::Error },
    #[snafu(display("{source}"))]
    UnificationProgram { source: unification::Error },
    #[snafu(display("invalid unification control state: {reason}"))]
    UnificationState { reason: &'static str },
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Program {
    format_version: u32,
    entry: Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Rules {
    format_version: u32,
    rules: BTreeMap<String, Rule>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Rule {
    result: Instruction,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "op", rename_all = "lowercase", deny_unknown_fields)]
enum Instruction {
    Conjunction {
        field: String,
    },
    Null {},
    Top {},
    Int {},
    Bool {},
    Text {
        field: String,
    },
    List {
        field: String,
    },
    Record {
        field: String,
    },
    #[serde(rename = "recordExpression")]
    RecordExpression {
        field: String,
    },
    #[serde(rename = "recordIndex")]
    RecordIndex {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    #[serde(rename = "recordLength")]
    RecordLength {
        value: Box<Instruction>,
    },
    #[serde(rename = "listExpression")]
    ListExpression {
        field: String,
    },
    #[serde(rename = "stringType")]
    StringType {},
    Boolean {
        value: bool,
    },
    #[serde(rename = "lowerBound")]
    LowerBound {
        value: Box<Instruction>,
    },
    #[serde(rename = "lowerExclusiveBound")]
    LowerExclusiveBound {
        value: Box<Instruction>,
    },
    #[serde(rename = "upperExclusiveBound")]
    UpperExclusiveBound {
        value: Box<Instruction>,
    },
    #[serde(rename = "upperBound")]
    UpperBound {
        value: Box<Instruction>,
    },
    Field {
        field: String,
    },
    Evaluate {
        field: String,
    },
    Negate {
        value: Box<Instruction>,
    },
    Index {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    #[serde(rename = "listSlice")]
    ListSlice {
        value: Box<Instruction>,
        lower: Box<Instruction>,
        upper: Box<Instruction>,
    },
    #[serde(rename = "listIndex")]
    ListIndex {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    Sum {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    Difference {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    Product {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    Divide {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    Modulo {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    Quotient {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    Remainder {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    Length {
        value: Box<Instruction>,
    },
    #[serde(rename = "stringLength")]
    StringLength {
        value: Box<Instruction>,
    },
    #[serde(rename = "listLength")]
    ListLength {
        value: Box<Instruction>,
    },
    #[serde(rename = "logicalNot")]
    LogicalNot {
        value: Box<Instruction>,
    },
    #[serde(rename = "logicalAnd")]
    LogicalAnd {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    #[serde(rename = "logicalOr")]
    LogicalOr {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    Less {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    #[serde(rename = "lessEqual")]
    LessEqual {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    Greater {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    #[serde(rename = "greaterEqual")]
    GreaterEqual {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    Equal {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    #[serde(rename = "notEqual")]
    NotEqual {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    Unify {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
}

/// Execute version-1 JSON documents using checked signed 64-bit arithmetic.
/// This executes accessed fields; it does not validate the complete CUE AST.
#[cfg(feature = "semantics")]
pub fn execute(program: &str, rules: &str) -> Result<i64, Error> {
    let program = runtime::program(program)?;
    let rules = parse_rules(rules)?;
    let mut context = runtime::EvaluationContext::default();
    runtime::concrete(evaluate(&program, &rules.rules, &mut context)?)
}

/// Execute to exact concrete JSON scalars or a bounded closed scalar list.
#[cfg(feature = "semantics")]
pub fn execute_json(program: &str, rules: &str) -> Result<Value, Error> {
    let program = runtime::program(program)?;
    let rules = parse_rules(rules)?;
    let mut context = runtime::EvaluationContext::default();
    runtime::concrete_json(evaluate(&program, &rules.rules, &mut context)?)
}

fn parse_rules(input: &str) -> Result<Rules, Error> {
    let rules: Rules = serde_json::from_str(input).context(JsonSnafu { document: "rules" })?;
    if rules.format_version != 1 {
        return VersionSnafu {
            document: "rules",
            version: rules.format_version,
        }
        .fail();
    }
    Ok(rules)
}

#[cfg(feature = "semantics")]
fn evaluate(
    node: &Value,
    rules: &BTreeMap<String, Rule>,
    context: &mut runtime::EvaluationContext,
) -> Result<runtime::Value, Error> {
    context.visit(|context| {
        let kind = runtime::kind(node)?;
        let rule = rules.get(kind).ok_or_else(|| Error::MissingRule {
            kind: kind.to_string(),
        })?;
        instruction(&rule.result, node, rules, context)
    })
}

// Keep operand temporaries in one shared reference-VM frame rather than one
// debug-build stack slot per binary opcode in the recursive dispatch function.
#[cfg(feature = "semantics")]
fn binary_instruction(
    left: &Instruction,
    right: &Instruction,
    node: &Value,
    rules: &BTreeMap<String, Rule>,
    context: &mut runtime::EvaluationContext,
    apply: fn(runtime::Value, runtime::Value) -> Result<runtime::Value, Error>,
) -> Result<runtime::Value, Error> {
    let left = instruction(left, node, rules, context)?;
    let right = instruction(right, node, rules, context)?;
    apply(left, right)
}

// Keep three owned operand temporaries out of unrelated recursive VM arms.
#[cfg(feature = "semantics")]
fn slice_instruction(
    operands: [&Instruction; 3],
    node: &Value,
    rules: &BTreeMap<String, Rule>,
    context: &mut runtime::EvaluationContext,
) -> Result<runtime::Value, Error> {
    let value = instruction(operands[0], node, rules, context)?;
    let lower = instruction(operands[1], node, rules, context)?;
    let upper = instruction(operands[2], node, rules, context)?;
    runtime::list_slice(value, lower, upper)
}

#[cfg(feature = "semantics")]
fn instruction(
    op: &Instruction,
    node: &Value,
    rules: &BTreeMap<String, Rule>,
    context: &mut runtime::EvaluationContext,
) -> Result<runtime::Value, Error> {
    match op {
        Instruction::Null {} => Ok(runtime::Value::Null),
        Instruction::Top {} => Ok(runtime::Value::Top),
        Instruction::Int {} => Ok(runtime::Value::Int),
        Instruction::Bool {} => Ok(runtime::Value::Bool),
        Instruction::Boolean { value } => Ok(runtime::Value::Boolean(*value)),
        Instruction::Text { field } => runtime::text(node, field),
        Instruction::List { field } => runtime::list(node, field),
        Instruction::Conjunction { field } => runtime::conjunction_interpreted(
            node,
            field,
            context,
            |node, context| evaluate(node, rules, context),
            include_str!(concat!(
                env!("CUESTACEAN_BOOTSTRAP_JSON"),
                "/conjunction-control.json"
            )),
        ),
        Instruction::ListExpression { field } => {
            runtime::list_expression_interpreted(node, field, context, |node, context| {
                evaluate(node, rules, context)
            })
        }
        Instruction::RecordExpression { field } => {
            runtime::record_expression_interpreted(node, field, context, |node, context| {
                evaluate(node, rules, context)
            })
        }
        Instruction::StringType {} => Ok(runtime::Value::String),
        Instruction::LowerExclusiveBound { value } => {
            runtime::lower_exclusive_bound(instruction(value, node, rules, context)?)
        }
        Instruction::UpperExclusiveBound { value } => {
            runtime::upper_exclusive_bound(instruction(value, node, rules, context)?)
        }
        Instruction::LowerBound { value } => {
            runtime::lower_bound(instruction(value, node, rules, context)?)
        }
        Instruction::UpperBound { value } => {
            runtime::upper_bound(instruction(value, node, rules, context)?)
        }
        Instruction::Field { field: name } => runtime::integer(node, name),
        Instruction::Evaluate { field: name } => {
            evaluate(runtime::field(node, name)?, rules, context)
        }
        Instruction::Negate { value } => runtime::negate(instruction(value, node, rules, context)?),
        Instruction::Sum { left, right } => {
            binary_instruction(left, right, node, rules, context, runtime::sum)
        }
        Instruction::Difference { left, right } => {
            binary_instruction(left, right, node, rules, context, runtime::difference)
        }
        Instruction::Product { left, right } => {
            binary_instruction(left, right, node, rules, context, runtime::product)
        }
        Instruction::Divide { left, right } => {
            binary_instruction(left, right, node, rules, context, runtime::divide)
        }
        Instruction::Modulo { left, right } => {
            binary_instruction(left, right, node, rules, context, runtime::modulo)
        }
        Instruction::Quotient { left, right } => {
            binary_instruction(left, right, node, rules, context, runtime::quotient)
        }
        Instruction::Remainder { left, right } => {
            binary_instruction(left, right, node, rules, context, runtime::remainder)
        }
        Instruction::Length { value } => runtime::length(instruction(value, node, rules, context)?),
        Instruction::Record { field } => runtime::record(node, field),
        Instruction::RecordLength { value } => {
            runtime::record_length(instruction(value, node, rules, context)?)
        }
        Instruction::RecordIndex { left, right } => {
            binary_instruction(left, right, node, rules, context, runtime::record_index)
        }
        Instruction::Index { left, right } => {
            binary_instruction(left, right, node, rules, context, runtime::index)
        }
        Instruction::ListSlice {
            value,
            lower,
            upper,
        } => slice_instruction([value, lower, upper], node, rules, context),
        Instruction::ListIndex { left, right } => {
            binary_instruction(left, right, node, rules, context, runtime::list_index)
        }
        Instruction::StringLength { value } => {
            runtime::string_length(instruction(value, node, rules, context)?)
        }
        Instruction::ListLength { value } => {
            runtime::list_length(instruction(value, node, rules, context)?)
        }
        Instruction::LogicalNot { value } => {
            runtime::logical_not(instruction(value, node, rules, context)?)
        }
        Instruction::LogicalAnd { left, right } => {
            let left = instruction(left, node, rules, context)?;
            let right = instruction(right, node, rules, context)?;
            runtime::logical_and(left, right)
        }
        Instruction::LogicalOr { left, right } => {
            let left = instruction(left, node, rules, context)?;
            let right = instruction(right, node, rules, context)?;
            runtime::logical_or(left, right)
        }
        Instruction::Less { left, right } => {
            let left = instruction(left, node, rules, context)?;
            let right = instruction(right, node, rules, context)?;
            runtime::less(left, right)
        }
        Instruction::LessEqual { left, right } => {
            let left = instruction(left, node, rules, context)?;
            let right = instruction(right, node, rules, context)?;
            runtime::less_equal(left, right)
        }
        Instruction::Greater { left, right } => {
            let left = instruction(left, node, rules, context)?;
            let right = instruction(right, node, rules, context)?;
            runtime::greater(left, right)
        }
        Instruction::GreaterEqual { left, right } => {
            let left = instruction(left, node, rules, context)?;
            let right = instruction(right, node, rules, context)?;
            runtime::greater_equal(left, right)
        }
        Instruction::Equal { left, right } => {
            let left = instruction(left, node, rules, context)?;
            let right = instruction(right, node, rules, context)?;
            runtime::equal(left, right)
        }
        Instruction::NotEqual { left, right } => {
            let left = instruction(left, node, rules, context)?;
            let right = instruction(right, node, rules, context)?;
            runtime::not_equal(left, right)
        }
        Instruction::Unify { left, right } => {
            let left = instruction(left, node, rules, context)?;
            let right = instruction(right, node, rules, context)?;
            runtime::unify(left, right)
        }
    }
}
