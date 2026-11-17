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
#[path = "../../generated/difference-control.rs"]
mod generated_difference;
#[cfg(feature = "semantics")]
#[path = "../../generated/divide-control.rs"]
mod generated_divide;
#[cfg(feature = "semantics")]
#[path = "../../generated/modulo-control.rs"]
mod generated_modulo;
#[cfg(feature = "semantics")]
#[path = "../../generated/negate-control.rs"]
mod generated_negate;
#[cfg(feature = "semantics")]
#[path = "../../generated/product-control.rs"]
mod generated_product;
#[cfg(feature = "semantics")]
#[path = "../../generated/quotient-control.rs"]
mod generated_quotient;
#[cfg(feature = "semantics")]
#[path = "../../generated/remainder-control.rs"]
mod generated_remainder;
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
#[path = "../../generated/integer-output.rs"]
mod generated_integer_output;
#[cfg(feature = "semantics")]
#[path = "../../generated/json-output.rs"]
mod generated_json_output;
#[cfg(feature = "semantics")]
#[path = "../../generated/less.rs"]
mod generated_less;
#[cfg(feature = "semantics")]
#[path = "../../generated/less-equal.rs"]
mod generated_less_equal;
#[cfg(feature = "semantics")]
#[path = "../../generated/logical-and.rs"]
mod generated_logical_and;
#[cfg(feature = "semantics")]
#[path = "../../generated/logical-not.rs"]
mod generated_logical_not;
#[cfg(feature = "semantics")]
#[path = "../../generated/logical-or.rs"]
mod generated_logical_or;

/// Owned scalar scaffold; abstract types are not conflicting values.
/// Concrete integers are still restricted to the bootstrap i64 domain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Null,
    Top,
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

/// Legacy integer output admitted by generated CUE policy.
#[cfg(feature = "semantics")]
pub fn concrete(value: Value) -> Result<i64, Error> {
    encode_integer(generated_integer_output::integer_output(value)?)
}

/// Preserve exact JSON scalars admitted by generated CUE policy.
#[cfg(feature = "semantics")]
pub fn concrete_json(value: Value) -> Result<JsonValue, Error> {
    encode_json(generated_json_output::json_output(value)?)
}

/// Extract an admitted integer; unsupported representations indicate a policy error.
pub fn encode_integer(value: Value) -> Result<i64, Error> {
    match value {
        Value::Integer(value) => Ok(value),
        _ => Err(Error::OutputPolicy),
    }
}

/// Encode admitted JSON scalars without choosing incomplete-value diagnostics.
pub fn encode_json(value: Value) -> Result<JsonValue, Error> {
    match value {
        Value::Null => Ok(JsonValue::Null),
        Value::Integer(value) => Ok(JsonValue::from(value)),
        Value::Boolean(value) => Ok(JsonValue::Bool(value)),
        Value::Text(value) => Ok(JsonValue::String(value.as_ref().to_owned())),
        _ => Err(Error::OutputPolicy),
    }
}

