//! Bounded flat scalar vet profile; all constraint execution stays generated.
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::io::Read;
use std::path::Path;

use bootstrap_executor::{Error as RuntimeError, ast, generated, parser};
use serde_json::{Value, json};

const MAX_BYTES: usize = 1024 * 1024;
const GRAMMAR: &str = include_str!(concat!(env!("CUESTACEAN_BOOTSTRAP_JSON"), "/grammar.json"));
const USAGE: &str = "usage: cuestacean vet -c schema.cue data.json";

#[derive(Debug)]
pub(super) enum Error {
    Usage,
    Setup(String),
    InvalidSchema(parser::Error),
    UnsupportedSchema(String),
    UnsupportedData(&'static str),
    Json(String),
    Evaluation { field: String, source: RuntimeError },
}

#[derive(Debug, PartialEq)]
enum Class {
    Conflict,
    Incomplete,
    Setup,
}

fn classify(error: &RuntimeError) -> Class {
    match error {
        RuntimeError::Conflict { .. }
        | RuntimeError::BooleanConflict { .. }
        | RuntimeError::StringConflict { .. }
        | RuntimeError::TypeConflict
        | RuntimeError::BoundsConflict => Class::Conflict,
        RuntimeError::Incomplete
        | RuntimeError::IncompleteValue
        | RuntimeError::IncompleteBool
        | RuntimeError::IncompleteString
        | RuntimeError::IncompleteBounds => Class::Incomplete,
        _ => Class::Setup,
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage => f.write_str(USAGE),
            Self::Setup(reason) => write!(f, "vet setup: {reason}"),
            Self::InvalidSchema(source) => write!(f, "invalid vet schema: {source}"),
            Self::UnsupportedSchema(reason) => write!(f, "unsupported vet schema: {reason}"),
            Self::UnsupportedData(reason) => write!(f, "unsupported vet data: {reason}"),
            Self::Json(reason) => f.write_str(reason),
            Self::Evaluation { field, source } => {
                let class = match classify(source) {
                    Class::Conflict => "conflict",
                    Class::Incomplete => "incomplete",
                    Class::Setup => "setup",
                };
                write!(f, "vet {class}: field {field}: {source}")
            }
        }
    }
}

fn arguments(args: impl Iterator<Item = OsString>) -> Result<(OsString, OsString), Error> {
    let mut args = args;
    if args.next().as_deref() != Some(OsStr::new("vet"))
        || args.next().as_deref() != Some(OsStr::new("-c"))
    {
        return Err(Error::Usage);
    }
    let schema = args.next().ok_or(Error::Usage)?;
    let data = args.next().ok_or(Error::Usage)?;
    if args.next().is_some()
        || Path::new(&schema).extension() != Some(OsStr::new("cue"))
        || Path::new(&data).extension() != Some(OsStr::new("json"))
        || schema.as_encoded_bytes().starts_with(b"-")
        || data.as_encoded_bytes().starts_with(b"-")
    {
        return Err(Error::Usage);
    }
    Ok((schema, data))
}

