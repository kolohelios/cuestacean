//! Disposable bootstrap machinery for interpreting exported rule instructions.
//! Node kinds and field names come from data, not Rust expression variants.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;
use snafu::{ResultExt, Snafu};

mod codegen;
pub mod parser;
pub mod runtime;

pub use codegen::generate;

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
    #[snafu(display("signed 64-bit integer multiplication overflow"))]
    ProductOverflow,
    #[snafu(display("conflicting integer values {left} and {right}"))]
    Conflict { left: i64, right: i64 },
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
    Field {
        field: String,
    },
    Evaluate {
        field: String,
    },
    Sum {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    Product {
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
pub fn execute(program: &str, rules: &str) -> Result<i64, Error> {
    let program = runtime::program(program)?;
    let rules = parse_rules(rules)?;
    evaluate(&program, &rules.rules)
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

fn evaluate(node: &Value, rules: &BTreeMap<String, Rule>) -> Result<i64, Error> {
    let kind = runtime::kind(node)?;
    let rule = rules.get(kind).ok_or_else(|| Error::MissingRule {
        kind: kind.to_string(),
    })?;
    instruction(&rule.result, node, rules)
}

fn instruction(
    op: &Instruction,
    node: &Value,
    rules: &BTreeMap<String, Rule>,
) -> Result<i64, Error> {
    match op {
        Instruction::Field { field: name } => runtime::integer(node, name),
        Instruction::Evaluate { field: name } => evaluate(runtime::field(node, name)?, rules),
        Instruction::Sum { left, right } => {
            let left = instruction(left, node, rules)?;
            let right = instruction(right, node, rules)?;
            runtime::sum(left, right)
        }
        Instruction::Product { left, right } => {
            let left = instruction(left, node, rules)?;
            let right = instruction(right, node, rules)?;
            runtime::product(left, right)
        }
        Instruction::Unify { left, right } => {
            let left = instruction(left, node, rules)?;
            let right = instruction(right, node, rules)?;
            runtime::unify(left, right)
        }
    }
}
