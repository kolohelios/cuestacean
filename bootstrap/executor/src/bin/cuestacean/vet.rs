//! Bounded scalar/explicit closed-record vet; scalar execution stays generated.
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
    RecordType { field: String },
    ClosedField { field: String },
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
            Self::RecordType { field } => {
                write!(f, "vet conflict: field {field}: expected a record")
            }
            Self::ClosedField { field } => {
                write!(f, "vet conflict: field {field}: field not allowed")
            }
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
    term: Term,
}

enum Term {
    Scalar(Value),
    Closed {
        fields: Vec<Field>,
        names: std::collections::BTreeSet<String>,
    },
}

fn identifier(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_alphabetic())
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn path(parent: &str, name: &str) -> String {
    if identifier(name) {
        if parent.is_empty() {
            name.into()
        } else {
            format!("{parent}.{name}")
        }
    } else {
        // Serializing a string cannot fail and escapes control characters.
        format!("{parent}[{}]", json!(name))
    }
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

// Decode the entire schema before inspecting data. Records under scalar unify
// stay on the old scalar admission path and are therefore unsupported.
fn decode_fields(fields: &[Value], nodes: &mut usize, parent: &str) -> Result<Vec<Field>, Error> {
    let mut names = std::collections::BTreeSet::new();
    let mut result = Vec::with_capacity(fields.len());
    for field in fields {
        let object = field
            .as_object()
            .ok_or_else(|| unsupported("expected field wrapper"))?;
        if object.len() != 2 || !object.contains_key("name") || !object.contains_key("value") {
            return Err(unsupported("unexpected field AST fields"));
        }
        let name = field
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| unsupported("missing field label"))?;
        if name.len() > 64 {
            return Err(unsupported("field label exceeds 64 bytes"));
        }
        if !identifier(name) {
            return Err(unsupported(
                "field labels require visible ASCII identifiers",
            ));
        }
        if !names.insert(name) {
            return Err(unsupported(format!(
                "duplicate field {}",
                path(parent, name)
            )));
        }
        let term = field
            .get("value")
            .ok_or_else(|| unsupported("missing field term"))?;
        let field_path = path(parent, name);
        let term = if term.get("kind").and_then(Value::as_str) == Some("recordExpression") {
            *nodes += 1;
            if *nodes > 63 {
                return Err(Error::Setup("schema exceeds 63 AST nodes".into()));
            }
            let object = term
                .as_object()
                .ok_or_else(|| unsupported("expected record AST"))?;
            if object.len() != 2 || !object.contains_key("fields") {
                return Err(unsupported("unexpected record AST fields"));
            }
            let children = term
                .get("fields")
                .and_then(Value::as_array)
                .ok_or_else(|| unsupported("expected record fields"))?;
            let fields = decode_fields(children, nodes, &field_path)?;
            let names = fields.iter().map(|field| field.name.clone()).collect();
            Term::Closed { fields, names }
        } else {
            let terms = admit(term, nodes)?;
            if terms.bound && !terms.int {
                return Err(unsupported(format!(
                    "field {field_path}: bounds require an explicit int term"
                )));
            }
            Term::Scalar(term.clone())
        };
        result.push(Field {
            name: name.into(),
            term,
        });
    }
    Ok(result)
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
    decode_fields(fields, &mut 0, "")
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

// Three bounded passes: known-scalar shapes, record structure, scalar execution.
// Missing records pass None down, never synthetic null or an empty runtime value.
fn shapes(fields: &[Field], data: Option<&serde_json::Map<String, Value>>) -> Result<(), Error> {
    for field in fields {
        let candidate = data.and_then(|data| data.get(&field.name));
        match &field.term {
            Term::Scalar(_) => {
                if candidate.is_some_and(|value| value.is_array() || value.is_object()) {
                    return Err(Error::UnsupportedData("known fields require scalar values"));
                }
            }
            Term::Closed { fields, .. } => {
                // Mismatched record candidates are deferred to the structural pass.
                if let Some(object) = candidate.and_then(Value::as_object) {
                    shapes(fields, Some(object))?;
                }
            }
        }
    }
    Ok(())
}

fn structure(
    fields: &[Field],
    data: Option<&serde_json::Map<String, Value>>,
    parent: &str,
) -> Result<(), Error> {
    for field in fields {
        if let Term::Closed { fields, names } = &field.term {
            let field_path = path(parent, &field.name);
            let candidate = data.and_then(|data| data.get(&field.name));
            let object = match candidate {
                None => None,
                Some(Value::Object(object)) => Some(object),
                Some(_) => return Err(Error::RecordType { field: field_path }),
            };
            if let Some(object) = object {
                // Explicit min keeps ordering independent of serde map features.
                if let Some(extra) = object.keys().filter(|key| !names.contains(*key)).min() {
                    return Err(Error::ClosedField {
                        field: path(&field_path, extra),
                    });
                }
            }
            structure(fields, object, &field_path)?;
        }
    }
    Ok(())
}

fn leaves(
    fields: &[Field],
    data: Option<&serde_json::Map<String, Value>>,
    parent: &str,
) -> Result<(), Error> {
    for field in fields {
        let candidate = data.and_then(|data| data.get(&field.name));
        let field_path = path(parent, &field.name);
        match &field.term {
            Term::Closed { fields, .. } => {
                leaves(fields, candidate.and_then(Value::as_object), &field_path)?;
            }
            Term::Scalar(term) => {
                let program = bridge(term, candidate)?;
                let input = serde_json::to_string(&program).map_err(|error| {
                    Error::Setup(format!("cannot serialize vet program: {error}"))
                })?;
                generated::execute_json(&input).map_err(|source| Error::Evaluation {
                    field: field_path,
                    source,
                })?;
            }
        }
    }
    Ok(())
}

fn validate(fields: &[Field], data: &Value) -> Result<(), Error> {
    let data = data
        .as_object()
        .ok_or(Error::UnsupportedData("object root required"))?;
    shapes(fields, Some(data))?;
    structure(fields, Some(data), "")?;
    // S scalar AST nodes + R records <=63; L scalar leaves <=S. Each supplied
    // scalar adds two visits, so total generated visits S+2L<=189. No retries,
    // and only each leaf term/candidate is serialized, never sibling/extra data.
    leaves(fields, Some(data), "")
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
            "x: close({\"y\":1}) & close({})",
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
    fn recursive_ast_and_decoded_label_admission() -> Result<(), Error> {
        let source = r#"person: close({"age":int,"child":close({"units":"years"})})"#;
        let expected = json!({"formatVersion":1,"entry":{"kind":"recordExpression","fields":[
            {"name":"person","value":{"kind":"recordExpression","fields":[
                {"name":"age","value":{"kind":"intType"}},
                {"name":"child","value":{"kind":"recordExpression","fields":[
                    {"name":"units","value":{"kind":"string","value":"years"}}
                ]}}
            ]}}
        ]}});
        assert_eq!(
            parser::parse_fields_file(source, GRAMMAR).map_err(Error::InvalidSchema)?,
            expected
        );
        let fields = schema(source)?;
        let Term::Closed { fields, names } = &fields[0].term else {
            return Err(Error::Setup("expected closed schema term".into()));
        };
        assert_eq!(fields[0].name, "age");
        assert!(names.contains("age") && names.contains("child"));
        check(
            r#"r: close({"\u0061":int,"a_2":close({"a":true})})"#,
            json!({"r":{"a":1}}),
        )?;
        for source in [
            r#"r: close({"a":1,"\u0061":1})"#,
            r#"r: close({"":1})"#,
            r#"r: close({"_hidden":1})"#,
            r#"r: close({"1a":1})"#,
            r#"r: close({"a.b":1})"#,
            r#"r: close({"a\n":1})"#,
            r#"r: close({"é":1})"#,
            r#"r: close({"\u00e9":1})"#,
            r#"r: close({"x":int & close({})})"#,
            r#"r: close({"x":close({}) & close({})})"#,
            r#"r: close({"x":>=0,"y":int})"#,
            r#"r: close({"x":int & >=0,"y":>=0})"#,
            r#"r: close({"x":1,"later":_})"#,
            r#"r: close({"x":[1]})"#,
        ] {
            assert!(
                matches!(schema(source), Err(Error::UnsupportedSchema(_))),
                "{source}"
            );
        }
        for length in [64, 65] {
            let source = format!("r: close({{\"{}\":1}})", "a".repeat(length));
            assert_eq!(schema(&source).is_ok(), length == 64);
        }
        // Malformed wrappers/record metadata are never silently ignored.
        for node in [
            json!({"kind":"recordExpression","fields":[],"extra":true}),
            json!({"kind":"recordExpression","fields":null}),
            json!({"kind":"recordExpression","fields":[{"name":"x","value":{"kind":"intType"},"extra":0}]}),
            json!({"kind":"recordExpression","fields":[{"name":0,"value":{"kind":"intType"}}]}),
        ] {
            assert!(matches!(
                decode_fields(&[json!({"name":"r","value":node})], &mut 0, ""),
                Err(Error::UnsupportedSchema(_))
            ));
        }
        Ok(())
    }

    #[test]
    fn recursive_absence_null_empty_and_mixed_leaves() -> Result<(), Error> {
        for source in [
            r#"r: close({})"#,
            r#"r: close({"units":"years"})"#,
            r#"r: close({"child":close({"n":int & 42,"empty":close({})})})"#,
        ] {
            check(source, json!({}))?;
            check(source, json!({"r":{}}))?;
            check(source, json!({"extra":[{"arbitrary":true}]}))?;
            for candidate in [Value::Null, json!([]), json!(0), json!(true), json!("x")] {
                let error = check(source, json!({"r":candidate}))
                    .err()
                    .ok_or_else(|| Error::Setup("expected record conflict".into()))?;
                assert!(matches!(&error, Error::RecordType { field } if field == "r"));
                assert_eq!(
                    error.to_string(),
                    "vet conflict: field r: expected a record"
                );
            }
        }
        let source = r#"r: close({"child":close({"n":int,"units":"years"})})
flag: bool"#;
        for document in [json!({}), json!({"r":{}}), json!({"r":{"child":{}}})] {
            assert!(matches!(check(source, document), Err(Error::Evaluation {
                field, source: RuntimeError::Incomplete, ..
            }) if field == "r.child.n"));
        }
        check(
            source,
            json!({"r":{"child":{"n":42}},"flag":true,"extra":[]}),
        )?;
        assert!(
            matches!(check(source, json!({"r":{"child":{"n":null}},"flag":true})),
            Err(Error::Evaluation { field, source: RuntimeError::TypeConflict })
                if field == "r.child.n")
        );
        assert!(matches!(check(r#"r: close({"n":1 & 2})"#, json!({})),
            Err(Error::Evaluation { field, source: RuntimeError::Conflict { .. } })
                if field == "r.n"));
        for source in [
            r#"r: close({"n":int & >=42 & <=42})"#,
            r#"r: close({"n":<=42 & int & >=42})"#,
        ] {
            assert!(matches!(
                check(source, json!({})),
                Err(Error::Evaluation {
                    source: RuntimeError::IncompleteBounds,
                    ..
                })
            ));
            check(source, json!({"r":{"n":42}}))?;
        }
        Ok(())
    }

    #[test]
    fn structural_precedence_and_safe_unknown_paths() -> Result<(), Error> {
        let source = r#"r: close({"n":int,"child":close({})})"#;
        for document in [
            json!({"r":{"z":0,"a":0}}),
            json!({"r":{"a":0,"z":0,"child":null}}),
        ] {
            assert!(
                matches!(check(source, document), Err(Error::ClosedField { field })
                if field == "r.a")
            );
        }
        for (key, expected) in [
            ("a.b", r#"r["a.b"]"#),
            ("line\n", r#"r["line\n"]"#),
            ("\"", r#"r["\""]"#),
            ("", r#"r[""]"#),
        ] {
            let error = check("r: close({})", json!({"r":{key:0}}))
                .err()
                .ok_or_else(|| Error::Setup("expected closed-field conflict".into()))?;
            assert!(matches!(&error, Error::ClosedField { field } if field == expected));
            assert_eq!(
                error.to_string(),
                format!("vet conflict: field {expected}: field not allowed")
            );
            assert!(!error.to_string().contains('\n'));
        }
        assert!(matches!(check(source, json!({"r":{"child":{"extra":0}}})),
            Err(Error::ClosedField { field }) if field == "r.child.extra"));
        // Structural traversal is depth first in schema order, not JSON order.
        for (source, expected) in [
            ("a: close({\"child\":close({})})\nb: close({})", "a.child"),
            ("b: close({})\na: close({\"child\":close({})})", "b"),
        ] {
            assert!(matches!(
                check(source, json!({"b":null,"a":{"child":null}})),
                Err(Error::RecordType { field }) if field == expected
            ));
        }
        // Structural errors anywhere precede even an earlier scalar conflict or
        // missing leaf. Both declaration orders are intentional pinned controls.
        for source in [
            "first: int\nr: close({})",
            "r: close({})\nfirst: int",
            "first: 1 & 2\nr: close({})",
            "r: close({})\nfirst: 1 & 2",
        ] {
            assert!(matches!(
                check(source, json!({"r":null})),
                Err(Error::RecordType { .. })
            ));
            assert!(matches!(
                check(source, json!({"r":{"extra":0}})),
                Err(Error::ClosedField { .. })
            ));
        }
        // Known scalar container shapes anywhere precede record structure.
        for source in [
            "r: close({})\ns: close({\"n\":int})",
            "s: close({\"n\":int})\nr: close({})",
        ] {
            for record in [Value::Null, json!({"extra":0})] {
                assert!(matches!(
                    check(source, json!({"r":record,"s":{"n":[]}})),
                    Err(Error::UnsupportedData(_))
                ));
            }
        }
        // A mismatched record is not descended into during shape admission.
        assert!(
            matches!(check(source, json!({"r":null})), Err(Error::RecordType { field })
            if field == "r")
        );
        // Scalar semantics retain source-order first failure, not Go's general
        // preference for a conflict over another field's incompleteness.
        assert!(
            matches!(check("r: close({\"missing\":int,\"bad\":1 & 2})", json!({})),
            Err(Error::Evaluation { field, source: RuntimeError::Incomplete })
                if field == "r.missing")
        );
        assert!(
            matches!(check("r: close({\"bad\":1 & 2,\"missing\":int})", json!({})),
            Err(Error::Evaluation { field, source: RuntimeError::Conflict { .. } })
                if field == "r.bad")
        );
        Ok(())
    }

    #[test]
    fn nested_scalar_profile_and_whole_json_controls() -> Result<(), Error> {
        let source = r#"person: close({"age":int,"units":"years"})"#;
        check(source, json!({"person":{"age":42}}))?;
        check(source, json!({"person":{"units":"years","age":42}}))?;
        assert!(matches!(check(source, json!({"person":{"age":false}})),
            Err(Error::Evaluation { field, source: RuntimeError::TypeConflict })
                if field == "person.age"));
        for text in [
            r#"{"person":{"age":42},"extra":{"a":1,"\u0061":2}}"#,
            r#"{"person":{"age":42,"extra":0.5}}"#,
            r#"{"person":{"age":42}} junk"#,
        ] {
            assert!(super::super::json::read(text.as_bytes()).is_err());
        }
        // Record shape admission does not absorb scalar objects/arrays or allow
        // out-of-domain bounds, computed terms, required fields or defaults.
        for source in [
            r#"r: close({"n":int & >9223372036854775807})"#,
            r#"r: close({"n":< -9223372036854775808 & int})"#,
            r#"r: close({"n":1+1})"#,
        ] {
            assert!(matches!(schema(source), Err(Error::UnsupportedSchema(_))));
        }
        for source in [r#"r: close({"n"!:int})"#, r#"r: close({"n":*1 | 2})"#] {
            assert!(matches!(schema(source), Err(Error::InvalidSchema(_))));
        }
        for candidate in [json!({}), json!([])] {
            assert!(matches!(
                check(source, json!({"person":{"age":candidate}})),
                Err(Error::UnsupportedData(_))
            ));
        }
        Ok(())
    }

    #[test]
    fn recursive_whole_admission_and_cumulative_limits() -> Result<(), Error> {
        for source in [
            "early: 1 & 2\nr: close({\"later\":_})",
            "early: int\nr: close({\"later\":>=0})",
        ] {
            assert!(matches!(
                check(source, json!({})),
                Err(Error::UnsupportedSchema(_))
            ));
        }
        // Empty records count even without scalar leaves; root costs one.
        for count in [63, 64] {
            let source = (0..count)
                .map(|i| format!("r{i}: close({{}})"))
                .collect::<Vec<_>>()
                .join("\n");
            if count == 63 {
                check(&source, json!({}))?;
            } else {
                assert!(matches!(schema(&source), Err(Error::InvalidSchema(_))));
            }
            let fields: Vec<_> = (0..count)
                .map(|i| {
                    json!({"name":format!("r{i}"),
                "value":{"kind":"recordExpression","fields":[]}})
                })
                .collect();
            let mut nodes = 0;
            assert_eq!(decode_fields(&fields, &mut nodes, "").is_ok(), count == 63);
            assert_eq!(nodes, count);
        }
        // A shared ledger across sibling records, not a fresh budget per subtree.
        let children: Vec<_> = (0..30)
            .map(|i| {
                json!({"name":format!("n{i}"),
            "value":{"kind":"integer","value":1}})
            })
            .collect();
        let subtree = json!({"kind":"recordExpression","fields":children});
        let mut fields = vec![
            json!({"name":"a","value":subtree}),
            json!({"name":"b","value":subtree}),
        ];
        let mut nodes = 0;
        decode_fields(&fields, &mut nodes, "")?;
        assert_eq!(nodes, 62);
        fields.push(json!({"name":"c","value":{"kind":"recordExpression","fields":[]}}));
        decode_fields(&fields, &mut 0, "")?;
        fields.push(json!({"name":"d","value":{"kind":"recordExpression","fields":[]}}));
        assert!(matches!(
            decode_fields(&fields, &mut 0, ""),
            Err(Error::Setup(_))
        ));
        for depth in [31, 32] {
            let mut term = "1".to_string();
            for _ in 0..depth {
                term = format!("close({{\"n\":{term}}})");
            }
            let source = format!("r: {term}");
            if depth == 31 {
                check(&source, json!({}))?;
            } else {
                assert!(matches!(schema(&source), Err(Error::InvalidSchema(_))));
            }
        }
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
