//! Finite CUE-authored control for scalar operations and its lowering.
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
pub(super) enum Side {
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
#[path = "kind.rs"]
mod kind;
pub use kind::Kind;
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
pub(super) enum Function {
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
    pub(super) fn arity(self) -> usize {
        if self.is_unary() {
            1
        } else if matches!(self, Self::ListSlice) {
            3
        } else {
            2
        }
    }

    pub(super) fn is_unary(self) -> bool {
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
pub(super) struct Program {
    format_version: u32,
    #[serde(default)]
    pub(super) function: Function,
    pub(super) result: Expr,
}
#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) enum IntegerOperation {
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
pub(super) enum Expr {
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
pub(super) enum End {
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
pub(super) enum Condition {
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
pub(super) fn limits(v: &serde_json::Value, depth: usize, count: &mut usize) -> Result<(), Error> {
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
pub(super) fn parse(input: &str) -> Result<Program, Error> {
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

    pub(super) fn validate_unary(&self) -> Result<(), Error> {
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

    pub(super) fn validate_unary(&self) -> Result<(), Error> {
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

    pub(super) fn validate(&self, bound: bool) -> Result<(), Error> {
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

    pub(super) fn validate_unary(&self) -> Result<(), Error> {
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
pub(super) fn project_unary(value: &mut serde_json::Value) {
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

pub(super) fn lower(p: Program, driver: &dyn crate::lowering::Driver) -> Result<String, Error> {
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
}
