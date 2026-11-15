//! Native primitives shared by interpreted and generated evaluators.
//! No expression kinds or evaluation rules are defined here.

use serde_json::Value as JsonValue;
use snafu::ResultExt;

use crate::{Error, JsonSnafu, Program, VersionSnafu};

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

fn endpoints(value: Value) -> (Option<i64>, Option<i64>) {
    match value {
        Value::Integer(value) => (Some(value), Some(value)),
        Value::Int => (None, None),
        Value::Bounds { lower, upper } => (lower, upper),
    }
}

pub fn unify(left: Value, right: Value) -> Result<Value, Error> {
    match (left, right) {
        (Value::Int, value) | (value, Value::Int) => Ok(value),
        (Value::Integer(left), Value::Integer(right)) if left == right => Ok(Value::Integer(left)),
        (Value::Integer(left), Value::Integer(right)) => Err(Error::Conflict { left, right }),
        (left, right) => {
            let (left_lower, left_upper) = endpoints(left);
            let (right_lower, right_upper) = endpoints(right);
            let lower = left_lower.max(right_lower);
            let upper = match (left_upper, right_upper) {
                (Some(left), Some(right)) => Some(left.min(right)),
                (left, right) => left.or(right),
            };
            if matches!((lower, upper), (Some(lower), Some(upper)) if lower > upper) {
                return Err(Error::BoundsConflict);
            }
            // Bounds alone do not materialize a singleton in upstream CUE.
            match (left, right) {
                (value @ Value::Integer(_), _) | (_, value @ Value::Integer(_)) => Ok(value),
                _ => Ok(Value::Bounds { lower, upper }),
            }
        }
    }
}
