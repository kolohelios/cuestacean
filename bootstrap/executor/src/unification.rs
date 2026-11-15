//! Finite, CUE-authored unification control and its Rust lowering.
use crate::runtime::Value;
use serde::Deserialize;
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("invalid unification control JSON: {source}"))]
    Json { source: serde_json::Error },
    #[snafu(display("invalid unification control: {reason}"))]
    Program { reason: &'static str },
}
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
enum Side {
    Left,
    Right,
}
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Endpoint {
    Lower,
    Upper,
}
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    Integer,
    Int,
    Bounds,
    Boolean,
    Bool,
}
#[derive(Clone, Copy, Deserialize)]
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
#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "camelCase", deny_unknown_fields)]
enum Expr {
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
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Failure {
    Integers,
    Bounds,
    Booleans,
    Types,
}
#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "camelCase", deny_unknown_fields)]
enum End {
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
#[derive(Deserialize)]
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
    fn rust(self) -> &'static str {
        match self {
            Self::Left => "_left",
            Self::Right => "_right",
        }
    }
    fn value(self, l: Value, r: Value) -> Value {
        match self {
            Self::Left => l,
            Self::Right => r,
        }
    }
}
impl Endpoint {
    fn rust(self) -> &'static str {
        match self {
            Self::Lower => "Lower",
            Self::Upper => "Upper",
        }
    }
    fn local(self) -> &'static str {
        match self {
            Self::Lower => "_lower",
            Self::Upper => "_upper",
        }
    }
    fn value(self, locals: (Option<i64>, Option<i64>)) -> Option<i64> {
        match self {
            Self::Lower => locals.0,
            Self::Upper => locals.1,
        }
    }
}
impl Kind {
    fn rust(self) -> &'static str {
        match self {
            Self::Integer => "Integer",
            Self::Int => "Int",
            Self::Bounds => "Bounds",
            Self::Boolean => "Boolean",
            Self::Bool => "Bool",
        }
    }
}
impl Relation {
    fn rust(self) -> &'static str {
        match self {
            Self::Equal => "Equal",
            Self::Less => "Less",
            Self::Greater => "Greater",
        }
    }
}
impl Failure {
    fn rust(self) -> &'static str {
        match self {
            Self::Integers => "Integers",
            Self::Bounds => "Bounds",
            Self::Booleans => "Booleans",
            Self::Types => "Types",
        }
    }
}
impl End {
    fn rust(&self) -> String {
        match self {
            Self::Input { input, endpoint } => format!(
                "Ok::<_, Error>(crate::unification::endpoint({}, crate::unification::Endpoint::{}))",
                input.rust(),
                endpoint.rust()
            ),
            Self::Local { endpoint } => format!("Ok::<_, Error>({})", endpoint.local()),
            Self::Empty {} => "Ok::<_, Error>(None::<i64>)".into(),
            Self::Choose {
                condition,
                then,
                otherwise,
            } => format!(
                "if ({})? {{ {} }} else {{ {} }}",
                condition.rust(),
                then.rust(),
                otherwise.rust()
            ),
        }
    }
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
    fn rust(&self) -> String {
        match self {
            Self::Same {} => "Ok::<_, Error>(_left == _right)".into(),
            Self::Or { left, right } => {
                format!("Ok::<_, Error>(({})? || ({})?)", left.rust(), right.rust())
            }
            Self::KindIs { input, kind } => format!(
                "Ok::<_, Error>(crate::unification::kind_is({}, crate::unification::Kind::{}))",
                input.rust(),
                kind.rust()
            ),
            Self::HasEndpoint { value } => format!("Ok::<_, Error>(({})?.is_some())", value.rust()),
            Self::Compare {
                left,
                right,
                relation,
            } => format!(
                "crate::unification::compare(({})?, ({})?, crate::unification::Relation::{})",
                left.rust(),
                right.rust(),
                relation.rust()
            ),
            Self::And { left, right } => {
                format!("Ok::<_, Error>(({})? && ({})?)", left.rust(), right.rust())
            }
        }
    }
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
    fn rust(&self) -> String {
        match self {
            Self::Input { input } => format!("Ok::<_, Error>({})", input.rust()),
            Self::If {
                condition,
                then,
                otherwise,
            } => format!(
                "if ({})? {{ {} }} else {{ {} }}",
                condition.rust(),
                then.rust(),
                otherwise.rust()
            ),
            Self::LetBounds {
                lower,
                upper,
                result,
            } => format!(
                "{{ let (_lower, _upper) = (({})?, ({})?); {} }}",
                lower.rust(),
                upper.rust(),
                result.rust()
            ),
            Self::Bounds { lower, upper } => format!(
                "Ok::<_, Error>(Value::Bounds {{ lower: ({})?, upper: ({})? }})",
                lower.rust(),
                upper.rust()
            ),
            Self::Fail { reason } => format!(
                "crate::unification::fail(crate::unification::Failure::{}, _left, _right)",
                reason.rust()
            ),
        }
    }
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
/// Lower validated control to native Rust control flow.
pub fn generate(input: &str) -> Result<String, Error> {
    let p = parse(input)?;
    Ok(format!(
        "// Generated by bootstrap-unify-codegen. Do not edit by hand.\nuse crate::{{Error, runtime::Value}};\npub fn unify(_left: Value, _right: Value) -> Result<Value, Error> {{\n{}\n}}\n",
        p.result.rust()
    ))
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
