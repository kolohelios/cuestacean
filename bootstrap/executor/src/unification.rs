//! Finite CUE-authored control for scalar operations and its lowering.
use crate::runtime::Value;
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("invalid unification control JSON: {source}"))]
    Json { source: serde_json::Error },
    #[snafu(display("{source}"))]
    Lowering { source: crate::lowering::Error },
    #[snafu(display("invalid unification control: {reason}"))]
    Program { reason: &'static str },
}
#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
enum Side {
    Left,
    Right,
}
#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Endpoint {
    Lower,
    Upper,
}
#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    Null,
    Top,
    Integer,
    Int,
    Bounds,
    Boolean,
    Bool,
    Text,
    String,
}
#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Relation {
    Equal,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}
#[derive(Clone, Copy, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
enum Function {
    #[default]
    Unify,
    Sum,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    LogicalAnd,
    LogicalOr,
    LogicalNot,
    IntegerOutput,
    JsonOutput,
    LowerBound,
    UpperBound,
    Negate,
    Difference,
    Product,
    Divide,
    Modulo,
    Quotient,
    Remainder,
}
impl Function {
    fn is_unary(self) -> bool {
        matches!(
            self,
            Self::LogicalNot
                | Self::Negate
                | Self::IntegerOutput
                | Self::JsonOutput
                | Self::LowerBound
                | Self::UpperBound
        )
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Program {
    format_version: u32,
    #[serde(default)]
    function: Function,
    result: Expr,
}
#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
enum IntegerOperation {
    Negate,
    Difference,
    Product,
    Divide,
    Modulo,
    Quotient,
    Remainder,
}
#[derive(Deserialize, Serialize)]
#[serde(tag = "op", rename_all = "camelCase", deny_unknown_fields)]
enum Expr {
    #[serde(rename(serialize = "inputValue", deserialize = "input"))]
    Input {
        input: Side,
    },
    If {
        condition: Condition,
        then: Box<Expr>,
        #[serde(rename = "else")]
        otherwise: Box<Expr>,
    },
    LetBounds {
        lower: End,
        upper: End,
        result: Box<Expr>,
    },
    Bounds {
        lower: End,
        upper: End,
    },
    IntegerSum {},
    IntegerPrimitive {
        operation: IntegerOperation,
    },
    TextConcat {},
    #[serde(rename(serialize = "integerValue", deserialize = "integer"))]
    Integer {
        value: i64,
    },
    Boolean {
        value: bool,
    },
    Fail {
        reason: Failure,
    },
}
#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Failure {
    BoundThreshold,
    Integers,
    Bounds,
    Booleans,
    Types,
    Strings,
    Arithmetic,
    DivisionByZero,
    Comparison,
    Ordering,
    Logical,
    IncompleteInt,
    IncompleteValue,
    IncompleteBool,
    IncompleteString,
    IncompleteBounds,
    IntegerOutput,
}
#[derive(Deserialize, Serialize)]
#[serde(tag = "op", rename_all = "camelCase", deny_unknown_fields)]
enum End {
    #[serde(rename(serialize = "inputEndpoint", deserialize = "input"))]
    Input {
        input: Side,
        endpoint: Endpoint,
    },
    #[serde(rename(serialize = "integerEndpoint", deserialize = "integer"))]
    Integer {
        value: i64,
    },
    Local {
        endpoint: Endpoint,
    },
    Empty {},
    Choose {
        condition: Box<Condition>,
        then: Box<End>,
        #[serde(rename = "else")]
        otherwise: Box<End>,
    },
}
#[derive(Deserialize, Serialize)]
#[serde(tag = "op", rename_all = "camelCase", deny_unknown_fields)]
enum Condition {
    Same {},
    TextCompare {
        relation: Relation,
    },
    Or {
        left: Box<Condition>,
        right: Box<Condition>,
    },
    KindIs {
        input: Side,
        kind: Kind,
    },
    BooleanIs {
        input: Side,
        value: bool,
    },
    HasEndpoint {
        value: End,
    },
    Compare {
        left: End,
        right: End,
        relation: Relation,
    },
    And {
        left: Box<Condition>,
        right: Box<Condition>,
    },
}
fn limits(v: &serde_json::Value, depth: usize, count: &mut usize) -> Result<(), Error> {
    *count += 1;
    if depth > 64 || *count > 512 {
        return ProgramSnafu {
            reason: "exceeds 64 JSON levels or 512 JSON nodes",
        }
        .fail();
    }
    match v {
        serde_json::Value::Array(a) => {
            for x in a {
                limits(x, depth + 1, count)?;
            }
        }
        serde_json::Value::Object(o) => {
            for x in o.values() {
                limits(x, depth + 1, count)?;
            }
        }
        _ => (),
    }
    Ok(())
}
fn parse(input: &str) -> Result<Program, Error> {
    let value = serde_json::from_str(input).context(JsonSnafu)?;
    limits(&value, 1, &mut 0)?;
    // Decode the original text: Value alone would discard duplicate fields.
    let p: Program = serde_json::from_str(input).context(JsonSnafu)?;
    if p.format_version != 1 {
        return ProgramSnafu {
            reason: "formatVersion 1 required",
        }
        .fail();
    }
    p.result.validate(false)?;
    if p.function.is_unary() {
        p.result.validate_unary()?;
    }
    Ok(p)
}
impl Expr {
    fn validate_unary(&self) -> Result<(), Error> {
        match self {
            Self::Input { input: Side::Left }
            | Self::Integer { .. }
            | Self::Boolean { .. }
            | Self::IntegerPrimitive {
                operation: IntegerOperation::Negate,
            }
            | Self::Fail {
                reason:
                    Failure::Logical
                    | Failure::BoundThreshold
                    | Failure::Arithmetic
                    | Failure::IncompleteInt
                    | Failure::IncompleteValue
                    | Failure::IncompleteBool
                    | Failure::IncompleteString
                    | Failure::IncompleteBounds
                    | Failure::IntegerOutput,
            } => Ok(()),
            Self::If {
                condition,
                then,
                otherwise,
            } => {
                condition.validate_unary()?;
                then.validate_unary()?;
                otherwise.validate_unary()
            }
            Self::Bounds { lower, upper } => {
                lower.validate_unary()?;
                upper.validate_unary()
            }
            _ => ProgramSnafu {
                reason: "unary control requires unary expressions",
            }
            .fail(),
        }
    }

