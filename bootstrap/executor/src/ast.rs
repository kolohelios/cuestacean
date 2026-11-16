//! CUE-authored JSON construction expressions. Parsing control flow is separate;
//! this handwritten compiler/interpreter supplies their initial executable meaning.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use snafu::{ResultExt, Snafu};

#[cfg(feature = "source")]
#[path = "../../generated/ast-atom.rs"]
mod atom_builder;
#[cfg(feature = "source")]
#[path = "../../generated/ast-binary.rs"]
mod binary_builder;
#[cfg(feature = "source")]
#[path = "../../generated/ast-integer.rs"]
mod integer_builder;
#[cfg(feature = "source")]
#[path = "../../generated/ast-program.rs"]
mod program_builder;
#[cfg(feature = "source")]
#[path = "../../generated/ast-unary.rs"]
mod unary_builder;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("invalid AST builder JSON: {source}"))]
    Json { source: serde_json::Error },
    #[snafu(display("{source}"))]
    Lowering { source: crate::lowering::Error },
    #[snafu(display("invalid AST builder: {reason}"))]
    Program { reason: &'static str },
    #[snafu(display("invalid AST builder arguments: {reason}"))]
    Arguments { reason: &'static str },
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Type {
    String,
    Integer,
    Json,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Parameter {
    name: String,
    r#type: Type,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Program {
    format_version: u32,
    parameters: Vec<Parameter>,
    result: Expression,
}

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "lowercase", deny_unknown_fields)]
enum Expression {
    Argument { name: String },
    String { value: String },
    Integer { value: i64 },
    Object { fields: Vec<Field> },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Field {
    key: Expression,
    value: Expression,
}

type Bindings = BTreeMap<String, (usize, Type)>;

struct Validated {
    program: Program,
    bindings: Bindings,
    uses: Vec<usize>,
}

fn invalid(reason: &'static str) -> Error {
    Error::Program { reason }
}

fn check(
    expr: &Expression,
    bindings: &Bindings,
    uses: &mut [usize],
    depth: usize,
    nodes: &mut usize,
) -> Result<Type, Error> {
    if depth > 16 {
        return Err(invalid("exceeds 16 expression levels"));
    }
    *nodes += 1;
    if *nodes > 64 {
        return Err(invalid("exceeds 64 expression nodes"));
    }
    match expr {
        Expression::Argument { name } => {
            let (index, ty) = bindings
                .get(name)
                .ok_or_else(|| invalid("unknown argument"))?;
            uses[*index] += 1;
            Ok(*ty)
        }
        Expression::String { .. } => Ok(Type::String),
        Expression::Integer { .. } => Ok(Type::Integer),
        Expression::Object { fields } => {
            for field in fields {
                if check(&field.key, bindings, uses, depth + 1, nodes)? != Type::String {
                    return Err(invalid("object key must be a string"));
                }
                check(&field.value, bindings, uses, depth + 1, nodes)?;
            }
            Ok(Type::Json)
        }
    }
}

fn parse(input: &str) -> Result<Validated, Error> {
    let program: Program = serde_json::from_str(input).context(JsonSnafu)?;
    if program.format_version != 1 {
        return Err(invalid("formatVersion 1 required"));
    }
    if program.parameters.len() > 16 {
        return Err(invalid("exceeds 16 parameters"));
    }
    let mut bindings = Bindings::new();
    for (index, parameter) in program.parameters.iter().enumerate() {
        if parameter.name.is_empty()
            || bindings
                .insert(parameter.name.clone(), (index, parameter.r#type))
                .is_some()
        {
            return Err(invalid("empty or duplicate parameter name"));
        }
    }
    let mut uses = vec![0; program.parameters.len()];
    check(&program.result, &bindings, &mut uses, 0, &mut 0)?;
    Ok(Validated {
        program,
        bindings,
        uses,
    })
}

fn evaluate(expr: &Expression, bindings: &Bindings, arguments: &[Value]) -> Result<Value, Error> {
    match expr {
        Expression::Argument { name } => {
            let (index, _) = bindings
                .get(name)
                .ok_or_else(|| invalid("unknown argument"))?;
            Ok(arguments[*index].clone())
        }
        Expression::String { value } => Ok(Value::String(value.clone())),
        Expression::Integer { value } => Ok(Value::from(*value)),
        Expression::Object { fields } => {
            let mut object = serde_json::Map::new();
            for field in fields {
                let key = evaluate(&field.key, bindings, arguments)?;
                let key = key
                    .as_str()
                    .ok_or_else(|| invalid("object key must be a string"))?;
                object.insert(key.to_owned(), evaluate(&field.value, bindings, arguments)?);
            }
            Ok(Value::Object(object))
        }
    }
}

/// Reference evaluation with positional, dynamically checked JSON arguments.
pub fn interpret(input: &str, arguments: &[Value]) -> Result<Value, Error> {
    let validated = parse(input)?;
    if arguments.len() != validated.program.parameters.len() {
        return ArgumentsSnafu {
            reason: "argument count mismatch",
        }
        .fail();
    }
    for (parameter, argument) in validated.program.parameters.iter().zip(arguments) {
        let valid = match parameter.r#type {
            Type::String => argument.is_string(),
            Type::Integer => argument.as_i64().is_some(),
            Type::Json => true,
        };
        if !valid {
            return ArgumentsSnafu {
                reason: "argument type mismatch",
            }
            .fail();
        }
    }
    evaluate(&validated.program.result, &validated.bindings, arguments)
}

// Annotate checked facts in evaluation order, never Rust snippets or clone choices.
fn prepare(
    expr: &Expression,
    bindings: &Bindings,
    uses: &mut [usize],
    role: &str,
) -> Result<Value, Error> {
    Ok(match expr {
        Expression::Argument { name } => {
            let (index, ty) = bindings
                .get(name)
                .ok_or_else(|| invalid("unknown argument"))?;
            uses[*index] = uses[*index]
                .checked_sub(1)
                .ok_or_else(|| invalid("invalid argument usage"))?;
            serde_json::json!({"op":"argument","index":index,"type":ty,"role":role,"remaining":uses[*index]})
        }
        Expression::String { value } => {
            serde_json::json!({"op":"string","value":value,"role":role})
        }
        Expression::Integer { value } => serde_json::json!({"op":"integer","value":value}),
        Expression::Object { fields } => {
            let fields = fields
                .iter()
                .map(|field| {
                    let key = prepare(&field.key, bindings, uses, "key")?;
                    let value = prepare(&field.value, bindings, uses, "value")?;
                    Ok(serde_json::json!({"op":"field", "key":key, "value":value}))
                })
                .collect::<Result<Vec<_>, Error>>()?;
            serde_json::json!({"op":"object","fields":fields})
        }
    })
}

/// Compile via CUE-owned layout/ownership policy after native binding validation.
pub fn generate(input: &str) -> Result<String, Error> {
    #[cfg(feature = "compiled-lowering")]
    {
        lower(parse(input)?, &crate::compiled_ast_lowering::Compiled)
    }
    #[cfg(not(feature = "compiled-lowering"))]
    {
        generate_with_policy(
            input,
            include_str!(concat!(
                env!("CUESTACEAN_BOOTSTRAP_JSON"),
                "/ast-lowering.json"
            )),
        )
    }
}

pub fn generate_with_policy(input: &str, policy: &str) -> Result<String, Error> {
    let validated = parse(input)?;
    let policy =
        crate::lowering::Policy::parse(policy).map_err(|source| Error::Lowering { source })?;
    lower(validated, &policy)
}

fn lower(mut validated: Validated, driver: &dyn crate::lowering::Driver) -> Result<String, Error> {
    let parameters = validated
        .program
        .parameters
        .iter()
        .enumerate()
        .map(|(index, p)| serde_json::json!({"op":"parameter","index":index,"type":p.r#type}))
        .collect::<Vec<_>>();
    let result = prepare(
        &validated.program.result,
        &validated.bindings,
        &mut validated.uses,
        "value",
    )?;
    if validated.uses.iter().any(|n| *n != 0) {
        return Err(invalid("invalid argument usage"));
    }
    let node = serde_json::json!({"op":"program","parameters":parameters,"result":result});
    crate::lowering::Context::run(&node, driver).map_err(|source| Error::Lowering { source })
}

#[cfg(feature = "source")]
pub fn program(entry: Value) -> Value {
    program_builder::build(entry)
}
#[cfg(feature = "source")]
pub fn atom(kind: String) -> Value {
    atom_builder::build(kind)
}
#[cfg(feature = "source")]
pub fn integer(kind: String, value_field: String, value: i64) -> Value {
    integer_builder::build(kind, value_field, value)
}
#[cfg(feature = "source")]
pub fn unary(kind: String, operand_field: String, operand: Value) -> Value {
    unary_builder::build(kind, operand_field, operand)
}
#[cfg(feature = "source")]
pub fn binary(
    kind: String,
    left_field: String,
    right_field: String,
    left: Value,
    right: Value,
) -> Value {
    binary_builder::build(kind, left_field, right_field, left, right)
}

#[cfg(test)]
mod tests {
    use super::{Bindings, Expression, Field, check};

    #[test]
    fn expression_size_and_depth_are_bounded() {
        let fields = (0..32)
            .map(|_| Field {
                key: Expression::String { value: "x".into() },
                value: Expression::Integer { value: 1 },
            })
            .collect();
        assert!(
            check(
                &Expression::Object { fields },
                &Bindings::new(),
                &mut [],
                0,
                &mut 0
            )
            .is_err()
        );
        let mut expr = Expression::Integer { value: 1 };
        for _ in 0..16 {
            expr = Expression::Object {
                fields: vec![Field {
                    key: Expression::String { value: "x".into() },
                    value: expr,
                }],
            };
        }
        assert!(check(&expr, &Bindings::new(), &mut [], 0, &mut 0).is_ok());
        let expr = Expression::Object {
            fields: vec![Field {
                key: Expression::String { value: "x".into() },
                value: expr,
            }],
        };
        assert!(check(&expr, &Bindings::new(), &mut [], 0, &mut 0).is_err());
    }
}
