//! Value primitives shared by interpreted and generated evaluators.
//! Unification dispatches CUE-generated control; other primitives remain native.

use serde_json::Value as JsonValue;
use snafu::ResultExt;

use crate::{Error, JsonSnafu, Program, VersionSnafu};

#[cfg(feature = "semantics")]
#[path = "../../generated/unify.rs"]
mod generated_unification;

/// Copy scalar scaffold: `Int`/`Bool` are abstract, not conflicting values.
/// Concrete integers are still restricted to the bootstrap i64 domain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Value {
    Integer(i64),
    Int,
    Boolean(bool),
    Bool,
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

/// Legacy integer output; general scalar consumers should use `concrete_json`.
pub fn concrete(value: Value) -> Result<i64, Error> {
    match value {
        Value::Integer(value) => Ok(value),
        Value::Int => Err(Error::Incomplete),
        Value::Boolean(_) => Err(Error::IntegerOutput),
        Value::Bool => Err(Error::IncompleteBool),
        Value::Bounds { .. } => Err(Error::IncompleteBounds),
    }
}

/// Preserve exact integer/boolean JSON scalars; reject incomplete values.
pub fn concrete_json(value: Value) -> Result<JsonValue, Error> {
    match value {
        Value::Boolean(value) => Ok(JsonValue::Bool(value)),
        value => concrete(value).map(JsonValue::from),
    }
}

fn operands(left: Value, right: Value) -> Result<(i64, i64), Error> {
    match (left, right) {
        (Value::Integer(left), Value::Integer(right)) => Ok((left, right)),
        _ => Err(Error::Arithmetic),
    }
}

pub fn negate(value: Value) -> Result<Value, Error> {
    let Value::Integer(value) = value else {
        return Err(Error::Arithmetic);
    };
    value
        .checked_neg()
        .map(Value::Integer)
        .ok_or(Error::NegationOverflow)
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

fn division_operands(left: Value, right: Value) -> Result<(i64, i64), Error> {
    let (left, right) = operands(left, right)?;
    if right == 0 {
        return Err(Error::DivisionByZero);
    }
    Ok((left, right))
}

pub fn divide(left: Value, right: Value) -> Result<Value, Error> {
    let (left, right) = division_operands(left, right)?;
    left.checked_div_euclid(right)
        .map(Value::Integer)
        .ok_or(Error::DivisionOverflow)
}

pub fn modulo(left: Value, right: Value) -> Result<Value, Error> {
    let (left, right) = division_operands(left, right)?;
    if right == -1 {
        return Ok(Value::Integer(0));
    }
    left.checked_rem_euclid(right)
        .map(Value::Integer)
        .ok_or(Error::DivisionOverflow)
}

pub fn quotient(left: Value, right: Value) -> Result<Value, Error> {
    let (left, right) = division_operands(left, right)?;
    left.checked_div(right)
        .map(Value::Integer)
        .ok_or(Error::DivisionOverflow)
}

pub fn remainder(left: Value, right: Value) -> Result<Value, Error> {
    let (left, right) = division_operands(left, right)?;
    if right == -1 {
        return Ok(Value::Integer(0));
    }
    left.checked_rem(right)
        .map(Value::Integer)
        .ok_or(Error::DivisionOverflow)
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