    fn validate(&self, bound: bool) -> Result<(), Error> {
        match self {
            Self::Input { .. }
            | Self::Fail { .. }
            | Self::IntegerSum {}
            | Self::IntegerPrimitive { .. }
            | Self::TextConcat {}
            | Self::Integer { .. }
            | Self::Boolean { .. } => Ok(()),
            Self::If {
                condition,
                then,
                otherwise,
            } => {
                condition.validate(bound)?;
                then.validate(bound)?;
                otherwise.validate(bound)
            }
            Self::LetBounds {
                lower,
                upper,
                result,
            } => {
                lower.validate(bound)?;
                upper.validate(bound)?;
                result.validate(true)
            }
            Self::Bounds { lower, upper } => {
                lower.validate(bound)?;
                upper.validate(bound)
            }
        }
    }
}
impl End {
    fn validate_unary(&self) -> Result<(), Error> {
        match self {
            Self::Input {
                input: Side::Left, ..
            }
            | Self::Integer { .. }
            | Self::Empty {} => Ok(()),
            Self::Choose {
                condition,
                then,
                otherwise,
            } => {
                condition.validate_unary()?;
                then.validate_unary()?;
                otherwise.validate_unary()
            }
            _ => ProgramSnafu {
                reason: "unary control requires unary endpoints",
            }
            .fail(),
        }
    }

    fn validate(&self, bound: bool) -> Result<(), Error> {
        match self {
            Self::Local { .. } if !bound => ProgramSnafu {
                reason: "local endpoint outside letBounds",
            }
            .fail(),
            Self::Choose {
                condition,
                then,
                otherwise,
            } => {
                condition.validate(bound)?;
                then.validate(bound)?;
                otherwise.validate(bound)
            }
            _ => Ok(()),
        }
    }
}
impl Condition {
    fn validate_unary(&self) -> Result<(), Error> {
        match self {
            Self::KindIs {
                input: Side::Left, ..
            }
            | Self::BooleanIs {
                input: Side::Left, ..
            } => Ok(()),
            Self::And { left, right } | Self::Or { left, right } => {
                left.validate_unary()?;
                right.validate_unary()
            }
            _ => ProgramSnafu {
                reason: "unary control requires unary conditions",
            }
            .fail(),
        }
    }

