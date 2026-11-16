//! Value primitives shared by interpreted and generated evaluators.
//! Scalar operations dispatch CUE-generated control over native kernels.

use std::sync::Arc;

use serde_json::Value as JsonValue;
use snafu::ResultExt;

use crate::{Error, JsonSnafu, Program, VersionSnafu};

#[cfg(feature = "semantics")]
#[path = "../../generated/unify.rs"]
mod generated_unification;

#[cfg(feature = "semantics")]
#[path = "../../generated/sum.rs"]
mod generated_sum;

#[cfg(feature = "semantics")]
#[path = "../../generated/equal.rs"]
mod generated_equal;

#[cfg(feature = "semantics")]
#[path = "../../generated/not-equal.rs"]
mod generated_not_equal;

#[cfg(feature = "semantics")]
#[path = "../../generated/greater.rs"]
mod generated_greater;
#[cfg(feature = "semantics")]
#[path = "../../generated/greater-equal.rs"]
mod generated_greater_equal;
#[cfg(feature = "semantics")]
#[path = "../../generated/less.rs"]
mod generated_less;
#[cfg(feature = "semantics")]
#[path = "../../generated/less-equal.rs"]
mod generated_less_equal;

/// Owned scalar scaffold; abstract types are not conflicting values.
/// Concrete integers are still restricted to the bootstrap i64 domain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Integer(i64),
    Int,
    Boolean(bool),
    Bool,
    Text(Arc<str>),
    String,
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

pub fn text(node: &JsonValue, name: &str) -> Result<Value, Error> {
    field(node, name)?
        .as_str()
        .map(|value| Value::Text(Arc::from(value)))
        .ok_or_else(|| Error::Text {
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
        Value::Text(_) => Err(Error::IntegerOutput),
        Value::String => Err(Error::IncompleteString),
        Value::Bounds { .. } => Err(Error::IncompleteBounds),
    }
}

/// Preserve exact concrete JSON scalars; reject incomplete values.
pub fn concrete_json(value: Value) -> Result<JsonValue, Error> {
    match value {
        Value::Boolean(value) => Ok(JsonValue::Bool(value)),
        Value::Text(value) => Ok(JsonValue::String(value.as_ref().to_owned())),
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

/// Checked integer primitive; overload selection belongs to CUE control.
pub fn integer_sum(left: &Value, right: &Value) -> Result<Value, Error> {
    let (Value::Integer(left), Value::Integer(right)) = (left, right) else {
        return Err(Error::Arithmetic);
    };
    left.checked_add(*right)
        .map(Value::Integer)
        .ok_or(Error::Overflow)
}

fn text_buffer(left_len: usize, right_len: usize) -> Result<String, Error> {
    let capacity = left_len.checked_add(right_len).ok_or(Error::TextCapacity)?;
    let mut buffer = String::new();
    buffer
        .try_reserve_exact(capacity)
        .map_err(|_| Error::TextCapacity)?;
    Ok(buffer)
}

/// Checked text primitive; concatenation preserves the original UTF-8 bytes.
pub fn text_concat(left: &Value, right: &Value) -> Result<Value, Error> {
    let (Value::Text(left), Value::Text(right)) = (left, right) else {
        return Err(Error::Arithmetic);
    };
    let mut buffer = text_buffer(left.len(), right.len())?;
    buffer.push_str(left);
    buffer.push_str(right);
    Ok(Value::Text(Arc::from(buffer)))
}

/// Add using generated CUE policy; compiler-only seeds exclude this API.
#[cfg(feature = "semantics")]
pub fn sum(left: Value, right: Value) -> Result<Value, Error> {
    generated_sum::sum(left, right)
}

/// Compare using generated CUE policy; compiler-only seeds exclude this API.
#[cfg(feature = "semantics")]
pub fn equal(left: Value, right: Value) -> Result<Value, Error> {
    generated_equal::equal(left, right)
}

/// Compare using generated CUE policy; compiler-only seeds exclude this API.
#[cfg(feature = "semantics")]
pub fn not_equal(left: Value, right: Value) -> Result<Value, Error> {
    generated_not_equal::not_equal(left, right)
}

/// Order using generated CUE policy; compiler-only seeds exclude this API.
#[cfg(feature = "semantics")]
pub fn less(left: Value, right: Value) -> Result<Value, Error> {
    generated_less::less(left, right)
}

#[cfg(feature = "semantics")]
pub fn less_equal(left: Value, right: Value) -> Result<Value, Error> {
    generated_less_equal::less_equal(left, right)
}

#[cfg(feature = "semantics")]
pub fn greater(left: Value, right: Value) -> Result<Value, Error> {
    generated_greater::greater(left, right)
}

#[cfg(feature = "semantics")]
pub fn greater_equal(left: Value, right: Value) -> Result<Value, Error> {
    generated_greater_equal::greater_equal(left, right)
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

#[cfg(test)]
mod tests {
    use super::{Value, integer_sum, text_buffer, text_concat};
    use crate::Error;
    use std::sync::Arc;

    #[test]
    fn checked_sum_and_concat_kernels() {
        assert_eq!(
            integer_sum(&Value::Integer(20), &Value::Integer(22)).ok(),
            Some(Value::Integer(42))
        );
        assert!(matches!(
            integer_sum(&Value::Integer(i64::MAX), &Value::Integer(1)),
            Err(Error::Overflow)
        ));
        let left: Arc<str> = Arc::from("é");
        let value = Value::Text(Arc::clone(&left));
        let result = text_concat(&value, &Value::Text(Arc::from("\u{301}")));
        assert_eq!(result.ok(), Some(Value::Text(Arc::from("é\u{301}"))));
        assert_eq!(&*left, "é");
        assert!(matches!(
            text_concat(&value, &Value::Int),
            Err(Error::Arithmetic)
        ));
        assert!(matches!(
            integer_sum(&value, &Value::Int),
            Err(Error::Arithmetic)
        ));
        assert!(matches!(
            text_buffer(usize::MAX, 1),
            Err(Error::TextCapacity)
        ));
        assert!(matches!(
            text_buffer(usize::MAX, 0),
            Err(Error::TextCapacity)
        ));
    }
}
