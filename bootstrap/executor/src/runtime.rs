//! Value primitives shared by interpreted and generated evaluators.
//! Unification dispatches CUE-generated control; other primitives remain native.

use serde_json::Value as JsonValue;
use snafu::ResultExt;

use crate::{Error, JsonSnafu, Program, VersionSnafu};

#[cfg(feature = "semantics")]
#[path = "../../generated/unify.rs"]
mod generated_unification;

/// Initial native value scaffold. `Int` is abstract, not a conflicting value.
/// Concrete integers are still restricted to the bootstrap i64 domain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Value {
    Integer(i64),
    Int,
    // Missing endpoints are infinite, not i64::MIN / i64::MAX. Even equal
    // endpoints remain abstract until unified with a concrete candidate.
    Bounds {
        lower: Option<i64>,
        upper: Option<i64>,
    },
}

pub fn program(input: &str) -> Result<JsonValue, Error> {
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

pub fn kind(node: &JsonValue) -> Result<&str, Error> {
    node.get("kind")
        .and_then(JsonValue::as_str)
        .ok_or(Error::Kind)
}

pub fn field<'a>(node: &'a JsonValue, name: &str) -> Result<&'a JsonValue, Error> {
    node.get(name).ok_or_else(|| Error::MissingField {
        field: name.to_string(),
    })
}

pub fn integer(node: &JsonValue, name: &str) -> Result<Value, Error> {
    field(node, name)?
        .as_i64()
        .map(Value::Integer)
        .ok_or_else(|| Error::Integer {
            field: name.to_string(),
        })
}

/// Scalar JSON output requires concreteness only at the outer boundary.
pub fn concrete(value: Value) -> Result<i64, Error> {
    match value {
        Value::Integer(value) => Ok(value),
        Value::Int => Err(Error::Incomplete),
        Value::Bounds { .. } => Err(Error::IncompleteBounds),
    }
}

fn operands(left: Value, right: Value) -> Result<(i64, i64), Error> {
    match (left, right) {
        (Value::Integer(left), Value::Integer(right)) => Ok((left, right)),
        _ => Err(Error::Arithmetic),
    }
}

pub fn sum(left: Value, right: Value) -> Result<Value, Error> {
    let (left, right) = operands(left, right)?;
    left.checked_add(right)
        .map(Value::Integer)
        .ok_or(Error::Overflow)
}

pub fn difference(left: Value, right: Value) -> Result<Value, Error> {
    let (left, right) = operands(left, right)?;
    left.checked_sub(right)
        .map(Value::Integer)
        .ok_or(Error::DifferenceOverflow)
}

pub fn product(left: Value, right: Value) -> Result<Value, Error> {
    let (left, right) = operands(left, right)?;
    left.checked_mul(right)
        .map(Value::Integer)
        .ok_or(Error::ProductOverflow)
}

pub fn lower_bound(value: Value) -> Result<Value, Error> {
    let Value::Integer(value) = value else {
        return Err(Error::BoundThreshold);
    };
    Ok(Value::Bounds {
        lower: Some(value),
        upper: None,
    })
}

pub fn upper_bound(value: Value) -> Result<Value, Error> {
    let Value::Integer(value) = value else {
        return Err(Error::BoundThreshold);
    };
    Ok(Value::Bounds {
        lower: None,
        upper: Some(value),
    })
}

/// Unify using the generated CUE policy; compiler-only seeds exclude this API.
#[cfg(feature = "semantics")]
pub fn unify(left: Value, right: Value) -> Result<Value, Error> {
    generated_unification::unify(left, right)
}