    fn validate(&self, bound: bool) -> Result<(), Error> {
        match self {
            Self::KindIs { .. }
            | Self::BooleanIs { .. }
            | Self::Same {}
            | Self::TextCompare { .. } => Ok(()),
            Self::HasEndpoint { value } => value.validate(bound),
            Self::Compare { left, right, .. } => {
                left.validate(bound)?;
                right.validate(bound)
            }
            Self::And { left, right } | Self::Or { left, right } => {
                left.validate(bound)?;
                right.validate(bound)
            }
        }
    }
}
// These primitives expose representation and errors, never intersection policy.
pub fn endpoint(value: &Value, end: Endpoint) -> Option<i64> {
    match value {
        Value::Integer(n) => Some(*n),
        Value::Null
        | Value::Top
        | Value::Int
        | Value::Bool
        | Value::Boolean(_)
        | Value::Text(_)
        | Value::String => None,
        Value::Bounds { lower, upper } => match end {
            Endpoint::Lower => *lower,
            Endpoint::Upper => *upper,
        },
    }
}
pub fn kind_is(value: &Value, kind: Kind) -> bool {
    matches!(
        (value, kind),
        (Value::Null, Kind::Null)
            | (Value::Top, Kind::Top)
            | (Value::Integer(_), Kind::Integer)
            | (Value::Int, Kind::Int)
            | (Value::Bounds { .. }, Kind::Bounds)
            | (Value::Boolean(_), Kind::Boolean)
            | (Value::Bool, Kind::Bool)
            | (Value::Text(_), Kind::Text)
            | (Value::String, Kind::String)
    )
}
/// Compare a concrete boolean representation against a literal.
pub fn boolean_is(value: &Value, expected: bool) -> Result<bool, crate::Error> {
    let Value::Boolean(value) = value else {
        return Err(crate::Error::Logical);
    };
    Ok(*value == expected)
}
pub fn compare(
    left: Option<i64>,
    right: Option<i64>,
    relation: Relation,
) -> Result<bool, crate::Error> {
    let (Some(left), Some(right)) = (left, right) else {
        return Err(crate::Error::UnificationState {
            reason: "comparison requires endpoints",
        });
    };
    Ok(match relation {
        Relation::Equal => left == right,
        Relation::Less => left < right,
        Relation::LessEqual => left <= right,
        Relation::Greater => left > right,
        Relation::GreaterEqual => left >= right,
    })
}

/// Compare concrete text bytes without normalization or allocation.
pub fn text_compare(left: &Value, right: &Value, relation: Relation) -> Result<bool, crate::Error> {
    let (Value::Text(left), Value::Text(right)) = (left, right) else {
        return Err(crate::Error::Ordering);
    };
    Ok(match relation {
        Relation::Equal => left == right,
        Relation::Less => left < right,
        Relation::LessEqual => left <= right,
        Relation::Greater => left > right,
        Relation::GreaterEqual => left >= right,
    })
}
pub fn fail(reason: Failure, left: &Value, right: &Value) -> Result<Value, crate::Error> {
    match reason {
        Failure::BoundThreshold => Err(crate::Error::BoundThreshold),
        Failure::Bounds => Err(crate::Error::BoundsConflict),
        Failure::Types => Err(crate::Error::TypeConflict),
        Failure::Arithmetic => Err(crate::Error::Arithmetic),
        Failure::DivisionByZero => Err(crate::Error::DivisionByZero),
        Failure::Comparison => Err(crate::Error::Comparison),
        Failure::Ordering => Err(crate::Error::Ordering),
        Failure::Logical => Err(crate::Error::Logical),
        Failure::IncompleteInt => Err(crate::Error::Incomplete),
        Failure::IncompleteValue => Err(crate::Error::IncompleteValue),
        Failure::IncompleteBool => Err(crate::Error::IncompleteBool),
        Failure::IncompleteString => Err(crate::Error::IncompleteString),
        Failure::IncompleteBounds => Err(crate::Error::IncompleteBounds),
        Failure::IntegerOutput => Err(crate::Error::IntegerOutput),
        Failure::Booleans => match (left, right) {
            (Value::Boolean(left), Value::Boolean(right)) => Err(crate::Error::BooleanConflict {
                left: *left,
                right: *right,
            }),
            _ => Err(crate::Error::UnificationState {
                reason: "boolean conflict requires booleans",
            }),
        },
        Failure::Strings => match (left, right) {
            (Value::Text(left), Value::Text(right)) => Err(crate::Error::StringConflict {
                left: left.as_ref().to_owned(),
                right: right.as_ref().to_owned(),
            }),
            _ => Err(crate::Error::UnificationState {
                reason: "string conflict requires strings",
            }),
        },
        Failure::Integers => match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => Err(crate::Error::Conflict {
                left: *left,
                right: *right,
            }),
            _ => Err(crate::Error::UnificationState {
                reason: "integer conflict requires integers",
            }),
        },
    }
}
impl Side {
    fn value<'a>(self, l: &'a Value, r: &'a Value) -> &'a Value {
        match self {
            Self::Left => l,
            Self::Right => r,
        }
    }
}
impl Endpoint {
    fn value(self, locals: (Option<i64>, Option<i64>)) -> Option<i64> {
        match self {
            Self::Lower => locals.0,
            Self::Upper => locals.1,
        }
    }
}
impl End {
    fn eval_unary(&self, left: &Value) -> Result<Option<i64>, crate::Error> {
        Ok(match self {
            Self::Input {
                input: Side::Left,
                endpoint: end,
            } => endpoint(left, *end),
            Self::Integer { value } => Some(*value),
            Self::Empty {} => None,
            Self::Choose {
                condition,
                then,
                otherwise,
            } => {
                if condition.eval_unary(left)? {
                    then.eval_unary(left)?
                } else {
                    otherwise.eval_unary(left)?
                }
            }
            _ => {
                return Err(crate::Error::UnificationState {
                    reason: "invalid unary endpoint",
                });
            }
        })
    }

    fn eval(
        &self,
        l: &Value,
        r: &Value,
        locals: (Option<i64>, Option<i64>),
    ) -> Result<Option<i64>, crate::Error> {
        Ok(match self {
            Self::Input { input, endpoint: e } => endpoint(input.value(l, r), *e),
            Self::Integer { value } => Some(*value),
            Self::Local { endpoint } => endpoint.value(locals),
            Self::Empty {} => None,
            Self::Choose {
                condition,
                then,
                otherwise,
            } => {
                if condition.eval(l, r, locals)? {
                    then.eval(l, r, locals)?
                } else {
                    otherwise.eval(l, r, locals)?
                }
            }
        })
    }
}
impl Condition {
    fn eval_unary(&self, left: &Value) -> Result<bool, crate::Error> {
        Ok(match self {
            Self::KindIs {
                input: Side::Left,
                kind,
            } => kind_is(left, *kind),
            Self::BooleanIs {
                input: Side::Left,
                value,
            } => boolean_is(left, *value)?,
            Self::And { left: a, right: b } => a.eval_unary(left)? && b.eval_unary(left)?,
            Self::Or { left: a, right: b } => a.eval_unary(left)? || b.eval_unary(left)?,
            _ => {
                return Err(crate::Error::UnificationState {
                    reason: "invalid unary condition",
                });
            }
        })
    }