fn schema_text(reader: impl Read) -> Result<String, Error> {
    let mut bytes = Vec::new();
    reader
        .take((MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| Error::Setup(format!("cannot read schema: {error}")))?;
    if bytes.len() > MAX_BYTES {
        return Err(Error::Setup("schema exceeds 1 MiB".into()));
    }
    String::from_utf8(bytes).map_err(|error| Error::Setup(format!("schema must be UTF-8: {error}")))
}

struct Field {
    name: String,
    term: Value,
}

#[derive(Default)]
struct Terms {
    bound: bool,
    int: bool,
}

fn unsupported(reason: impl Into<String>) -> Error {
    Error::UnsupportedSchema(reason.into())
}

// Syntax admission only: no interval arithmetic, refinement, or evaluation.
fn admit(node: &Value, nodes: &mut usize) -> Result<Terms, Error> {
    *nodes += 1;
    if *nodes > 63 {
        return Err(Error::Setup("schema exceeds 63 scalar AST nodes".into()));
    }
    let object = node
        .as_object()
        .ok_or_else(|| unsupported("expected scalar AST"))?;
    let kind = node
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(|| unsupported("missing scalar kind"))?;
    let mut terms = Terms::default();
    let keys: &[&str] = match kind {
        "true" | "false" | "null" | "boolType" | "stringType" => &["kind"],
        "intType" => {
            terms.int = true;
            &["kind"]
        }
        "string" if node.get("value").is_some_and(Value::is_string) => &["kind", "value"],
        "integer" | "lowerBound" | "upperBound" | "lowerExclusiveBound" | "upperExclusiveBound" => {
            let value = node
                .get("value")
                .and_then(Value::as_i64)
                .ok_or_else(|| unsupported("literal signed i64 required"))?;
            terms.bound = kind != "integer";
            if (kind == "lowerExclusiveBound" && value == i64::MAX)
                || (kind == "upperExclusiveBound" && value == i64::MIN)
            {
                return Err(unsupported("exclusive bound outside signed i64 domain"));
            }
            &["kind", "value"]
        }
        "unify" => {
            let left = admit(
                node.get("left")
                    .ok_or_else(|| unsupported("missing left term"))?,
                nodes,
            )?;
            let right = admit(
                node.get("right")
                    .ok_or_else(|| unsupported("missing right term"))?,
                nodes,
            )?;
            terms.bound = left.bound || right.bound;
            terms.int = left.int || right.int;
            &["kind", "left", "right"]
        }
        _ => return Err(unsupported(format!("scalar kind {kind:?}"))),
    };
    if object.len() != keys.len() || !keys.iter().all(|key| object.contains_key(*key)) {
        return Err(unsupported("unexpected scalar AST fields"));
    }
    Ok(terms)
}

fn schema(source: &str) -> Result<Vec<Field>, Error> {
    let program = parser::parse_fields_file(source, GRAMMAR).map_err(|error| match error {
        parser::Error::Source { .. } | parser::Error::Integer { .. } => Error::InvalidSchema(error),
        _ => Error::Setup(error.to_string()),
    })?;
    let fields = program
        .get("entry")
        .and_then(|entry| entry.get("fields"))
        .and_then(Value::as_array)
        .ok_or_else(|| Error::Setup("missing file fields".into()))?;
    let mut nodes = 0;
    let mut result = Vec::with_capacity(fields.len());
    for field in fields {
        let name = field
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| Error::Setup("missing file label".into()))?;
        if name.len() > 64 {
            return Err(unsupported("field label exceeds 64 bytes"));
        }
        let term = field
            .get("value")
            .ok_or_else(|| Error::Setup("missing field term".into()))?;
        let terms = admit(term, &mut nodes)?;
        if terms.bound && !terms.int {
            return Err(unsupported(format!(
                "field {name}: bounds require an explicit int term"
            )));
        }
        result.push(Field {
            name: name.into(),
            term: term.clone(),
        });
    }
    Ok(result)
}

fn scalar(value: &Value) -> Result<Value, Error> {
    match value {
        Value::Null => Ok(json!({"kind":"null"})),
        Value::Bool(true) => Ok(json!({"kind":"true"})),
        Value::Bool(false) => Ok(json!({"kind":"false"})),
        Value::String(text) => Ok(json!({"kind":"string","value":text})),
        Value::Number(number) => number
            .as_i64()
            .map(|value| json!({"kind":"integer","value":value}))
            .ok_or(Error::UnsupportedData("signed i64 scalar required")),
        _ => Err(Error::UnsupportedData("known fields require scalar values")),
    }
}

fn bridge(term: &Value, candidate: Option<&Value>) -> Result<Value, Error> {
    let entry = match candidate {
        Some(value) => ast::binary(
            "unify".into(),
            "left".into(),
            "right".into(),
            term.clone(),
            scalar(value)?,
        ),
        None => term.clone(),
    };
    Ok(ast::program(entry))
}

fn validate(fields: &[Field], data: &Value) -> Result<(), Error> {
    let data = data
        .as_object()
        .ok_or(Error::UnsupportedData("object root required"))?;
    // Complete every shape check before the first semantic failure. Extras are
    // already fully decoded, but never enter the evaluator's collection profile.
    for field in fields {
        if data
            .get(&field.name)
            .is_some_and(|value| value.is_array() || value.is_object())
        {
            return Err(Error::UnsupportedData("known fields require scalar values"));
        }
    }
    // sum(field AST nodes) <= 63; each supplied scalar adds two visits. With
    // <= 63 fields this is <= 189 cumulative visits, without retries/repetition.
    // Only this field's term and scalar are cloned/serialized, never other data.
    for field in fields {
        let program = bridge(&field.term, data.get(&field.name))?;
        let input = serde_json::to_string(&program)
            .map_err(|error| Error::Setup(format!("cannot serialize vet program: {error}")))?;
        generated::execute_json(&input).map_err(|source| Error::Evaluation {
            field: field.name.clone(),
            source,
        })?;
    }
    Ok(())
}

