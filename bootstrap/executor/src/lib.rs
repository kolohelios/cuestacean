//! Disposable bootstrap machinery for interpreting exported rule instructions.
//! Node kinds and field names come from data, not Rust expression variants.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;
use snafu::{ResultExt, Snafu};

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
}

/// Execute version-1 JSON documents using checked signed 64-bit arithmetic.
/// This executes accessed fields; it does not validate the complete CUE AST.
pub fn execute(program: &str, rules: &str) -> Result<i64, Error> {
    let program: Program = serde_json::from_str(program).context(JsonSnafu {
        document: "program",
    })?;
    let rules: Rules = serde_json::from_str(rules).context(JsonSnafu { document: "rules" })?;
    for (document, version) in [
        ("program", program.format_version),
        ("rules", rules.format_version),
    ] {
        if version != 1 {
            return VersionSnafu { document, version }.fail();
        }
    }
    evaluate(&program.entry, &rules.rules)
}

fn evaluate(node: &Value, rules: &BTreeMap<String, Rule>) -> Result<i64, Error> {
    let kind = node
        .get("kind")
        .and_then(Value::as_str)
        .ok_or(Error::Kind)?;
    let rule = rules.get(kind).ok_or_else(|| Error::MissingRule {
        kind: kind.to_string(),
    })?;
    instruction(&rule.result, node, rules)
}

fn field<'a>(node: &'a Value, name: &str) -> Result<&'a Value, Error> {
    node.get(name).ok_or_else(|| Error::MissingField {
        field: name.to_string(),
    })
}

fn instruction(
    op: &Instruction,
    node: &Value,
    rules: &BTreeMap<String, Rule>,
) -> Result<i64, Error> {
    match op {
        Instruction::Field { field: name } => {
            field(node, name)?.as_i64().ok_or_else(|| Error::Integer {
                field: name.clone(),
            })
        }
        Instruction::Evaluate { field: name } => evaluate(field(node, name)?, rules),
        Instruction::Sum { left, right } => {
            let left = instruction(left, node, rules)?;
            let right = instruction(right, node, rules)?;
            left.checked_add(right).ok_or(Error::Overflow)
        }
    }
}