    fn eval(
        &self,
        l: &Value,
        r: &Value,
        locals: (Option<i64>, Option<i64>),
    ) -> Result<bool, crate::Error> {
        Ok(match self {
            Self::Same {} => l == r,
            Self::TextCompare { relation } => text_compare(l, r, *relation)?,
            Self::Or { left, right } => left.eval(l, r, locals)? || right.eval(l, r, locals)?,
            Self::KindIs { input, kind } => kind_is(input.value(l, r), *kind),
            Self::BooleanIs { input, value } => boolean_is(input.value(l, r), *value)?,
            Self::HasEndpoint { value } => value.eval(l, r, locals)?.is_some(),
            Self::Compare {
                left,
                right,
                relation,
            } => compare(
                left.eval(l, r, locals)?,
                right.eval(l, r, locals)?,
                *relation,
            )?,
            Self::And { left, right } => left.eval(l, r, locals)? && right.eval(l, r, locals)?,
        })
    }
}
impl Expr {
    fn eval_unary(&self, left: &Value) -> Result<Value, crate::Error> {
        match self {
            Self::Input { input: Side::Left } => Ok(left.clone()),
            Self::Bounds { lower, upper } => Ok(Value::Bounds {
                lower: lower.eval_unary(left)?,
                upper: upper.eval_unary(left)?,
            }),
            Self::Fail {
                reason: Failure::BoundThreshold,
            } => Err(crate::Error::BoundThreshold),
            Self::Integer { value } => Ok(Value::Integer(*value)),
            Self::Boolean { value } => Ok(Value::Boolean(*value)),
            Self::Fail {
                reason: Failure::Logical,
            } => Err(crate::Error::Logical),
            Self::Fail {
                reason: Failure::Arithmetic,
            } => Err(crate::Error::Arithmetic),
            Self::Fail {
                reason: Failure::IncompleteInt,
            } => Err(crate::Error::Incomplete),
            Self::Fail {
                reason: Failure::IncompleteValue,
            } => Err(crate::Error::IncompleteValue),
            Self::Fail {
                reason: Failure::IncompleteBool,
            } => Err(crate::Error::IncompleteBool),
            Self::Fail {
                reason: Failure::IncompleteString,
            } => Err(crate::Error::IncompleteString),
            Self::Fail {
                reason: Failure::IncompleteBounds,
            } => Err(crate::Error::IncompleteBounds),
            Self::Fail {
                reason: Failure::IntegerOutput,
            } => Err(crate::Error::IntegerOutput),
            Self::IntegerPrimitive {
                operation: IntegerOperation::Negate,
            } => crate::runtime::integer_negate(left),
            Self::If {
                condition,
                then,
                otherwise,
            } => {
                if condition.eval_unary(left)? {
                    then.eval_unary(left)
                } else {
                    otherwise.eval_unary(left)
                }
            }
            _ => Err(crate::Error::UnificationState {
                reason: "invalid unary expression",
            }),
        }
    }

    fn eval(
        &self,
        l: &Value,
        r: &Value,
        locals: (Option<i64>, Option<i64>),
    ) -> Result<Value, crate::Error> {
        match self {
            Self::Input { input } => Ok(input.value(l, r).clone()),
            Self::If {
                condition,
                then,
                otherwise,
            } => {
                if condition.eval(l, r, locals)? {
                    then.eval(l, r, locals)
                } else {
                    otherwise.eval(l, r, locals)
                }
            }
            Self::LetBounds {
                lower,
                upper,
                result,
            } => result.eval(l, r, (lower.eval(l, r, locals)?, upper.eval(l, r, locals)?)),
            Self::Bounds { lower, upper } => Ok(Value::Bounds {
                lower: lower.eval(l, r, locals)?,
                upper: upper.eval(l, r, locals)?,
            }),
            Self::IntegerSum {} => crate::runtime::integer_sum(l, r),
            Self::IntegerPrimitive { operation } => match operation {
                IntegerOperation::Negate => crate::runtime::integer_negate(l),
                IntegerOperation::Difference => crate::runtime::integer_difference(l, r),
                IntegerOperation::Product => crate::runtime::integer_product(l, r),
                IntegerOperation::Divide => crate::runtime::integer_divide(l, r),
                IntegerOperation::Modulo => crate::runtime::integer_modulo(l, r),
                IntegerOperation::Quotient => crate::runtime::integer_quotient(l, r),
                IntegerOperation::Remainder => crate::runtime::integer_remainder(l, r),
            },
            Self::TextConcat {} => crate::runtime::text_concat(l, r),
            Self::Integer { value } => Ok(Value::Integer(*value)),
            Self::Boolean { value } => Ok(Value::Boolean(*value)),
            Self::Fail { reason } => fail(*reason, l, r),
        }
    }
}
/// Lower validated control through the shared CUE compiler policy.
pub fn generate(input: &str) -> Result<String, Error> {
    #[cfg(feature = "compiled-lowering")]
    {
        lower(parse(input)?, &crate::compiled_unify_lowering::Compiled)
    }
    #[cfg(not(feature = "compiled-lowering"))]
    {
        generate_with_policy(
            input,
            include_str!(concat!(
                env!("CUESTACEAN_BOOTSTRAP_JSON"),
                "/unify-lowering.json"
            )),
        )
    }
}

pub fn generate_with_policy(input: &str, policy: &str) -> Result<String, Error> {
    let p = parse(input)?;
    let policy =
        crate::lowering::Policy::parse(policy).map_err(|source| Error::Lowering { source })?;
    lower(p, &policy)
}

// Only validated unary result trees reach this serialization-only projection.
fn project_unary(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(fields) => {
            if fields.get("op").and_then(serde_json::Value::as_str) == Some("fail") {
                fields.insert(
                    "op".to_owned(),
                    serde_json::Value::String("unaryFail".to_owned()),
                );
            }
            for child in fields.values_mut() {
                project_unary(child);
            }
        }
        serde_json::Value::Array(items) => {
            for child in items {
                project_unary(child);
            }
        }
        _ => {}
    }
}

