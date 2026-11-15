//! Disposable bootstrap machinery for interpreting exported rule instructions.
//! Node kinds and field names come from data, not Rust expression variants.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;
use snafu::{ResultExt, Snafu};

pub mod ast;
mod codegen;
#[cfg(feature = "source")]
pub mod parser;
pub mod pratt;
pub mod runtime;
pub mod scanner;
pub mod unification;

pub use codegen::generate;

#[cfg(feature = "semantics")]
extern crate self as bootstrap_executor;

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
    #[snafu(display("expression must have a string kind"))]
    Kind,
    #[snafu(display("no rule for node kind {kind:?}"))]
    MissingRule { kind: String },
    #[snafu(display("missing field {field:?}"))]
    MissingField { field: String },
    #[snafu(display("field {field:?} must be a signed 64-bit integer"))]
    Integer { field: String },
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
    #[snafu(display("conflicting boolean values {left} and {right}"))]
    BooleanConflict { left: bool, right: bool },
    #[snafu(display("conflicting value types"))]
    TypeConflict,
    #[snafu(display("non-concrete bool value"))]
    IncompleteBool,
    #[snafu(display("integer output requires an integer value"))]
    IntegerOutput,
    #[snafu(display("conflicting integer bounds"))]
    BoundsConflict,
    #[snafu(display("non-concrete integer bounds"))]
    IncompleteBounds,
    #[snafu(display("bound threshold requires a concrete integer"))]
    BoundThreshold,
    #[snafu(display("arithmetic requires concrete integer operands"))]
    Arithmetic,
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

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "lowercase", deny_unknown_fields)]
enum Instruction {
    Int {},
    Bool {},
    Boolean {
        value: bool,
    },
    #[serde(rename = "lowerBound")]
    LowerBound {
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
    runtime::concrete(evaluate(&program, &rules.rules)?)
}

/// Execute to an exact concrete JSON scalar (integer or boolean).
#[cfg(feature = "semantics")]
pub fn execute_json(program: &str, rules: &str) -> Result<Value, Error> {
    let program = runtime::program(program)?;
    let rules = parse_rules(rules)?;
    runtime::concrete_json(evaluate(&program, &rules.rules)?)
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
fn evaluate(node: &Value, rules: &BTreeMap<String, Rule>) -> Result<runtime::Value, Error> {
    let kind = runtime::kind(node)?;
    let rule = rules.get(kind).ok_or_else(|| Error::MissingRule {
        kind: kind.to_string(),
    })?;
    instruction(&rule.result, node, rules)
}

#[cfg(feature = "semantics")]
fn instruction(
    op: &Instruction,
    node: &Value,
    rules: &BTreeMap<String, Rule>,
) -> Result<runtime::Value, Error> {
    match op {
        Instruction::Int {} => Ok(runtime::Value::Int),
        Instruction::Bool {} => Ok(runtime::Value::Bool),
        Instruction::Boolean { value } => Ok(runtime::Value::Boolean(*value)),
        Instruction::LowerBound { value } => runtime::lower_bound(instruction(value, node, rules)?),
        Instruction::UpperBound { value } => runtime::upper_bound(instruction(value, node, rules)?),
        Instruction::Field { field: name } => runtime::integer(node, name),
        Instruction::Evaluate { field: name } => evaluate(runtime::field(node, name)?, rules),
        Instruction::Negate { value } => runtime::negate(instruction(value, node, rules)?),
        Instruction::Sum { left, right } => {
            let left = instruction(left, node, rules)?;
            let right = instruction(right, node, rules)?;
            runtime::sum(left, right)
        }
        Instruction::Difference { left, right } => {
            let left = instruction(left, node, rules)?;
            let right = instruction(right, node, rules)?;
            runtime::difference(left, right)
        }
        Instruction::Product { left, right } => {
            let left = instruction(left, node, rules)?;
            let right = instruction(right, node, rules)?;
            runtime::product(left, right)
        }
        Instruction::Divide { left, right } => {
            let left = instruction(left, node, rules)?;
            let right = instruction(right, node, rules)?;
            runtime::divide(left, right)
        }
        Instruction::Modulo { left, right } => {
            let left = instruction(left, node, rules)?;
            let right = instruction(right, node, rules)?;
            runtime::modulo(left, right)
        }
        Instruction::Quotient { left, right } => {
            let left = instruction(left, node, rules)?;
            let right = instruction(right, node, rules)?;
            runtime::quotient(left, right)
        }
        Instruction::Remainder { left, right } => {
            let left = instruction(left, node, rules)?;
            let right = instruction(right, node, rules)?;
            runtime::remainder(left, right)
        }
        Instruction::Unify { left, right } => {
            let left = instruction(left, node, rules)?;
            let right = instruction(right, node, rules)?;
            runtime::unify(left, right)
        }
    }
}
