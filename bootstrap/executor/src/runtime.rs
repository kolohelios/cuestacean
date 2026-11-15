//! Native primitives shared by interpreted and generated evaluators.
//! No expression kinds or evaluation rules are defined here.

use serde_json::Value;
use snafu::ResultExt;

use crate::{Error, JsonSnafu, Program, VersionSnafu};

pub fn program(input: &str) -> Result<Value, Error> {
    let program: Program = serde_json::from_str(input).context(JsonSnafu {
        document: "program",
    })?;
    if program.format_version != 1 {
        return VersionSnafu {
            document: "program",
            version: program.format_version,
        }
        .fail();
    }
    Ok(program.entry)
}

pub fn kind(node: &Value) -> Result<&str, Error> {
    node.get("kind").and_then(Value::as_str).ok_or(Error::Kind)
}

pub fn field<'a>(node: &'a Value, name: &str) -> Result<&'a Value, Error> {
    node.get(name).ok_or_else(|| Error::MissingField {
        field: name.to_string(),
    })
}

pub fn integer(node: &Value, name: &str) -> Result<i64, Error> {
    field(node, name)?.as_i64().ok_or_else(|| Error::Integer {
        field: name.to_string(),
    })
}

pub fn sum(left: i64, right: i64) -> Result<i64, Error> {
    left.checked_add(right).ok_or(Error::Overflow)
}

pub fn product(left: i64, right: i64) -> Result<i64, Error> {
    left.checked_mul(right).ok_or(Error::ProductOverflow)
}