fn lower(p: Program, driver: &dyn crate::lowering::Driver) -> Result<String, Error> {
    let unary = p.function.is_unary();
    let mut result = serde_json::json!(p.result);
    if unary {
        project_unary(&mut result);
    }
    let node =
        serde_json::json!({"op":"program", "function":p.function, "unary":unary, "result":result});
    crate::lowering::Context::run(&node, driver).map_err(|source| Error::Lowering { source })
}
/// Execute the same validated control directly, for differential testing.
pub fn interpret(input: &str, left: Value, right: Value) -> Result<Value, crate::Error> {
    let p = parse(input).map_err(|source| crate::Error::UnificationProgram { source })?;
    if p.function.is_unary() {
        return Err(crate::Error::UnificationProgram {
            source: Error::Program {
                reason: "binary interpreter requires binary control",
            },
        });
    }
    p.result.eval(&left, &right, (None, None))
}

/// Execute validated unary control without fabricating a second operand.
pub fn interpret_unary(input: &str, value: Value) -> Result<Value, crate::Error> {
    let p = parse(input).map_err(|source| crate::Error::UnificationProgram { source })?;
    if !p.function.is_unary() {
        return Err(crate::Error::UnificationProgram {
            source: Error::Program {
                reason: "unary interpreter requires unary control",
            },
        });
    }
    p.result.eval_unary(&value)
}

#[cfg(test)]
mod tests {
    use super::{limits, parse};
    use serde_json::Value;

    #[test]
    fn null_and_top_representation() {
        use super::{Endpoint, Kind, endpoint, kind_is};
        use crate::runtime::Value as Scalar;

        for (value, kind, other) in [
            (Scalar::Null, Kind::Null, Kind::Top),
            (Scalar::Top, Kind::Top, Kind::Null),
        ] {
            assert!(kind_is(&value, kind));
            assert!(!kind_is(&value, other));
            assert_eq!(endpoint(&value, Endpoint::Lower), None);
            assert_eq!(endpoint(&value, Endpoint::Upper), None);
        }
        assert_eq!(
            serde_json::to_string(&Kind::Null).ok().as_deref(),
            Some("\"null\"")
        );
        assert_eq!(
            serde_json::to_string(&Kind::Top).ok().as_deref(),
            Some("\"top\"")
        );
        assert!(serde_json::from_str::<Kind>("\"null\"").is_ok());
        assert!(serde_json::from_str::<Kind>("\"top\"").is_ok());
    }

    #[test]
    fn unary_arity_scope_and_interpretation() {
        use super::{interpret, interpret_unary, project_unary};
        use crate::runtime::Value as Scalar;

        let control = r#"{"formatVersion":1,"function":"logicalNot","result":{"op":"if","condition":{"op":"kindIs","input":"left","kind":"boolean"},"then":{"op":"if","condition":{"op":"booleanIs","input":"left","value":true},"then":{"op":"boolean","value":false},"else":{"op":"boolean","value":true}},"else":{"op":"fail","reason":"logical"}}}"#;
        assert_eq!(
            interpret_unary(control, Scalar::Boolean(true)).ok(),
            Some(Scalar::Boolean(false))
        );
        assert_eq!(
            interpret_unary(control, Scalar::Boolean(false)).ok(),
            Some(Scalar::Boolean(true))
        );
        assert!(matches!(
            interpret_unary(control, Scalar::Bool),
            Err(crate::Error::Logical)
        ));
        assert!(matches!(
            interpret_unary(control, Scalar::Integer(1)),
            Err(crate::Error::Logical)
        ));
        assert!(interpret(control, Scalar::Boolean(true), Scalar::Boolean(false)).is_err());
        let binary = r#"{"formatVersion":1,"result":{"op":"input","input":"left"}}"#;
        assert!(interpret_unary(binary, Scalar::Boolean(true)).is_err());
        for expr in [
            r#"{"op":"input","input":"right"}"#,
            r#"{"op":"integerSum"}"#,
            r#"{"op":"textConcat"}"#,
            r#"{"op":"letBounds","lower":{"op":"empty"},"upper":{"op":"empty"},"result":{"op":"input","input":"left"}}"#,
            r#"{"op":"bounds","lower":{"op":"input","input":"right","endpoint":"lower"},"upper":{"op":"empty"}}"#,
            r#"{"op":"fail","reason":"types"}"#,
            r#"{"op":"unaryFail","reason":"logical"}"#,
        ] {
            let invalid = format!(
                r#"{{"formatVersion":1,"function":"logicalNot","result":{{"op":"if","condition":{{"op":"kindIs","input":"left","kind":"boolean"}},"then":{{"op":"boolean","value":true}},"else":{expr}}}}}"#
            );
            assert!(parse(&invalid).is_err(), "{expr}");
        }
        for condition in [
            r#"{"op":"same"}"#,
            r#"{"op":"kindIs","input":"right","kind":"boolean"}"#,
            r#"{"op":"booleanIs","input":"right","value":true}"#,
            r#"{"op":"hasEndpoint","value":{"op":"empty"}}"#,
            r#"{"op":"textCompare","relation":"equal"}"#,
            r#"{"op":"and","left":{"op":"kindIs","input":"left","kind":"boolean"},"right":{"op":"same"}}"#,
        ] {
            let invalid = format!(
                r#"{{"formatVersion":1,"function":"logicalNot","result":{{"op":"if","condition":{condition},"then":{{"op":"boolean","value":true}},"else":{{"op":"boolean","value":false}}}}}}"#
            );
            assert!(parse(&invalid).is_err(), "{condition}");
        }
        let p = parse(control).expect("valid unary control");
        let mut projected = serde_json::to_value(p.result).expect("serialize result");
        project_unary(&mut projected);
        assert_eq!(projected["else"]["op"], "unaryFail");
        assert_eq!(projected["else"]["reason"], "logical");
    }