pub(super) fn run(args: impl Iterator<Item = OsString>) -> Result<(), Error> {
    let (schema_path, data_path) = arguments(args)?;
    let open = |path: &OsStr| {
        std::fs::File::open(path).map_err(|error| {
            Error::Setup(format!(
                "cannot read {}: {error}",
                Path::new(path).display()
            ))
        })
    };
    let fields = schema(&schema_text(open(&schema_path)?)?)?;
    let data = super::json::read(open(&data_path)?).map_err(Error::Json)?;
    validate(&fields, &data)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(source: &str, data: Value) -> Result<(), Error> {
        validate(&schema(source)?, &data)
    }

    #[test]
    fn exact_arguments() {
        for args in [
            vec!["vet", "-c", "schema.cue", "data.json"],
            vec!["vet", "-c", "./-schema.cue", "./-data.json"],
        ] {
            assert!(arguments(args.into_iter().map(OsString::from)).is_ok());
        }
        for args in [
            vec!["vet"],
            vec!["vet", "schema.cue", "data.json"],
            vec!["vet", "-c=false", "schema.cue", "data.json"],
            vec!["vet", "-c", "schema.CUE", "data.json"],
            vec!["vet", "-c", "schema.cue", "data.JSON"],
            vec!["vet", "-c", "schema.cue", "-"],
            vec!["vet", "-c", "-schema.cue", "data.json"],
            vec!["vet", "-c", "schema.cue", "data.json", "extra.json"],
            vec!["vet", "data.json", "-c", "schema.cue"],
            vec!["vet", "-c", "schema.cue", "--", "data.json"],
        ] {
            assert!(matches!(
                arguments(args.into_iter().map(OsString::from)),
                Err(Error::Usage)
            ));
        }
    }

    #[test]
    fn exact_bridge_ast() -> Result<(), Error> {
        let term = json!({"kind":"unify","left":{"kind":"intType"},
            "right":{"kind":"lowerBound","value":0}});
        assert_eq!(
            bridge(&term, Some(&json!(42)))?,
            json!({"formatVersion":1,"entry":{
            "kind":"unify","left":{"kind":"unify","left":{"kind":"intType"},
            "right":{"kind":"lowerBound","value":0}},"right":{"kind":"integer","value":42}}})
        );
        assert_eq!(
            bridge(&term, None)?,
            json!({"formatVersion":1,"entry":term})
        );
        for (data, expected) in [
            (Value::Null, json!({"kind":"null"})),
            (json!(true), json!({"kind":"true"})),
            (json!(false), json!({"kind":"false"})),
            (json!("\"é\n"), json!({"kind":"string","value":"\"é\n"})),
            (json!(i64::MIN), json!({"kind":"integer","value":i64::MIN})),
            (json!(i64::MAX), json!({"kind":"integer","value":i64::MAX})),
        ] {
            assert_eq!(scalar(&data)?, expected);
        }
        Ok(())
    }

    #[test]
    fn admission_is_syntax_only_and_whole_schema() {
        for source in [
            "x: and([int, 1])",
            "x: _",
            "x: >=(0)",
            "x: 1+1",
            "x: -(2)",
            "x: close({\"y\":1})",
            "x: [1]",
            "x: len(\"a\")",
            "x: int == 1",
            "x: int\ny: 1+1",
            "x: 1 & 2\ny: _",
            "x: >0 & <1",
            "x: >=42 & <=42",
            "x: int & >9223372036854775807",
            "x: < -9223372036854775808 & int",
        ] {
            assert!(
                matches!(schema(source), Err(Error::UnsupportedSchema(_))),
                "{source}"
            );
        }
        for source in [
            "close({\"x\":int})",
            "(close({\"x\":1}))",
            "42",
            "",
            "// empty\n",
        ] {
            assert!(
                matches!(schema(source), Err(Error::InvalidSchema(_))),
                "{source}"
            );
        }
        let long = format!("{}: int", "a".repeat(65));
        assert!(matches!(schema(&long), Err(Error::UnsupportedSchema(_))));
        for source in [
            "x: >0 & <1 & int",
            "x: int & >=42 & <=42",
            "x: <=42 & int & >=42",
        ] {
            assert!(schema(source).is_ok(), "{source}");
        }
    }

    #[test]
    fn generated_semantics_and_missing_values() -> Result<(), Error> {
        for source in ["x: int & >=0 & <10", "x: <10 & >=0 & int"] {
            check(source, json!({"x":0,"extra":[null,{"nested":true}]}))?;
            check(source, json!({"x":9}))?;
            for data in [
                json!({"x":-1}),
                json!({"x":10}),
                json!({"x":"0"}),
                json!({"x":null}),
            ] {
                let error = check(source, data)
                    .err()
                    .ok_or_else(|| Error::Setup("expected conflict".into()))?;
                assert!(error.to_string().starts_with("vet conflict: field x:"));
            }
        }
        for source in [
            "x: 42",
            "x: int & 42",
            "x: 42 & int",
            "x: null",
            "x: true",
            "x: \"a\"",
        ] {
            check(source, json!({}))?;
        }
        for source in [
            "x: int",
            "x: bool",
            "x: string",
            "x: int & >=42 & <=42",
            "x: <=42 & >=42 & int",
        ] {
            let error = check(source, json!({}))
                .err()
                .ok_or_else(|| Error::Setup("expected incomplete".into()))?;
            assert!(error.to_string().starts_with("vet incomplete: field x:"));
        }
        for source in ["x: int & >=42 & <=42", "x: <=42 & >=42 & int"] {
            check(source, json!({"x":42}))?;
        }
        for source in ["x: int & >0 & <1", "x: <1 & >0 & int", "x: 1 & 2"] {
            let error = check(source, json!({}))
                .err()
                .ok_or_else(|| Error::Setup("expected conflict".into()))?;
            assert!(error.to_string().starts_with("vet conflict:"));
        }
        Ok(())
    }

    #[test]
    fn typed_bound_edges_and_untyped_exclusions() -> Result<(), Error> {
        for (source, value) in [
            ("x: int & >=9223372036854775807", i64::MAX),
            ("x: <= -9223372036854775808 & int", i64::MIN),
            ("x: int & >9223372036854775806", i64::MAX),
            ("x: < -9223372036854775807 & int", i64::MIN),
        ] {
            check(source, json!({"x":value}))?;
            assert!(matches!(
                check(source, json!({})),
                Err(Error::Evaluation {
                    source: RuntimeError::IncompleteBounds,
                    ..
                })
            ));
        }
        for source in ["x: >0 & <1", "x: <1 & >0", "x: >=0", "x: >=42 & <=42"] {
            for data in [json!({}), json!({"x":0}), json!({"x":42})] {
                assert!(matches!(
                    check(source, data),
                    Err(Error::UnsupportedSchema(_))
                ));
            }
        }
        for (source, value) in [
            ("x: bool", json!(true)),
            ("x: bool", json!(false)),
            ("x: string", json!("é")),
            ("x: null", Value::Null),
        ] {
            check(source, json!({"x":value}))?;
        }
        Ok(())
    }

    #[test]
    fn shapes_precede_semantics_and_json_is_fully_decoded() -> Result<(), Error> {
        for data in [json!({"y":[]}), json!({"x":"wrong","y":{}})] {
            assert!(matches!(
                check("x: int\ny: bool", data),
                Err(Error::UnsupportedData(_))
            ));
        }
        assert!(matches!(
            check("x: int", json!([])),
            Err(Error::UnsupportedData(_))
        ));
        for text in [
            r#"{"x":1,"extra":{"a":1,"a":2}}"#,
            r#"{"x":1,"extra":0.5}"#,
            r#"{"x":1} junk"#,
        ] {
            assert!(super::super::json::read(text.as_bytes()).is_err());
        }
        check("x: int", json!({"x":1,"extra":{"list":[null]}}))?;
        Ok(())
    }

    #[test]
    fn typed_runtime_classes_and_bounded_reads() -> Result<(), Error> {
        for error in [
            RuntimeError::TypeConflict,
            RuntimeError::BoundsConflict,
            RuntimeError::Conflict { left: 1, right: 2 },
            RuntimeError::BooleanConflict {
                left: true,
                right: false,
            },
            RuntimeError::StringConflict {
                left: "a".into(),
                right: "b".into(),
            },
        ] {
            assert_eq!(classify(&error), Class::Conflict);
        }
        for error in [
            RuntimeError::Incomplete,
            RuntimeError::IncompleteValue,
            RuntimeError::IncompleteBool,
            RuntimeError::IncompleteString,
            RuntimeError::IncompleteBounds,
        ] {
            assert_eq!(classify(&error), Class::Incomplete);
        }
        for error in [
            RuntimeError::EvaluationVisits,
            RuntimeError::EvaluationDepth,
            RuntimeError::EmptyBound,
            RuntimeError::OutputPolicy,
            RuntimeError::EndpointOffsetOverflow,
            RuntimeError::Kind,
            RuntimeError::MissingRule {
                kind: "unknown".into(),
            },
        ] {
            assert_eq!(classify(&error), Class::Setup);
        }
        let mut reader = std::io::Cursor::new(vec![b' '; MAX_BYTES + 100]);
        assert!(matches!(schema_text(&mut reader), Err(Error::Setup(_))));
        assert_eq!(reader.position(), (MAX_BYTES + 1) as u64);
        assert_eq!(schema_text(&vec![b' '; MAX_BYTES][..])?.len(), MAX_BYTES);
        assert!(matches!(schema_text(&[0xff][..]), Err(Error::Setup(_))));
        Ok(())
    }
}
