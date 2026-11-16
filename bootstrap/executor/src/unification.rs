//! Finite, CUE-authored unification control and its Rust lowering.
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
    Integer,
    Int,
    Bounds,
    Boolean,
    Bool,
}
#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Relation {
    Equal,
    Less,
    Greater,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Program {
    format_version: u32,
    result: Expr,
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
    Fail {
        reason: Failure,
    },
}
#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Failure {
    Integers,
    Bounds,
    Booleans,
    Types,
}
#[derive(Deserialize, Serialize)]
#[serde(tag = "op", rename_all = "camelCase", deny_unknown_fields)]
enum End {
    #[serde(rename(serialize = "inputEndpoint", deserialize = "input"))]
    Input {
        input: Side,
        endpoint: Endpoint,
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
    Or {
        left: Box<Condition>,
        right: Box<Condition>,
    },
    KindIs {
        input: Side,
        kind: Kind,
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
    Ok(p)
}
impl Expr {
    fn validate(&self, bound: bool) -> Result<(), Error> {
        match self {
            Self::Input { .. } | Self::Fail { .. } => Ok(()),
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
    fn validate(&self, bound: bool) -> Result<(), Error> {
        match self {
            Self::KindIs { .. } | Self::Same {} => Ok(()),
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
pub fn endpoint(value: Value, end: Endpoint) -> Option<i64> {
    match value {
        Value::Integer(n) => Some(n),
        Value::Int | Value::Bool | Value::Boolean(_) => None,
        Value::Bounds { lower, upper } => match end {
            Endpoint::Lower => lower,
            Endpoint::Upper => upper,
        },
    }
}
pub fn kind_is(value: Value, kind: Kind) -> bool {
    matches!(
        (value, kind),
        (Value::Integer(_), Kind::Integer)
            | (Value::Int, Kind::Int)
            | (Value::Bounds { .. }, Kind::Bounds)
            | (Value::Boolean(_), Kind::Boolean)
            | (Value::Bool, Kind::Bool)
    )
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
        Relation::Greater => left > right,
    })
}
pub fn fail(reason: Failure, left: Value, right: Value) -> Result<Value, crate::Error> {
    match reason {
        Failure::Bounds => Err(crate::Error::BoundsConflict),
        Failure::Types => Err(crate::Error::TypeConflict),
        Failure::Booleans => match (left, right) {
            (Value::Boolean(left), Value::Boolean(right)) => {
                Err(crate::Error::BooleanConflict { left, right })
            }
            _ => Err(crate::Error::UnificationState {
                reason: "boolean conflict requires booleans",
            }),
        },
        Failure::Integers => match (left, right) {
            (Value::Integer(left), Value::Integer(right)) => {
                Err(crate::Error::Conflict { left, right })
            }
            _ => Err(crate::Error::UnificationState {
                reason: "integer conflict requires integers",
            }),
        },
    }
}
impl Side {
    fn value(self, l: Value, r: Value) -> Value {
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
    fn eval(
        &self,
        l: Value,
        r: Value,
        locals: (Option<i64>, Option<i64>),
    ) -> Result<Option<i64>, crate::Error> {
        Ok(match self {
            Self::Input { input, endpoint: e } => endpoint(input.value(l, r), *e),
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
    fn eval(
        &self,
        l: Value,
        r: Value,
        locals: (Option<i64>, Option<i64>),
    ) -> Result<bool, crate::Error> {
        Ok(match self {
            Self::Same {} => l == r,
            Self::Or { left, right } => left.eval(l, r, locals)? || right.eval(l, r, locals)?,
            Self::KindIs { input, kind } => kind_is(input.value(l, r), *kind),
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
    fn eval(
        &self,
        l: Value,
        r: Value,
        locals: (Option<i64>, Option<i64>),
    ) -> Result<Value, crate::Error> {
        match self {
            Self::Input { input } => Ok(input.value(l, r)),
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

fn lower(p: Program, driver: &dyn crate::lowering::Driver) -> Result<String, Error> {
    let node = serde_json::json!({"op":"program", "result":p.result});
    crate::lowering::Context::run(&node, driver).map_err(|source| Error::Lowering { source })
}
/// Execute the same validated control directly, for differential testing.
pub fn interpret(input: &str, left: Value, right: Value) -> Result<Value, crate::Error> {
    let p = parse(input).map_err(|source| crate::Error::UnificationProgram { source })?;
    p.result.eval(left, right, (None, None))
}

#[cfg(test)]
mod tests {
    use super::limits;
    use serde_json::Value;

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