    #[test]
    fn unary_endpoint_execution_and_defensive_errors() {
        use super::{Condition, End, Endpoint, Expr, Side};
        use crate::runtime::Value as Scalar;

        let input = Scalar::Bounds {
            lower: Some(i64::MIN),
            upper: Some(i64::MAX),
        };
        let project = |endpoint| End::Input {
            input: Side::Left,
            endpoint,
        };
        let bounds = Expr::Bounds {
            lower: project(Endpoint::Lower),
            upper: project(Endpoint::Upper),
        };
        assert!(bounds.validate_unary().is_ok());
        assert_eq!(bounds.eval_unary(&input).ok(), Some(input.clone()));
        for invalid in [
            End::Input {
                input: Side::Right,
                endpoint: Endpoint::Lower,
            },
            End::Local {
                endpoint: Endpoint::Upper,
            },
        ] {
            assert!(invalid.validate_unary().is_err());
            assert!(matches!(
                invalid.eval_unary(&input),
                Err(crate::Error::UnificationState { .. })
            ));
            let choose = End::Choose {
                condition: Box::new(Condition::KindIs {
                    input: Side::Left,
                    kind: super::Kind::Bounds,
                }),
                then: Box::new(End::Integer { value: 7 }),
                otherwise: Box::new(invalid),
            };
            assert!(choose.validate_unary().is_err());
            // Evaluation is lazy; validation, not evaluation, checks unused arms.
            assert_eq!(choose.eval_unary(&input).ok(), Some(Some(7)));
        }
        let bounds = Expr::Bounds {
            lower: End::Choose {
                condition: Box::new(Condition::BooleanIs {
                    input: Side::Left,
                    value: true,
                }),
                then: Box::new(End::Empty {}),
                otherwise: Box::new(End::Empty {}),
            },
            upper: End::Local {
                endpoint: Endpoint::Upper,
            },
        };
        assert!(matches!(
            bounds.eval_unary(&input),
            Err(crate::Error::Logical)
        ));
    }