/// Checked integer negation primitive; dispatch belongs to CUE control.
pub fn integer_negate(value: &Value) -> Result<Value, Error> {
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

/// Select a logical value using generated CUE policy.
#[cfg(feature = "semantics")]
pub fn logical_and(left: Value, right: Value) -> Result<Value, Error> {
    generated_logical_and::logical_and(left, right)
}

/// Select a logical value using generated CUE policy.
#[cfg(feature = "semantics")]
pub fn logical_or(left: Value, right: Value) -> Result<Value, Error> {
    generated_logical_or::logical_or(left, right)
}

/// Invert a logical value using generated CUE policy.
#[cfg(feature = "semantics")]
pub fn logical_not(value: Value) -> Result<Value, Error> {
    generated_logical_not::logical_not(value)
}

/// Negate using generated CUE policy.
#[cfg(feature = "semantics")]
pub fn negate(value: Value) -> Result<Value, Error> {
    generated_negate::negate(value)
}

/// Subtract using generated CUE policy.
#[cfg(feature = "semantics")]
pub fn difference(left: Value, right: Value) -> Result<Value, Error> {
    generated_difference::difference(left, right)
}

/// Multiply using generated CUE policy.
#[cfg(feature = "semantics")]
pub fn product(left: Value, right: Value) -> Result<Value, Error> {
    generated_product::product(left, right)
}

/// Checked integer subtraction primitive; dispatch belongs to CUE control.
pub fn integer_difference(left: &Value, right: &Value) -> Result<Value, Error> {
    let (Value::Integer(left), Value::Integer(right)) = (left, right) else {
        return Err(Error::Arithmetic);
    };
    left.checked_sub(*right)
        .map(Value::Integer)
        .ok_or(Error::DifferenceOverflow)
}

/// Checked integer multiplication primitive; dispatch belongs to CUE control.
pub fn integer_product(left: &Value, right: &Value) -> Result<Value, Error> {
    let (Value::Integer(left), Value::Integer(right)) = (left, right) else {
        return Err(Error::Arithmetic);
    };
    left.checked_mul(*right)
        .map(Value::Integer)
        .ok_or(Error::ProductOverflow)
}

/// Divide using generated CUE policy.
#[cfg(feature = "semantics")]
pub fn divide(left: Value, right: Value) -> Result<Value, Error> {
    generated_divide::divide(left, right)
}

/// Compute Euclidean remainder using generated CUE policy.
#[cfg(feature = "semantics")]
pub fn modulo(left: Value, right: Value) -> Result<Value, Error> {
    generated_modulo::modulo(left, right)
}

/// Compute truncating quotient using generated CUE policy.
#[cfg(feature = "semantics")]
pub fn quotient(left: Value, right: Value) -> Result<Value, Error> {
    generated_quotient::quotient(left, right)
}

/// Compute truncating remainder using generated CUE policy.
#[cfg(feature = "semantics")]
pub fn remainder(left: Value, right: Value) -> Result<Value, Error> {
    generated_remainder::remainder(left, right)
}

/// Checked Euclidean division primitive; dispatch belongs to CUE control.
pub fn integer_divide(left: &Value, right: &Value) -> Result<Value, Error> {
    let (Value::Integer(left), Value::Integer(right)) = (left, right) else {
        return Err(Error::Arithmetic);
    };
    if *right == 0 {
        return Err(Error::DivisionByZero);
    }
    left.checked_div_euclid(*right)
        .map(Value::Integer)
        .ok_or(Error::DivisionOverflow)
}

/// Checked Euclidean remainder primitive; unit-divisor policy belongs to CUE.
pub fn integer_modulo(left: &Value, right: &Value) -> Result<Value, Error> {
    let (Value::Integer(left), Value::Integer(right)) = (left, right) else {
        return Err(Error::Arithmetic);
    };
    if *right == 0 {
        return Err(Error::DivisionByZero);
    }
    left.checked_rem_euclid(*right)
        .map(Value::Integer)
        .ok_or(Error::DivisionOverflow)
}

/// Checked truncating division primitive; dispatch belongs to CUE control.
pub fn integer_quotient(left: &Value, right: &Value) -> Result<Value, Error> {
    let (Value::Integer(left), Value::Integer(right)) = (left, right) else {
        return Err(Error::Arithmetic);
    };
    if *right == 0 {
        return Err(Error::DivisionByZero);
    }
    left.checked_div(*right)
        .map(Value::Integer)
        .ok_or(Error::DivisionOverflow)
}

/// Checked truncating remainder primitive; unit-divisor policy belongs to CUE.
pub fn integer_remainder(left: &Value, right: &Value) -> Result<Value, Error> {
    let (Value::Integer(left), Value::Integer(right)) = (left, right) else {
        return Err(Error::Arithmetic);
    };
    if *right == 0 {
        return Err(Error::DivisionByZero);
    }
    left.checked_rem(*right)
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
    #[cfg(feature = "semantics")]
    fn null_and_top_output() {
        assert!(matches!(
            super::concrete(Value::Null),
            Err(Error::IntegerOutput)
        ));
        assert!(matches!(
            super::concrete(Value::Top),
            Err(Error::IncompleteValue)
        ));
        assert_eq!(
            super::concrete_json(Value::Null).ok(),
            Some(serde_json::Value::Null)
        );
        assert!(matches!(
            super::concrete_json(Value::Top),
            Err(Error::IncompleteValue)
        ));
    }

    #[test]
    fn checked_output_encoders() {
        for value in [i64::MIN, 0, i64::MAX] {
            assert_eq!(
                super::encode_integer(Value::Integer(value)).ok(),
                Some(value)
            );
            assert_eq!(
                super::encode_json(Value::Integer(value)).ok(),
                Some(serde_json::Value::from(value))
            );
        }
        assert_eq!(
            super::encode_json(Value::Null).ok(),
            Some(serde_json::Value::Null)
        );
        for value in [false, true] {
            assert_eq!(
                super::encode_json(Value::Boolean(value)).ok(),
                Some(serde_json::Value::Bool(value))
            );
        }
        let text: Arc<str> = Arc::from("é\n\"\\");
        assert_eq!(
            super::encode_json(Value::Text(Arc::clone(&text))).ok(),
            Some(serde_json::Value::String("é\n\"\\".to_owned()))
        );
        assert_eq!(Arc::strong_count(&text), 1);
        for value in [
            Value::Top,
            Value::Int,
            Value::Bool,
            Value::String,
            Value::Bounds {
                lower: Some(0),
                upper: Some(0),
            },
        ] {
            assert!(matches!(
                super::encode_json(value.clone()),
                Err(Error::OutputPolicy)
            ));
            assert!(matches!(
                super::encode_integer(value),
                Err(Error::OutputPolicy)
            ));
        }
        for value in [Value::Null, Value::Boolean(false), Value::Text(text)] {
            assert!(matches!(
                super::encode_integer(value),
                Err(Error::OutputPolicy)
            ));
        }
        assert_eq!(
            Error::OutputPolicy.to_string(),
            "output policy returned an unsupported value"
        );
    }

    #[test]
    fn checked_integer_kernels() {
        use super::{integer_difference, integer_negate, integer_product};

        assert_eq!(
            integer_difference(&Value::Integer(i64::MIN), &Value::Integer(i64::MIN)).ok(),
            Some(Value::Integer(0))
        );
        assert!(matches!(
            integer_negate(&Value::Integer(i64::MIN)),
            Err(Error::NegationOverflow)
        ));
        assert!(matches!(
            integer_difference(&Value::Integer(i64::MAX), &Value::Integer(-1)),
            Err(Error::DifferenceOverflow)
        ));
        assert!(matches!(
            integer_product(&Value::Integer(i64::MIN), &Value::Integer(-1)),
            Err(Error::ProductOverflow)
        ));
        for value in [
            Value::Null,
            Value::Top,
            Value::Int,
            Value::Bool,
            Value::Bounds {
                lower: None,
                upper: None,
            },
            Value::Text(Arc::from("x")),
        ] {
            assert!(matches!(integer_negate(&value), Err(Error::Arithmetic)));
            assert!(matches!(
                integer_difference(&value, &Value::Integer(1)),
                Err(Error::Arithmetic)
            ));
            assert!(matches!(
                integer_product(&Value::Integer(1), &value),
                Err(Error::Arithmetic)
            ));
        }
    }

    #[test]
    fn checked_division_kernels() {
        use super::{integer_divide, integer_modulo, integer_quotient, integer_remainder};

        for (kernel, expected) in [
            (
                integer_divide as fn(&Value, &Value) -> Result<Value, Error>,
                -3,
            ),
            (integer_modulo, 2),
            (integer_quotient, -2),
            (integer_remainder, -1),
        ] {
            assert_eq!(
                kernel(&Value::Integer(-7), &Value::Integer(3)).ok(),
                Some(Value::Integer(expected))
            );
            assert!(matches!(
                kernel(&Value::Integer(i64::MIN), &Value::Integer(-1)),
                Err(Error::DivisionOverflow)
            ));
            assert!(matches!(
                kernel(&Value::Integer(1), &Value::Integer(0)),
                Err(Error::DivisionByZero)
            ));
            for value in [
                Value::Null,
                Value::Top,
                Value::Int,
                Value::Bool,
                Value::Boolean(false),
                Value::String,
                Value::Text(Arc::from("x")),
                Value::Bounds {
                    lower: Some(0),
                    upper: Some(0),
                },
            ] {
                assert!(matches!(
                    kernel(&value, &Value::Integer(0)),
                    Err(Error::Arithmetic)
                ));
                assert!(matches!(
                    kernel(&Value::Integer(1), &value),
                    Err(Error::Arithmetic)
                ));
            }
        }
    }

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
