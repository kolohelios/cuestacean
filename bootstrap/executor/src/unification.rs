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
    Third,
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
    List,
    Record,
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
    StringLength,
    ListLength,
    Index,
    ListIndex,
    ListSlice,
    RecordIndex,
    RecordLength,
    Length,
    IntegerOutput,
    JsonOutput,
    LowerBound,
    UpperBound,
    LowerExclusiveBound,
    UpperExclusiveBound,
    Negate,
    Difference,
    Product,
    Divide,
    Modulo,
    Quotient,
    Remainder,
}
impl Function {
    fn arity(self) -> usize {
        if self.is_unary() {
            1
        } else if matches!(self, Self::ListSlice) {
            3
        } else {
            2
        }
    }

    fn is_unary(self) -> bool {
        matches!(
            self,
            Self::LogicalNot
                | Self::StringLength
                | Self::ListLength
                | Self::RecordLength
                | Self::Length
                | Self::Negate
                | Self::IntegerOutput
                | Self::JsonOutput
                | Self::LowerBound
                | Self::UpperBound
                | Self::LowerExclusiveBound
                | Self::UpperExclusiveBound
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
    TextLength {},
    ListCount {},
    ListElement {},
    ListSlice {},
    RecordGet {},
    RecordCount {},
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
    ListSliceOperands,
    ListSliceRange,
    CollectionIndexOperands,
    IndexOperands,
    IndexRange,
    RecordIndexOperands,
    RecordMissing,
    RecordLength,
    Records,
    LengthOperand,
    Length,
    ListLength,
    Lists,
    BoundThreshold,
    EmptyBound,
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
    #[serde(rename(serialize = "listCountEndpoint", deserialize = "listCount"))]
    ListCount {},
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
    Offset {
        value: Box<End>,
        delta: i64,
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
    RecordHas {},
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
    p.result.validate_arity(p.function.arity())?;
    if p.function.is_unary() {
        p.result.validate_unary()?;
    }
    Ok(p)
}
impl Expr {
    fn validate_arity(&self, arity: usize) -> Result<(), Error> {
        match self {
            Self::Input { input } => input.validate_arity(arity),
            Self::ListSlice {} if arity != 3 => ProgramSnafu {
                reason: "listSlice requires ternary control",
            }
            .fail(),
            Self::If {
                condition,
                then,
                otherwise,
            } => {
                condition.validate_arity(arity)?;
                then.validate_arity(arity)?;
                otherwise.validate_arity(arity)
            }
            Self::LetBounds {
                lower,
                upper,
                result,
            } => {
                lower.validate_arity(arity)?;
                upper.validate_arity(arity)?;
                result.validate_arity(arity)
            }
            Self::Bounds { lower, upper } => {
                lower.validate_arity(arity)?;
                upper.validate_arity(arity)
            }
            _ => Ok(()),
        }
    }

    fn validate_unary(&self) -> Result<(), Error> {
        match self {
            Self::Input { input: Side::Left }
            | Self::Integer { .. }
            | Self::Boolean { .. }
            | Self::TextLength {}
            | Self::ListCount {}
            | Self::RecordCount {}
            | Self::IntegerPrimitive {
                operation: IntegerOperation::Negate,
            }
            | Self::Fail {
                reason:
                    Failure::Logical
                    | Failure::Length
                    | Failure::ListLength
                    | Failure::RecordLength
                    | Failure::LengthOperand
                    | Failure::BoundThreshold
                    | Failure::EmptyBound
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
            | Self::ListElement {}
            | Self::ListSlice {}
            | Self::RecordGet {}
            | Self::RecordCount {}
            | Self::TextLength {}
            | Self::ListCount {}
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
    fn validate_arity(&self, arity: usize) -> Result<(), Error> {
        match self {
            Self::Input { input, .. } => input.validate_arity(arity),
            Self::Offset { value, .. } => value.validate_arity(arity),
            Self::Choose {
                condition,
                then,
                otherwise,
            } => {
                condition.validate_arity(arity)?;
                then.validate_arity(arity)?;
                otherwise.validate_arity(arity)
            }
            _ => Ok(()),
        }
    }

    fn validate_unary(&self) -> Result<(), Error> {
        match self {
            Self::Input {
                input: Side::Left, ..
            }
            | Self::Integer { .. }
            | Self::Empty {} => Ok(()),
            Self::Offset { value, delta } => {
                validate_delta(*delta)?;
                value.validate_unary()
            }
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
            Self::Offset { value, delta } => {
                validate_delta(*delta)?;
                value.validate(bound)
            }
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
    fn validate_arity(&self, arity: usize) -> Result<(), Error> {
        match self {
            Self::KindIs { input, .. } | Self::BooleanIs { input, .. } => {
                input.validate_arity(arity)
            }
            Self::HasEndpoint { value } => value.validate_arity(arity),
            Self::Compare { left, right, .. } => {
                left.validate_arity(arity)?;
                right.validate_arity(arity)
            }
            Self::And { left, right } | Self::Or { left, right } => {
                left.validate_arity(arity)?;
                right.validate_arity(arity)
            }
            _ => Ok(()),
        }
    }

    fn validate_unary(&self) -> Result<(), Error> {
        match self {
            Self::KindIs {
                input: Side::Left, ..
            }
            | Self::BooleanIs {
                input: Side::Left, ..
            } => Ok(()),
            Self::Compare { left, right, .. } => {
                left.validate_unary()?;
                right.validate_unary()
            }
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
            | Self::RecordHas {}
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
fn validate_delta(delta: i64) -> Result<(), Error> {
    if matches!(delta, -1 | 1) {
        Ok(())
    } else {
        ProgramSnafu {
            reason: "endpoint offset delta must be -1 or 1",
        }
        .fail()
    }
}
// These primitives expose representation and errors, never intersection policy.
/// Checked endpoint addition; missing endpoints remain infinite.
/// The control DSL separately restricts offsets to unit deltas.
pub fn offset_endpoint(value: Option<i64>, delta: i64) -> Result<Option<i64>, crate::Error> {
    value
        .map(|value| {
            value
                .checked_add(delta)
                .ok_or(crate::Error::EndpointOffsetOverflow)
        })
        .transpose()
}
pub fn endpoint(value: &Value, end: Endpoint) -> Option<i64> {
    match value {
        Value::Integer(n) => Some(*n),
        Value::Null
        | Value::Top
        | Value::Int
        | Value::Bool
        | Value::Boolean(_)
        | Value::Text(_)
        | Value::List(_)
        | Value::Record(_)
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
            | (Value::List(_), Kind::List)
            | (Value::Record(_), Kind::Record)
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
        Failure::ListSliceOperands => Err(crate::Error::ListSliceOperands),
        Failure::ListSliceRange => Err(crate::Error::ListSliceRange),
        Failure::CollectionIndexOperands => Err(crate::Error::CollectionIndexOperands),
        Failure::IndexOperands => Err(crate::Error::IndexOperands),
        Failure::IndexRange => Err(crate::Error::IndexRange),
        Failure::RecordIndexOperands => Err(crate::Error::RecordIndexOperands),
        Failure::RecordMissing => {
            let (Value::Record(_), Value::Text(field)) = (left, right) else {
                return Err(crate::Error::RecordIndexOperands);
            };
            Err(crate::Error::RecordMissing {
                field: field.to_string(),
            })
        }
        Failure::RecordLength => Err(crate::Error::RecordLength),
        Failure::Records => Err(crate::Error::RecordConflict),
        Failure::LengthOperand => Err(crate::Error::LengthOperand),
        Failure::Length => Err(crate::Error::Length),
        Failure::ListLength => Err(crate::Error::ListLength),
        Failure::Lists => Err(crate::Error::ListConflict),
        Failure::BoundThreshold => Err(crate::Error::BoundThreshold),
        Failure::EmptyBound => Err(crate::Error::EmptyBound),
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
    fn validate_arity(self, arity: usize) -> Result<(), Error> {
        if matches!(self, Self::Third) && arity != 3 {
            return ProgramSnafu {
                reason: "third input requires ternary control",
            }
            .fail();
        }
        Ok(())
    }

    fn value<'a>(
        self,
        l: &'a Value,
        r: &'a Value,
        third: Option<&'a Value>,
    ) -> Result<&'a Value, crate::Error> {
        match self {
            Self::Left => Ok(l),
            Self::Right => Ok(r),
            Self::Third => third.ok_or(crate::Error::UnificationState {
                reason: "third input requires ternary control",
            }),
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
            Self::Offset { value, delta } => offset_endpoint(value.eval_unary(left)?, *delta)?,
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
        third: Option<&Value>,
        locals: (Option<i64>, Option<i64>),
    ) -> Result<Option<i64>, crate::Error> {
        Ok(match self {
            Self::ListCount {} => endpoint(&crate::runtime::list_count(l)?, Endpoint::Lower),
            Self::Input { input, endpoint: e } => endpoint(input.value(l, r, third)?, *e),
            Self::Integer { value } => Some(*value),
            Self::Local { endpoint } => endpoint.value(locals),
            Self::Empty {} => None,
            Self::Offset { value, delta } => {
                offset_endpoint(value.eval(l, r, third, locals)?, *delta)?
            }
            Self::Choose {
                condition,
                then,
                otherwise,
            } => {
                if condition.eval(l, r, third, locals)? {
                    then.eval(l, r, third, locals)?
                } else {
                    otherwise.eval(l, r, third, locals)?
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
            Self::Compare {
                left: a,
                right: b,
                relation,
            } => compare(a.eval_unary(left)?, b.eval_unary(left)?, *relation)?,
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
        third: Option<&Value>,
        locals: (Option<i64>, Option<i64>),
    ) -> Result<bool, crate::Error> {
        Ok(match self {
            Self::Same {} => l == r,
            Self::RecordHas {} => crate::runtime::record_has(l, r)?,
            Self::TextCompare { relation } => text_compare(l, r, *relation)?,
            Self::Or { left, right } => {
                left.eval(l, r, third, locals)? || right.eval(l, r, third, locals)?
            }
            Self::KindIs { input, kind } => kind_is(input.value(l, r, third)?, *kind),
            Self::BooleanIs { input, value } => boolean_is(input.value(l, r, third)?, *value)?,
            Self::HasEndpoint { value } => value.eval(l, r, third, locals)?.is_some(),
            Self::Compare {
                left,
                right,
                relation,
            } => compare(
                left.eval(l, r, third, locals)?,
                right.eval(l, r, third, locals)?,
                *relation,
            )?,
            Self::And { left, right } => {
                left.eval(l, r, third, locals)? && right.eval(l, r, third, locals)?
            }
        })
    }
}
impl Expr {
    fn eval_unary(&self, left: &Value) -> Result<Value, crate::Error> {
        match self {
            Self::Input { input: Side::Left } => Ok(left.clone()),
            Self::Fail {
                reason: Failure::LengthOperand,
            } => Err(crate::Error::LengthOperand),
            Self::TextLength {} => crate::runtime::text_length(left),
            Self::ListCount {} => crate::runtime::list_count(left),
            Self::RecordCount {} => crate::runtime::record_count(left),
            Self::Fail {
                reason: Failure::RecordLength,
            } => Err(crate::Error::RecordLength),
            Self::Fail {
                reason: Failure::ListLength,
            } => Err(crate::Error::ListLength),
            Self::Fail {
                reason: Failure::Length,
            } => Err(crate::Error::Length),
            Self::Bounds { lower, upper } => Ok(Value::Bounds {
                lower: lower.eval_unary(left)?,
                upper: upper.eval_unary(left)?,
            }),
            Self::Fail {
                reason: Failure::BoundThreshold,
            } => Err(crate::Error::BoundThreshold),
            Self::Fail {
                reason: Failure::EmptyBound,
            } => Err(crate::Error::EmptyBound),
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
        third: Option<&Value>,
        locals: (Option<i64>, Option<i64>),
    ) -> Result<Value, crate::Error> {
        match self {
            Self::Input { input } => Ok(input.value(l, r, third)?.clone()),
            Self::If {
                condition,
                then,
                otherwise,
            } => {
                if condition.eval(l, r, third, locals)? {
                    then.eval(l, r, third, locals)
                } else {
                    otherwise.eval(l, r, third, locals)
                }
            }
            Self::LetBounds {
                lower,
                upper,
                result,
            } => result.eval(
                l,
                r,
                third,
                (
                    lower.eval(l, r, third, locals)?,
                    upper.eval(l, r, third, locals)?,
                ),
            ),
            Self::Bounds { lower, upper } => Ok(Value::Bounds {
                lower: lower.eval(l, r, third, locals)?,
                upper: upper.eval(l, r, third, locals)?,
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
            Self::ListSlice {} => {
                crate::runtime::list_slice_elements(l, r, Side::Third.value(l, r, third)?)
            }
            Self::ListElement {} => crate::runtime::list_at(l, r),
            Self::RecordGet {} => crate::runtime::record_get(l, r),
            Self::RecordCount {} => crate::runtime::record_count(l),
            Self::TextConcat {} => crate::runtime::text_concat(l, r),
            Self::TextLength {} => crate::runtime::text_length(l),
            Self::ListCount {} => crate::runtime::list_count(l),
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
    let node = serde_json::json!({
        "op":"program", "function":p.function, "unary":unary,
        "ternary":p.function.arity() == 3, "result":result
    });
    crate::lowering::Context::run(&node, driver).map_err(|source| Error::Lowering { source })
}
/// Execute the same validated control directly, for differential testing.
pub fn interpret(input: &str, left: Value, right: Value) -> Result<Value, crate::Error> {
    let p = parse(input).map_err(|source| crate::Error::UnificationProgram { source })?;
    if p.function.arity() != 2 {
        return Err(crate::Error::UnificationProgram {
            source: Error::Program {
                reason: "binary interpreter requires binary control",
            },
        });
    }
    p.result.eval(&left, &right, None, (None, None))
}

/// Execute validated ternary control without changing unary or binary arity.
pub fn interpret_ternary(
    input: &str,
    left: Value,
    right: Value,
    third: Value,
) -> Result<Value, crate::Error> {
    let p = parse(input).map_err(|source| crate::Error::UnificationProgram { source })?;
    if p.function.arity() != 3 {
        return Err(crate::Error::UnificationProgram {
            source: Error::Program {
                reason: "ternary interpreter requires ternary control",
            },
        });
    }
    p.result.eval(&left, &right, Some(&third), (None, None))
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

    fn control(function: &str, result: &str) -> String {
        format!(r#"{{"formatVersion":1,"function":"{function}","result":{result}}}"#)
    }

    #[test]
    fn three_interpreter_arity_matrix() {
        use super::{interpret, interpret_ternary, interpret_unary};
        use crate::runtime::Value as Scalar;

        for (function, arity) in [("logicalNot", 1), ("unify", 2), ("listSlice", 3)] {
            let program = control(function, r#"{"op":"input","input":"left"}"#);
            assert_eq!(interpret_unary(&program, Scalar::Null).is_ok(), arity == 1);
            assert_eq!(
                interpret(&program, Scalar::Null, Scalar::Top).is_ok(),
                arity == 2
            );
            assert_eq!(
                interpret_ternary(&program, Scalar::Null, Scalar::Top, Scalar::Int).is_ok(),
                arity == 3
            );
        }
    }

    #[test]
    fn ternary_inputs_conditions_and_slice_kernel() {
        use super::interpret_ternary;
        use crate::runtime::{ClosedList, Value as Scalar};

        let select = control(
            "listSlice",
            r#"{"op":"if","condition":{"op":"and","left":{"op":"kindIs","input":"third","kind":"boolean"},"right":{"op":"booleanIs","input":"third","value":true}},"then":{"op":"input","input":"third"},"else":{"op":"integerSum"}}"#,
        );
        for (third, expected) in [
            (Scalar::Boolean(true), Scalar::Boolean(true)),
            (Scalar::Boolean(false), Scalar::Integer(7)),
            (Scalar::Null, Scalar::Integer(7)),
        ] {
            assert_eq!(
                interpret_ternary(&select, Scalar::Integer(3), Scalar::Integer(4), third).ok(),
                Some(expected)
            );
        }
        let list =
            Scalar::List(ClosedList::new(vec![Scalar::Null, Scalar::Integer(9)]).expect("list"));
        let endpoints = control(
            "listSlice",
            r#"{"op":"if","condition":{"op":"compare","left":{"op":"input","input":"third","endpoint":"upper"},"right":{"op":"listCount"},"relation":"lessEqual"},"then":{"op":"bounds","lower":{"op":"input","input":"right","endpoint":"lower"},"upper":{"op":"input","input":"third","endpoint":"upper"}},"else":{"op":"fail","reason":"listSliceRange"}}"#,
        );
        assert_eq!(
            interpret_ternary(
                &endpoints,
                list.clone(),
                Scalar::Integer(1),
                Scalar::Integer(2)
            )
            .ok(),
            Some(Scalar::Bounds {
                lower: Some(1),
                upper: Some(2)
            })
        );
        assert!(matches!(
            interpret_ternary(
                &endpoints,
                list.clone(),
                Scalar::Integer(1),
                Scalar::Integer(3)
            ),
            Err(crate::Error::ListSliceRange)
        ));
        let slice = control("listSlice", r#"{"op":"listSlice"}"#);
        let expected = Scalar::List(ClosedList::new(vec![Scalar::Integer(9)]).expect("slice"));
        assert_eq!(
            interpret_ternary(&slice, list, Scalar::Integer(1), Scalar::Integer(2)).ok(),
            Some(expected)
        );
        for (reason, expected) in [
            (
                "listSliceOperands",
                "list slicing requires a concrete closed list and integer endpoints",
            ),
            ("listSliceRange", "list slice out of range"),
        ] {
            let program = control(
                "listSlice",
                &format!(r#"{{"op":"fail","reason":"{reason}"}}"#),
            );
            assert_eq!(
                interpret_ternary(&program, Scalar::Null, Scalar::Null, Scalar::Null)
                    .expect_err("failure")
                    .to_string(),
                expected
            );
        }
    }

    #[test]
    fn third_references_reject_in_dead_values_and_conditions_before_backend() {
        for function in ["logicalNot", "unify"] {
            for result in [
                r#"{"op":"if","condition":{"op":"kindIs","input":"left","kind":"null"},"then":{"op":"input","input":"left"},"else":{"op":"input","input":"third"}}"#,
                r#"{"op":"if","condition":{"op":"or","left":{"op":"kindIs","input":"left","kind":"null"},"right":{"op":"kindIs","input":"third","kind":"integer"}},"then":{"op":"input","input":"left"},"else":{"op":"input","input":"left"}}"#,
                r#"{"op":"if","condition":{"op":"and","left":{"op":"kindIs","input":"left","kind":"integer"},"right":{"op":"booleanIs","input":"third","value":true}},"then":{"op":"input","input":"left"},"else":{"op":"input","input":"left"}}"#,
            ] {
                let program = control(function, result);
                assert!(parse(&program).is_err());
                assert!(matches!(
                    super::generate_with_policy(&program, "not JSON"),
                    Err(super::Error::Program { .. })
                ));
                assert!(parse(&control("listSlice", result)).is_ok());
            }
        }
    }

    #[test]
    fn third_endpoints_reject_through_dead_choose_and_condition_paths() {
        for end in [
            r#"{"op":"input","input":"third","endpoint":"lower"}"#,
            r#"{"op":"offset","value":{"op":"input","input":"third","endpoint":"upper"},"delta":1}"#,
            r#"{"op":"choose","condition":{"op":"kindIs","input":"left","kind":"null"},"then":{"op":"empty"},"else":{"op":"input","input":"third","endpoint":"lower"}}"#,
            r#"{"op":"choose","condition":{"op":"booleanIs","input":"third","value":true},"then":{"op":"empty"},"else":{"op":"empty"}}"#,
        ] {
            for result in [
                format!(r#"{{"op":"bounds","lower":{end},"upper":{{"op":"empty"}}}}"#),
                format!(
                    r#"{{"op":"if","condition":{{"op":"hasEndpoint","value":{end}}},"then":{{"op":"integer","value":1}},"else":{{"op":"integer","value":0}}}}"#
                ),
                format!(
                    r#"{{"op":"if","condition":{{"op":"compare","left":{{"op":"integer","value":0}},"right":{end},"relation":"lessEqual"}},"then":{{"op":"integer","value":1}},"else":{{"op":"integer","value":0}}}}"#
                ),
            ] {
                for function in ["lowerBound", "unify"] {
                    let program = control(function, &result);
                    assert!(matches!(
                        super::generate_with_policy(&program, "not JSON"),
                        Err(super::Error::Program { .. })
                    ));
                }
                assert!(parse(&control("listSlice", &result)).is_ok());
            }
        }
    }

    #[test]
    fn slice_primitive_is_ternary_only_even_in_dead_arms() {
        let result = r#"{"op":"if","condition":{"op":"kindIs","input":"left","kind":"null"},"then":{"op":"input","input":"left"},"else":{"op":"listSlice"}}"#;
        for function in ["logicalNot", "unify", "listIndex"] {
            assert!(parse(&control(function, result)).is_err());
        }
        assert!(parse(&control("listSlice", result)).is_ok());
        for result in [
            r#"{"op":"input","input":"right"}"#,
            r#"{"op":"integerSum"}"#,
            r#"{"op":"listElement"}"#,
            r#"{"op":"integerPrimitive","operation":"difference"}"#,
        ] {
            assert!(parse(&control("logicalNot", result)).is_err());
            assert!(parse(&control("unify", result)).is_ok());
            assert!(parse(&control("listSlice", result)).is_ok());
        }
    }

    #[test]
    fn ternary_original_wire_rejects_duplicates_and_unknown_fields() {
        for result in [
            r#"{"op":"input","input":"third","input":"third"}"#,
            r#"{"op":"listSlice","op":"listSlice"}"#,
            r#"{"op":"listSlice","extra":0}"#,
            r#"{"op":"inputValue","input":"third"}"#,
            r#"{"op":"bounds","lower":{"op":"input","input":"third","endpoint":"lower","endpoint":"lower"},"upper":{"op":"empty"}}"#,
            r#"{"op":"if","condition":{"op":"kindIs","input":"third","input":"third","kind":"integer"},"then":{"op":"listSlice"},"else":{"op":"listSlice"}}"#,
        ] {
            assert!(parse(&control("listSlice", result)).is_err(), "{result}");
        }
        let program = control("listSlice", r#"{"op":"listSlice"}"#);
        assert!(
            parse(&program.replace(
                r#""function":"listSlice""#,
                r#""function":"listSlice","function":"listSlice""#
            ))
            .is_err()
        );
    }

    #[test]
    fn ternary_endpoint_scope_is_simultaneous_and_eager() {
        use crate::runtime::Value as Scalar;
        let result = r#"{"op":"letBounds","lower":{"op":"input","input":"right","endpoint":"lower"},"upper":{"op":"input","input":"third","endpoint":"upper"},"result":{"op":"letBounds","lower":{"op":"local","endpoint":"upper"},"upper":{"op":"local","endpoint":"lower"},"result":{"op":"bounds","lower":{"op":"local","endpoint":"lower"},"upper":{"op":"local","endpoint":"upper"}}}}"#;
        assert_eq!(
            super::interpret_ternary(
                &control("listSlice", result),
                Scalar::Null,
                Scalar::Integer(3),
                Scalar::Integer(8)
            )
            .ok(),
            Some(Scalar::Bounds {
                lower: Some(8),
                upper: Some(3)
            })
        );
        for result in [
            r#"{"op":"letBounds","lower":{"op":"integer","value":1},"upper":{"op":"local","endpoint":"lower"},"result":{"op":"listSlice"}}"#,
            r#"{"op":"if","condition":{"op":"kindIs","input":"third","kind":"integer"},"then":{"op":"listSlice"},"else":{"op":"bounds","lower":{"op":"local","endpoint":"lower"},"upper":{"op":"empty"}}}"#,
        ] {
            assert!(parse(&control("listSlice", result)).is_err());
        }
    }

    #[test]
    fn compiler_projection_preserves_old_arity_and_serializes_third() {
        struct Projection;
        impl crate::lowering::Driver for Projection {
            fn render(
                &self,
                _: &mut crate::lowering::Context,
                node: &Value,
                _: usize,
            ) -> Result<String, crate::lowering::Error> {
                Ok(node.to_string())
            }
        }
        for (function, unary, ternary, side) in [
            ("logicalNot", true, false, "left"),
            ("unify", false, false, "right"),
            ("listSlice", false, true, "third"),
        ] {
            let program = control(function, &format!(r#"{{"op":"input","input":"{side}"}}"#));
            let projected = super::lower(parse(&program).expect("valid control"), &Projection)
                .expect("projection");
            let node: Value = serde_json::from_str(&projected).expect("JSON projection");
            assert_eq!(node["function"], function);
            assert_eq!(node["unary"], unary);
            assert_eq!(node["ternary"], ternary);
            assert_eq!(node["result"]["op"], "inputValue");
            assert_eq!(node["result"]["input"], side);
        }
    }

    #[test]
    fn checked_offsets_and_recursive_execution() {
        use super::{End, offset_endpoint};
        use crate::runtime::Value as Scalar;

        for (value, delta, expected) in [
            (None, 1, None),
            (None, -1, None),
            (Some(i64::MIN), 1, Some(i64::MIN + 1)),
            (Some(i64::MAX), -1, Some(i64::MAX - 1)),
            (Some(7), 3, Some(10)),
        ] {
            assert_eq!(offset_endpoint(value, delta).ok(), Some(expected));
        }
        for (value, delta) in [(i64::MAX, 1), (i64::MIN, -1)] {
            assert!(matches!(
                offset_endpoint(Some(value), delta),
                Err(crate::Error::EndpointOffsetOverflow)
            ));
        }
        let end = End::Offset {
            value: Box::new(End::Offset {
                value: Box::new(End::Integer { value: 7 }),
                delta: 1,
            }),
            delta: -1,
        };
        assert_eq!(end.eval_unary(&Scalar::Null).ok(), Some(Some(7)));
        assert_eq!(
            end.eval(&Scalar::Null, &Scalar::Top, None, (None, None))
                .ok(),
            Some(Some(7))
        );
    }

    #[test]
    fn offset_units_and_unary_compare_scope() {
        use super::{Condition, End, Relation};
        use crate::runtime::Value as Scalar;

        for delta in [i64::MIN, -2, -1, 0, 1, 2, i64::MAX] {
            let end = End::Offset {
                value: Box::new(End::Empty {}),
                delta,
            };
            assert_eq!(end.validate(false).is_ok(), matches!(delta, -1 | 1));
            assert_eq!(end.validate_unary().is_ok(), matches!(delta, -1 | 1));
        }
        for end in [
            r#"{"op":"integer","value":7}"#,
            r#"{"op":"input","input":"left","endpoint":"lower"}"#,
            r#"{"op":"input","input":"right","endpoint":"lower"}"#,
            r#"{"op":"local","endpoint":"lower"}"#,
        ] {
            let condition: Condition = serde_json::from_str(&format!(
                r#"{{"op":"compare","left":{{"op":"offset","value":{end},"delta":1}},"right":{{"op":"integer","value":8}},"relation":"equal"}}"#
            ))
            .expect("valid condition shape");
            let allowed = !end.contains("right") && !end.contains("local");
            assert_eq!(condition.validate_unary().is_ok(), allowed);
            if allowed {
                assert_eq!(condition.eval_unary(&Scalar::Integer(7)).ok(), Some(true));
            }
            let choose = End::Choose {
                condition: Box::new(Condition::Compare {
                    left: End::Integer { value: 0 },
                    right: End::Integer { value: 0 },
                    relation: Relation::Equal,
                }),
                then: Box::new(End::Empty {}),
                otherwise: Box::new(End::Choose {
                    condition: Box::new(condition),
                    then: Box::new(End::Empty {}),
                    otherwise: Box::new(End::Empty {}),
                }),
            };
            assert_eq!(choose.validate_unary().is_ok(), allowed);
        }
    }

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

        for function in [
            "integerOutput",
            "jsonOutput",
            "lowerBound",
            "upperBound",
            "lowerExclusiveBound",
            "upperExclusiveBound",
        ] {
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
                    "emptyBound",
                    "exclusive bound is empty in the signed 64-bit integer domain",
                ),
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