    #[test]
    fn output_control_arity_and_failures() {
        use super::{interpret, interpret_unary, project_unary};
        use crate::runtime::Value as Scalar;

        for function in ["integerOutput", "jsonOutput", "lowerBound", "upperBound"] {
            let identity = format!(
                r#"{{"formatVersion":1,"function":"{function}","result":{{"op":"input","input":"left"}}}}"#
            );
            assert_eq!(
                interpret_unary(&identity, Scalar::Null).ok(),
                Some(Scalar::Null)
            );
            assert!(interpret(&identity, Scalar::Null, Scalar::Top).is_err());
            for (reason, expected) in [
                ("incompleteInt", "non-concrete int value"),
                ("incompleteValue", "non-concrete value"),
                ("incompleteBool", "non-concrete bool value"),
                ("incompleteString", "non-concrete string value"),
                ("incompleteBounds", "non-concrete integer bounds"),
                ("integerOutput", "integer output requires an integer value"),
                (
                    "boundThreshold",
                    "bound threshold requires a concrete integer",
                ),
            ] {
                let result = format!(r#"{{"op":"fail","reason":"{reason}"}}"#);
                let unary =
                    format!(r#"{{"formatVersion":1,"function":"{function}","result":{result}}}"#);
                assert_eq!(
                    interpret_unary(&unary, Scalar::Null)
                        .expect_err("unary failure")
                        .to_string(),
                    expected
                );
                let p = parse(&unary).expect("valid unary failure");
                let mut projected = serde_json::json!(p.result);
                project_unary(&mut projected);
                assert_eq!(projected["op"], "unaryFail");
                assert_eq!(projected["reason"], reason);
                let binary = format!(r#"{{"formatVersion":1,"result":{result}}}"#);
                assert_eq!(
                    interpret(&binary, Scalar::Null, Scalar::Top)
                        .expect_err("binary failure")
                        .to_string(),
                    expected
                );
            }
        }
    }

    #[test]
    fn integer_primitive_validation_and_execution() {
        use super::{interpret, interpret_unary, project_unary};
        use crate::runtime::Value as Scalar;

        let unary = r#"{"formatVersion":1,"function":"negate","result":{"op":"integerPrimitive","operation":"negate"}}"#;
        assert_eq!(
            interpret_unary(unary, Scalar::Integer(7)).ok(),
            Some(Scalar::Integer(-7))
        );
        assert!(interpret(unary, Scalar::Integer(7), Scalar::Integer(1)).is_err());
        let failure = r#"{"formatVersion":1,"function":"negate","result":{"op":"fail","reason":"arithmetic"}}"#;
        assert!(matches!(
            interpret_unary(failure, Scalar::Int),
            Err(crate::Error::Arithmetic)
        ));
        if let Ok(p) = parse(failure) {
            let mut projected = serde_json::json!(p.result);
            project_unary(&mut projected);
            assert_eq!(projected["op"], "unaryFail");
            assert_eq!(projected["reason"], "arithmetic");
        } else {
            panic!("valid unary failure");
        }
        for (operation, expected) in [("negate", -7), ("difference", 1), ("product", 42)] {
            let control = format!(
                r#"{{"formatVersion":1,"function":"difference","result":{{"op":"integerPrimitive","operation":"{operation}"}}}}"#
            );
            assert_eq!(
                interpret(&control, Scalar::Integer(7), Scalar::Integer(6)).ok(),
                Some(Scalar::Integer(expected))
            );
            assert!(interpret_unary(&control, Scalar::Integer(7)).is_err());
        }
        for expr in [
            r#"{"op":"integerPrimitive","operation":"difference"}"#,
            r#"{"op":"integerPrimitive","operation":"product"}"#,
            r#"{"op":"integerPrimitive","operation":"unknown"}"#,
            r#"{"op":"integerPrimitive","operation":1}"#,
            r#"{"op":"integerPrimitive"}"#,
            r#"{"op":"integerPrimitive","operation":"negate","extra":1}"#,
            r#"{"op":"integerPrimitive","operation":"negate","operation":"negate"}"#,
        ] {
            let control = format!(
                r#"{{"formatVersion":1,"function":"negate","result":{{"op":"if","condition":{{"op":"kindIs","input":"left","kind":"integer"}},"then":{{"op":"input","input":"left"}},"else":{expr}}}}}"#
            );
            assert!(parse(&control).is_err(), "{expr}");
        }
    }

    #[test]
    fn division_primitives_and_integer_literal_roles() {
        use super::{interpret, interpret_unary};
        use crate::runtime::Value as Scalar;

        for (operation, expected) in [
            ("divide", -3),
            ("modulo", 2),
            ("quotient", -2),
            ("remainder", -1),
        ] {
            let control = format!(
                r#"{{"formatVersion":1,"function":"{operation}","result":{{"op":"integerPrimitive","operation":"{operation}"}}}}"#
            );
            assert_eq!(
                interpret(&control, Scalar::Integer(-7), Scalar::Integer(3)).ok(),
                Some(Scalar::Integer(expected))
            );
            assert!(interpret_unary(&control, Scalar::Integer(7)).is_err());
            let unary = control.replacen(
                &format!(r#""function":"{operation}""#),
                r#""function":"negate""#,
                1,
            );
            assert!(parse(&unary).is_err());
        }
        for value in [i64::MIN, i64::MAX] {
            let literal = format!(r#"{{"op":"integer","value":{value}}}"#);
            let control = format!(r#"{{"formatVersion":1,"result":{literal}}}"#);
            assert_eq!(
                interpret(&control, Scalar::Null, Scalar::Top).ok(),
                Some(Scalar::Integer(value))
            );
            let p = parse(&control).expect("valid integer value literal");
            assert_eq!(serde_json::json!(p.result)["op"], "integerValue");
            let unary = control.replace(
                r#""formatVersion":1"#,
                r#""formatVersion":1,"function":"negate""#,
            );
            assert_eq!(
                interpret_unary(&unary, Scalar::Null).ok(),
                Some(Scalar::Integer(value))
            );
            assert!(parse(&control.replace("integer", "integerValue")).is_err());
            let bounds = format!(
                r#"{{"formatVersion":1,"result":{{"op":"bounds","lower":{literal},"upper":{literal}}}}}"#
            );
            let p = parse(&bounds).expect("valid integer endpoint literal");
            assert_eq!(
                serde_json::json!(p.result)["lower"]["op"],
                "integerEndpoint"
            );
            assert_eq!(
                interpret(&bounds, Scalar::Null, Scalar::Top).ok(),
                Some(Scalar::Bounds {
                    lower: Some(value),
                    upper: Some(value)
                })
            );
            assert!(parse(&bounds.replace("integer", "integerEndpoint")).is_err());
        }
        for value in ["9223372036854775808", "-9223372036854775809", "1.0", "null"] {
            let control =
                format!(r#"{{"formatVersion":1,"result":{{"op":"integer","value":{value}}}}}"#);
            assert!(parse(&control).is_err());
        }
        let failure = r#"{"formatVersion":1,"result":{"op":"fail","reason":"divisionByZero"}}"#;
        assert!(matches!(
            interpret(failure, Scalar::Null, Scalar::Top),
            Err(crate::Error::DivisionByZero)
        ));
        assert!(
            parse(&failure.replace(
                r#""formatVersion":1"#,
                r#""formatVersion":1,"function":"negate""#
            ))
            .is_err()
        );
    }

    #[test]
    fn function_and_new_expression_validation() {
        let default = r#"{"formatVersion":1,"result":{"op":"integerSum"}}"#;
        let sum = r#"{"formatVersion":1,"function":"sum","result":{"op":"textConcat"}}"#;
        assert!(parse(default).is_ok());
        assert!(parse(sum).is_ok());
        for function in ["null", "\"other\"", "123"] {
            let control = format!(
                r#"{{"formatVersion":1,"function":{function},"result":{{"op":"integerSum"}}}}"#
            );
            assert!(parse(&control).is_err());
        }
        assert!(parse(r#"{"formatVersion":1,"result":{"op":"textConcat","extra":true}}"#).is_err());
        for function in [
            "equal",
            "notEqual",
            "less",
            "lessEqual",
            "greater",
            "greaterEqual",
            "logicalAnd",
            "logicalOr",
        ] {
            let control = format!(
                r#"{{"formatVersion":1,"function":"{function}","result":{{"op":"boolean","value":false}}}}"#
            );
            assert!(parse(&control).is_ok());
        }
        for value in ["null", "1", "\"false\""] {
            let control =
                format!(r#"{{"formatVersion":1,"result":{{"op":"boolean","value":{value}}}}}"#);
            assert!(parse(&control).is_err());
        }
        assert!(
            parse(r#"{"formatVersion":1,"result":{"op":"boolean","value":true,"extra":1}}"#)
                .is_err()
        );
        assert!(
            parse(r#"{"formatVersion":1,"result":{"op":"fail","reason":"comparison"}}"#).is_ok()
        );
    }

    #[test]
    fn ordering_primitives_and_control_validation() {
        use super::{Relation, compare, text_compare};
        use crate::runtime::Value as Scalar;
        use std::sync::Arc;

        assert_eq!(
            compare(Some(i64::MIN), Some(i64::MAX), Relation::LessEqual).ok(),
            Some(true)
        );
        assert_eq!(
            compare(Some(i64::MAX), Some(i64::MAX), Relation::GreaterEqual).ok(),
            Some(true)
        );
        assert_eq!(
            compare(Some(i64::MIN), Some(i64::MAX), Relation::GreaterEqual).ok(),
            Some(false)
        );
        assert!(matches!(
            compare(None, Some(0), Relation::LessEqual),
            Err(crate::Error::UnificationState {
                reason: "comparison requires endpoints"
            })
        ));
        assert!(matches!(
            compare(Some(0), None, Relation::GreaterEqual),
            Err(crate::Error::UnificationState {
                reason: "comparison requires endpoints"
            })
        ));
        let text = |s: &str| Scalar::Text(Arc::from(s));
        assert_eq!(
            text_compare(&text("é"), &text("e\u{301}"), Relation::Greater).ok(),
            Some(true)
        );
        assert_eq!(
            text_compare(&text("10"), &text("2"), Relation::Less).ok(),
            Some(true)
        );
        assert_eq!(
            text_compare(&text(""), &text(""), Relation::LessEqual).ok(),
            Some(true)
        );
        assert_eq!(
            text_compare(&text(""), &text(""), Relation::GreaterEqual).ok(),
            Some(true)
        );
        assert!(matches!(
            text_compare(&text(""), &Scalar::String, Relation::Equal),
            Err(crate::Error::Ordering)
        ));
        for relation in ["equal", "less", "lessEqual", "greater", "greaterEqual"] {
            let control = format!(
                r#"{{"formatVersion":1,"result":{{"op":"if","condition":{{"op":"textCompare","relation":"{relation}"}},"then":{{"op":"boolean","value":true}},"else":{{"op":"fail","reason":"ordering"}}}}}}"#
            );
            assert!(parse(&control).is_ok());
        }
        assert!(parse(r#"{"formatVersion":1,"result":{"op":"if","condition":{"op":"textCompare","relation":"less","extra":1},"then":{"op":"boolean","value":true},"else":{"op":"boolean","value":false}}}"#).is_err());
    }

    #[test]
    fn boolean_predicate_representation_and_validation() {
        use super::boolean_is;
        use crate::runtime::Value as Scalar;

        assert_eq!(boolean_is(&Scalar::Boolean(true), true).ok(), Some(true));
        assert_eq!(boolean_is(&Scalar::Boolean(false), true).ok(), Some(false));
        for value in [Scalar::Bool, Scalar::Int, Scalar::Integer(1)] {
            assert!(matches!(
                boolean_is(&value, false),
                Err(crate::Error::Logical)
            ));
        }
        let control = |condition: &str| {
            format!(
                r#"{{"formatVersion":1,"result":{{"op":"if","condition":{condition},"then":{{"op":"boolean","value":true}},"else":{{"op":"fail","reason":"logical"}}}}}}"#
            )
        };
        assert!(
            parse(&control(
                r#"{"op":"booleanIs","input":"left","value":false}"#
            ))
            .is_ok()
        );
        for condition in [
            r#"{"op":"booleanIs","input":"left"}"#,
            r#"{"op":"booleanIs","input":"left","value":null}"#,
            r#"{"op":"booleanIs","input":"left","value":1}"#,
            r#"{"op":"booleanIs","input":"other","value":true}"#,
            r#"{"op":"booleanIs","input":"left","value":true,"extra":1}"#,
            r#"{"op":"booleanIs","input":"left","value":true,"value":false}"#,
        ] {
            assert!(parse(&control(condition)).is_err());
        }
        let unused_invalid = r#"{"formatVersion":1,"result":{"op":"if","condition":{"op":"same"},"then":{"op":"boolean","value":true},"else":{"op":"if","condition":{"op":"booleanIs","input":"left"},"then":{"op":"boolean","value":true},"else":{"op":"boolean","value":false}}}}"#;
        assert!(parse(unused_invalid).is_err());
    }

    #[test]
    fn structural_boundaries_count_the_root() {
        let mut value = Value::Null;
        for _ in 1..64 {
            value = Value::Array(vec![value]);
        }
        assert!(limits(&value, 1, &mut 0).is_ok());
        value = Value::Array(vec![value]);
        assert!(limits(&value, 1, &mut 0).is_err());
        let value = Value::Array(vec![Value::Null; 511]);
        assert!(limits(&value, 1, &mut 0).is_ok());
        let value = Value::Array(vec![Value::Null; 512]);
        assert!(limits(&value, 1, &mut 0).is_err());
    }
}
