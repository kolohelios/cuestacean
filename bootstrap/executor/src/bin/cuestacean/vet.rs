//! Bounded scalar/prefix-list/closed-record vet; scalar execution stays generated.
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::io::Read;
use std::path::Path;

use bootstrap_executor::{Error as RuntimeError, ast, generated, parser};
use serde_json::{Value, json};

const MAX_BYTES: usize = 1024 * 1024;
const MAX_INSTANCE_WORK: usize = 16 * 1024 * 1024;
const GRAMMAR: &str = include_str!(concat!(
    env!("CUESTACEAN_BOOTSTRAP_JSON"),
    "/vet-grammar.json"
));
const USAGE: &str = "usage: cuestacean vet -c schema.cue data.json";

#[derive(Debug)]
pub(super) enum Error {
    Usage,
    Setup(String),
    InvalidSchema(parser::Error),
    UnsupportedSchema(String),
    UnsupportedData(&'static str),
    Json(String),
    Evaluation {
        field: String,
        source: RuntimeError,
    },
    RecordType {
        field: String,
    },
    ListType {
        field: String,
    },
    ListLength {
        field: String,
        expected: usize,
        actual: usize,
    },
    ListMinimum {
        field: String,
        expected: usize,
        actual: usize,
    },
    ClosedField {
        field: String,
    },
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
            Self::ListType { field } => {
                write!(f, "vet conflict: {field}: expected a list")
            }
            Self::ListLength {
                field,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "vet conflict: {field}: incompatible list lengths (expected {expected}, got {actual})"
                )
            }
            Self::ListMinimum {
                field,
                expected,
                actual,
            } => write!(
                f,
                "vet conflict: {field}: incompatible list lengths (expected at least {expected}, got {actual})"
            ),
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
    List {
        elements: Vec<Term>,
        tail: Option<Tail>,
    },
    Closed {
        fields: Vec<Field>,
        names: std::collections::BTreeSet<String>,
    },
}

// Only admitted scalar ASTs can be instantiated. Byte size is measured once,
// not by serializing the tail again for every projected position.
struct Tail {
    scalar: Value,
    nodes: usize,
    bytes: usize,
}

fn identifier(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_alphabetic())
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn index_path(parent: &str, index: usize) -> String {
    format!("{parent}[{index}]")
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
        return Err(Error::Setup("schema exceeds 63 AST nodes".into()));
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

fn admit_scalar(term: &Value, nodes: &mut usize, term_path: &str) -> Result<(), Error> {
    let terms = admit(term, nodes)?;
    if terms.bound && !terms.int {
        return Err(unsupported(format!(
            "field {term_path}: bounds require an explicit int term"
        )));
    }
    Ok(())
}

// Decode the entire schema before inspecting data. Containers under scalar
// unify stay on the scalar admission path and are therefore unsupported.
fn decode_term(term: &Value, nodes: &mut usize, term_path: &str) -> Result<Term, Error> {
    let kind = term.get("kind").and_then(Value::as_str);
    if !matches!(
        kind,
        Some("recordExpression" | "listExpression" | "typedListExpression")
    ) {
        admit_scalar(term, nodes, term_path)?;
        return Ok(Term::Scalar(term.clone()));
    }
    *nodes += 1;
    if *nodes > 63 {
        return Err(Error::Setup("schema exceeds 63 AST nodes".into()));
    }
    let key = if kind == Some("recordExpression") {
        "fields"
    } else {
        "elements"
    };
    let object = term
        .as_object()
        .ok_or_else(|| unsupported("expected container AST"))?;
    let typed = kind == Some("typedListExpression");
    if object.len() != if typed { 3 } else { 2 }
        || !object.contains_key(key)
        || (typed && !object.contains_key("tail"))
    {
        return Err(unsupported("unexpected container AST fields"));
    }
    let children = term
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| unsupported("expected container children"))?;
    if kind == Some("recordExpression") {
        let fields = decode_fields(children, nodes, term_path)?;
        let names = fields.iter().map(|field| field.name.clone()).collect();
        Ok(Term::Closed { fields, names })
    } else {
        let elements = children
            .iter()
            .enumerate()
            .map(|(index, child)| decode_term(child, nodes, &index_path(term_path, index)))
            .collect::<Result<_, _>>()?;
        let tail = if typed {
            let scalar = &term["tail"];
            let before = *nodes;
            // The tail needs its own explicit int, independently of the prefix.
            admit_scalar(scalar, nodes, &format!("{term_path}[tail]"))?;
            let bytes = serde_json::to_vec(scalar)
                .map_err(|error| Error::Setup(format!("cannot serialize vet tail: {error}")))?
                .len();
            Some(Tail {
                scalar: scalar.clone(),
                nodes: *nodes - before,
                bytes,
            })
        } else {
            None
        };
        Ok(Term::List { elements, tail })
    }
}

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
        let term = decode_term(term, nodes, &field_path)?;
        result.push(Field {
            name: name.into(),
            term,
        });
    }
    Ok(result)
}

fn schema(source: &str) -> Result<Vec<Field>, Error> {
    let program = parser::parse_vet_fields_file(source, GRAMMAR).map_err(|error| match error {
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

// Inclusive 16 Mi-unit logical bound, not wall time or total memory. Each extra
// position costs exactly 128 + scalar AST nodes + compact AST bytes + 6*decoded
// candidate string bytes. The factor six bounds JSON escaping; wrong-kind
// scalar candidates still pay the base cost before the shape pass rejects them.
// Reserve only additional tail applications. One ledger follows all schema
// associations, even beyond an earlier semantic failure; unknown fields and
// wrong-kind containers are not descended into. No bridge/evaluator work here.
struct InstanceWork {
    used: usize,
    limit: usize,
}

fn work_limit() -> Error {
    Error::Setup("instance work limit exceeded".into())
}

impl InstanceWork {
    fn tail(&mut self, tail: &Tail, candidate: &Value) -> Result<(), Error> {
        let string_bytes = candidate.as_str().map_or(0, str::len);
        let cost = string_bytes
            .checked_mul(6)
            .and_then(|cost| cost.checked_add(128))
            .and_then(|cost| cost.checked_add(tail.nodes))
            .and_then(|cost| cost.checked_add(tail.bytes))
            .ok_or_else(work_limit)?;
        let used = self.used.checked_add(cost).ok_or_else(work_limit)?;
        if used > self.limit {
            return Err(work_limit());
        }
        self.used = used;
        Ok(())
    }

    fn term(&mut self, term: &Term, candidate: Option<&Value>) -> Result<(), Error> {
        match term {
            Term::Scalar(_) => {}
            Term::Closed { fields, .. } => {
                if let Some(object) = candidate.and_then(Value::as_object) {
                    self.fields(fields, object)?;
                }
            }
            Term::List { elements, tail } => {
                if let Some(array) = candidate.and_then(Value::as_array) {
                    for (term, value) in elements.iter().zip(array) {
                        self.term(term, Some(value))?;
                    }
                    if let Some(tail) = tail {
                        for value in array.iter().skip(elements.len()) {
                            self.tail(tail, value)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn fields(
        &mut self,
        fields: &[Field],
        data: &serde_json::Map<String, Value>,
    ) -> Result<(), Error> {
        for field in fields {
            self.term(&field.term, data.get(&field.name))?;
        }
        Ok(())
    }
}

// After reservation: known-scalar shapes, container structure, scalar execution.
// Missing containers pass None down, never synthetic null or empty runtime values.
fn term_shapes(term: &Term, candidate: Option<&Value>) -> Result<(), Error> {
    match term {
        Term::Scalar(_) => {
            if candidate.is_some_and(|value| value.is_array() || value.is_object()) {
                return Err(Error::UnsupportedData("known fields require scalar values"));
            }
        }
        Term::Closed { fields, .. } => {
            if let Some(object) = candidate.and_then(Value::as_object) {
                shapes(fields, Some(object))?;
            }
        }
        Term::List { elements, tail } => {
            // Check paired and typed trailing positions even on wrong lengths.
            // Wrong-kind containers are deferred to structure, as for records.
            if let Some(array) = candidate.and_then(Value::as_array) {
                for (term, value) in elements.iter().zip(array) {
                    term_shapes(term, Some(value))?;
                }
                if tail.is_some()
                    && array
                        .iter()
                        .skip(elements.len())
                        .any(|value| value.is_array() || value.is_object())
                {
                    return Err(Error::UnsupportedData("known fields require scalar values"));
                }
            }
        }
    }
    Ok(())
}

fn shapes(fields: &[Field], data: Option<&serde_json::Map<String, Value>>) -> Result<(), Error> {
    for field in fields {
        let candidate = data.and_then(|data| data.get(&field.name));
        term_shapes(&field.term, candidate)?;
    }
    Ok(())
}

fn structure(
    fields: &[Field],
    data: Option<&serde_json::Map<String, Value>>,
    parent: &str,
) -> Result<(), Error> {
    for field in fields {
        term_structure(
            &field.term,
            data.and_then(|data| data.get(&field.name)),
            &path(parent, &field.name),
        )?;
    }
    Ok(())
}

fn term_structure(term: &Term, candidate: Option<&Value>, term_path: &str) -> Result<(), Error> {
    match term {
        Term::Scalar(_) => {}
        Term::Closed { fields, names } => {
            let object = match candidate {
                None => None,
                Some(Value::Object(object)) => Some(object),
                Some(_) => {
                    return Err(Error::RecordType {
                        field: term_path.into(),
                    });
                }
            };
            if let Some(object) = object {
                // Explicit min keeps ordering independent of serde map features.
                if let Some(extra) = object.keys().filter(|key| !names.contains(*key)).min() {
                    return Err(Error::ClosedField {
                        field: path(term_path, extra),
                    });
                }
            }
            structure(fields, object, term_path)?;
        }
        Term::List { elements, tail } => {
            let array = match candidate {
                None => None,
                Some(Value::Array(array)) => {
                    if tail.is_some() && array.len() < elements.len() {
                        return Err(Error::ListMinimum {
                            field: term_path.into(),
                            expected: elements.len(),
                            actual: array.len(),
                        });
                    }
                    if tail.is_none() && array.len() != elements.len() {
                        return Err(Error::ListLength {
                            field: term_path.into(),
                            expected: elements.len(),
                            actual: array.len(),
                        });
                    }
                    Some(array)
                }
                Some(_) => {
                    return Err(Error::ListType {
                        field: term_path.into(),
                    });
                }
            };
            for (index, term) in elements.iter().enumerate() {
                term_structure(
                    term,
                    array.and_then(|array| array.get(index)),
                    &index_path(term_path, index),
                )?;
            }
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
        term_leaves(&field.term, candidate, &field_path)?;
    }
    Ok(())
}

fn term_leaves(term: &Term, candidate: Option<&Value>, term_path: &str) -> Result<(), Error> {
    match term {
        Term::Closed { fields, .. } => {
            leaves(fields, candidate.and_then(Value::as_object), term_path)?;
        }
        Term::List { elements, tail } => {
            let array = candidate.and_then(Value::as_array);
            for (index, term) in elements.iter().enumerate() {
                term_leaves(
                    term,
                    array.and_then(|array| array.get(index)),
                    &index_path(term_path, index),
                )?;
            }
            if let (Some(tail), Some(array)) = (tail, array) {
                for (index, candidate) in array.iter().enumerate().skip(elements.len()) {
                    scalar_leaf(&tail.scalar, Some(candidate), &index_path(term_path, index))?;
                }
            }
        }
        Term::Scalar(term) => scalar_leaf(term, candidate, term_path)?,
    }
    Ok(())
}

fn scalar_leaf(term: &Value, candidate: Option<&Value>, term_path: &str) -> Result<(), Error> {
    let program = bridge(term, candidate)?;
    let input = serde_json::to_string(&program)
        .map_err(|error| Error::Setup(format!("cannot serialize vet program: {error}")))?;
    generated::execute_json(&input).map_err(|source| Error::Evaluation {
        field: term_path.into(),
        source,
    })?;
    Ok(())
}

fn validate(fields: &[Field], data: &Value) -> Result<(), Error> {
    let data = data
        .as_object()
        .ok_or(Error::UnsupportedData("object root required"))?;
    InstanceWork {
        used: 0,
        limit: MAX_INSTANCE_WORK,
    }
    .fields(fields, data)?;
    shapes(fields, Some(data))?;
    structure(fields, Some(data), "")?;
    // S scalar AST nodes + R records + A lists <=63; L scalar leaves <=S. Each supplied
    // scalar adds two visits, so ordinary generated visits S+2L<=189. Additional
    // tail applications use the shared byte-sensitive reservation above. No retries,
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
            "x: [1] & [1]",
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
            r#"r: close({"x":[1] & [1]})"#,
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
    fn fixed_list_ast_and_recursive_admission() -> Result<(), Error> {
        let source = r#"x: [close({"n":[int]}),[]]"#;
        assert_eq!(
            parser::parse_fields_file(source, GRAMMAR).map_err(Error::InvalidSchema)?,
            json!({"formatVersion":1,"entry":{"kind":"recordExpression","fields":[
                {"name":"x","value":{"kind":"listExpression","elements":[
                    {"kind":"recordExpression","fields":[{"name":"n","value":{
                        "kind":"listExpression","elements":[{"kind":"intType"}]}}]},
                    {"kind":"listExpression","elements":[]}
                ]}}
            ]}})
        );
        check("x: [1]", json!({}))?;
        check(r#"r: close({"x":[1]})"#, json!({}))?;
        for term in [
            "[1] & [1]",
            "int & [1]",
            "[int,>=0]",
            "[_,1]",
            "[1+1]",
            "[>=(0)]",
            "[len(\"a\")]",
            "[int & close({})]",
            "[close({\"n\":int & >9223372036854775807})]",
        ] {
            let source = format!("early: 1 & 2\nx: {term}");
            assert!(
                matches!(schema(&source), Err(Error::UnsupportedSchema(_))),
                "{source}"
            );
        }
        for node in [
            json!({"kind":"listExpression"}),
            json!({"kind":"listExpression","elements":null}),
            json!({"kind":"listExpression","elements":[],"extra":0}),
            json!({"kind":"listExpression","elements":[{"kind":"unknown"}]}),
            json!({"kind":"listExpression","elements":[{"kind":"intType","extra":0}]}),
            json!({"kind":"listExpression","elements":[{"kind":"recordExpression","fields":[
                {"name":"n","value":{"kind":"intType"},"extra":0}]}]}),
        ] {
            assert!(matches!(
                decode_term(&node, &mut 0, "x"),
                Err(Error::UnsupportedSchema(_))
            ));
        }
        Ok(())
    }

    #[test]
    fn fixed_list_absence_lengths_and_generated_scalars() -> Result<(), Error> {
        for (source, data) in [
            ("x: []", json!({})),
            ("x: []", json!({"x":[]})),
            (r#"x: [42,"kg"]"#, json!({})),
            (r#"x: [int,"kg"]"#, json!({"x":[1,"kg"]})),
            (r#"x: [[42],close({"a":[],"n":null})]"#, json!({})),
            (
                r#"x: [[42],close({"a":[],"n":null})]"#,
                json!({"x":[[42],{}]}),
            ),
            (r#"x: [close({"n":int}),[42]]"#, json!({"x":[{"n":1},[42]]})),
            (
                "x: [null,true,\"a🦀\",-9223372036854775808,9223372036854775807,9007199254740993]",
                json!({}),
            ),
        ] {
            check(source, data)?;
        }
        for (source, data, expected) in [
            (r#"x: [int,"kg"]"#, json!({}), "x[0]"),
            (r#"x: [close({"n":int}),[42]]"#, json!({}), "x[0].n"),
            (
                r#"x: [close({"n":int}),[42]]"#,
                json!({"x":[{},[42]]}),
                "x[0].n",
            ),
            (
                r#"x: close({"items":[close({"n":int})]})"#,
                json!({"x":{}}),
                "x.items[0].n",
            ),
            ("x: [int,1 & 2]", json!({}), "x[0]"),
        ] {
            assert!(matches!(check(source, data), Err(Error::Evaluation {
                field, source: RuntimeError::Incomplete }) if field == expected));
        }
        for (source, data, expected, actual, location) in [
            (r#"x: [int,"kg"]"#, json!({"x":[]}), 2, 0, "x"),
            (r#"x: [42,"kg"]"#, json!({"x":[42]}), 2, 1, "x"),
            (r#"x: [int,"kg"]"#, json!({"x":[1,"kg",0]}), 2, 3, "x"),
            ("x: []", json!({"x":[1]}), 0, 1, "x"),
            ("x: [int,1 & 2]", json!({"x":[0]}), 2, 1, "x"),
            ("x: [int,int]", json!({"x":[false]}), 2, 1, "x"),
            ("x: [int,int]", json!({"x":[false,0,1]}), 2, 3, "x"),
            (
                r#"x: [close({"n":int}),[42]]"#,
                json!({"x":[{"n":1},[]]}),
                1,
                0,
                "x[1]",
            ),
        ] {
            let error = check(source, data)
                .err()
                .ok_or_else(|| Error::Setup("expected length".into()))?;
            assert!(
                matches!(&error, Error::ListLength { field, expected: n, actual: m }
                if field == location && *n == expected && *m == actual)
            );
            assert_eq!(
                error.to_string(),
                format!(
                    "vet conflict: {location}: incompatible list lengths (expected {expected}, got {actual})"
                )
            );
        }
        for value in [Value::Null, json!({}), json!(true), json!(0), json!("s")] {
            let error = check("x: [int]", json!({"x":value}))
                .err()
                .ok_or_else(|| Error::Setup("expected list type".into()))?;
            assert!(matches!(&error, Error::ListType { field } if field == "x"));
            assert_eq!(error.to_string(), "vet conflict: x: expected a list");
        }
        assert!(matches!(check("x: [[int]]", json!({"x":[null]})),
            Err(Error::ListType { field }) if field == "x[0]"));
        assert!(matches!(check("x: [close({})]", json!({"x":[null]})),
            Err(Error::RecordType { field }) if field == "x[0]"));
        for (source, data, expected) in [
            (r#"x: [int,"kg"]"#, json!({"x":[false,"kg"]}), "x[0]"),
            ("x: [[int]]", json!({"x":[[null]]}), "x[0][0]"),
        ] {
            assert!(matches!(check(source, data), Err(Error::Evaluation {
                field, source: RuntimeError::TypeConflict }) if field == expected));
        }
        for (term, expected) in [("[int,1 & 2]", "x[1]"), ("[1 & 2,int]", "x[0]")] {
            assert!(matches!(check(&format!("x: {term}"), json!({"x":[0,1]})),
                Err(Error::Evaluation { field, source: RuntimeError::Conflict { .. } }) if field == expected));
        }
        for term in ["int & >=0 & <10", "<10 & >=0 & int"] {
            let source = format!("x: [{term}]");
            for value in [0, 9] {
                check(&source, json!({"x":[value]}))?;
            }
            for value in [-1, 10] {
                assert!(matches!(check(&source, json!({"x":[value]})),
                    Err(Error::Evaluation { field, .. }) if field == "x[0]"));
            }
        }
        assert!(matches!(
            check("x: [int & >=42 & <=42]", json!({})),
            Err(Error::Evaluation {
                source: RuntimeError::IncompleteBounds,
                ..
            })
        ));
        check("x: [int & >=42 & <=42]", json!({"x":[42]}))?;
        Ok(())
    }

    #[test]
    fn fixed_list_shapes_structure_order_and_paths() -> Result<(), Error> {
        for (source, data) in [
            ("x: [int,int]", json!({"x":[{}]})),
            ("x: [int,int]", json!({"x":[1,{},0]})),
            (r#"x: [close({"n":int}),int]"#, json!({"x":[{"n":[]}]})),
            ("x: [[int],int]", json!({"x":[[{}]]})),
        ] {
            assert!(matches!(
                check(source, data),
                Err(Error::UnsupportedData(_))
            ));
        }
        for source in ["x: [int]\ny: int", "y: int\nx: [int]"] {
            assert!(matches!(
                check(source, json!({"x":[],"y":{}})),
                Err(Error::UnsupportedData(_))
            ));
            assert!(matches!(
                check(source, json!({"x":null,"y":[]})),
                Err(Error::UnsupportedData(_))
            ));
        }
        for (source, data) in [
            ("x: [int]", json!({"x":[1,{}]})),
            ("x: []", json!({"x":[{}]})),
            ("x: [close({})]", json!({"x":[null,0]})),
        ] {
            assert!(
                matches!(check(source, data), Err(Error::ListLength { field, .. }) if field == "x")
            );
        }
        for first in ["int", "1 & 2"] {
            for source in [
                format!("first: {first}\nx: [int]"),
                format!("x: [int]\nfirst: {first}"),
            ] {
                assert!(matches!(
                    check(&source, json!({"x":[]})),
                    Err(Error::ListLength { .. })
                ));
                assert!(matches!(
                    check(&source, json!({"x":null})),
                    Err(Error::ListType { .. })
                ));
            }
        }
        for (source, expected_list) in [
            (r#"x: close({"a":[],"b":close({})})"#, true),
            (r#"x: close({"b":close({}),"a":[]})"#, false),
        ] {
            let error = check(source, json!({"x":{"a":[0],"b":{"extra":0}}}));
            if expected_list {
                assert!(matches!(error, Err(Error::ListLength { field, .. }) if field == "x.a"));
            } else {
                assert!(matches!(error, Err(Error::ClosedField { field }) if field == "x.b.extra"));
            }
        }
        for key in ["a.b", "", "\"", "line\n", "é"] {
            let error = check("x: [close({})]", json!({"x":[{key:0}]}))
                .err()
                .ok_or_else(|| Error::Setup("expected closure".into()))?;
            let expected = format!("x[0][{}]", json!(key));
            assert!(matches!(&error, Error::ClosedField { field } if field == &expected));
            assert!(!error.to_string().contains('\n'));
        }
        assert!(matches!(check(r#"x: [close({"n":int}),[42]]"#,
            json!({"x":[{"n":1,"extra":0},[42]]})),
            Err(Error::ClosedField { field }) if field == "x[0].extra"));
        // Full JSON admission cannot be hidden in an unmatched array position.
        for text in [r#"{"x":[1,{"a":0,"\u0061":1}]}"#, r#"{"x":[1,0.5]}"#] {
            assert!(super::super::json::read(text.as_bytes()).is_err());
        }
        Ok(())
    }

    #[test]
    fn fixed_list_nested_negative_controls() -> Result<(), Error> {
        for term in [
            "[ref]",
            "[*1 | 2]",
            "[{n:1}]",
            "[close({\"n\"?:int})]",
            "[close({\"n\"!:int})]",
        ] {
            assert!(matches!(
                schema(&format!("x: {term}")),
                Err(Error::InvalidSchema(_))
            ));
        }
        for term in [r#"[close({"n":0,"\u006e":0})]"#, r#"[close({"a.b":0})]"#] {
            assert!(matches!(
                schema(&format!("x: {term}")),
                Err(Error::UnsupportedSchema(_))
            ));
        }
        let source = r#"x: close({"items":[close({"n":int})]})"#;
        for data in [json!({}), json!({"x":{}}), json!({"x":{"items":[{}]}})] {
            assert!(matches!(check(source, data), Err(Error::Evaluation {
                field, source: RuntimeError::Incomplete }) if field == "x.items[0].n"));
        }
        check(source, json!({"x":{"items":[{"n":42}]}}))?;
        let source =
            "x: [null,true,\"a🦀\",-9223372036854775808,9223372036854775807,9007199254740993]";
        check(
            source,
            json!({"x":[null,true,"a🦀",i64::MIN,i64::MAX,9007199254740993_i64]}),
        )?;
        for document in [json!({"x":[false,false]}), json!({"x":[null,null]})] {
            assert!(
                matches!(check("x: [int,int]", document), Err(Error::Evaluation {
                field, source: RuntimeError::TypeConflict }) if field == "x[0]")
            );
        }
        assert!(matches!(check("x: [[int]]", json!({"x":[[]]})),
            Err(Error::ListLength { field, expected: 1, actual: 0 }) if field == "x[0]"));
        assert!(
            matches!(check(r#"x: close({"items":[[int]]})"#, json!({"x":{"items":[[null]]}})),
            Err(Error::Evaluation { field, .. }) if field == "x.items[0][0]")
        );
        Ok(())
    }

    #[test]
    fn fixed_list_shared_nodes_and_mixed_depth() -> Result<(), Error> {
        for count in [61, 62, 63] {
            let source = format!("x: [{}]", vec!["0"; count].join(","));
            if count <= 62 {
                check(&source, json!({}))?;
            } else {
                assert!(matches!(schema(&source), Err(Error::InvalidSchema(_))));
            }
        }
        for count in [63, 64] {
            let fields: Vec<_> = (0..count)
                .map(|i| {
                    json!({"name":format!("f{i}"),
                "value":{"kind":"listExpression","elements":[]}})
                })
                .collect();
            let mut nodes = 0;
            assert_eq!(decode_fields(&fields, &mut nodes, "").is_ok(), count == 63);
            assert_eq!(nodes, count);
            let source = (0..count)
                .map(|i| format!("f{i}: []"))
                .collect::<Vec<_>>()
                .join("\n");
            assert_eq!(schema(&source).is_ok(), count == 63);
        }
        for scalar_count in [60, 61, 62] {
            let source = format!("a: [{}]\nb: close({{}})", vec!["0"; scalar_count].join(","));
            assert_eq!(schema(&source).is_ok(), scalar_count <= 61);
            let children = vec![json!({"kind":"integer","value":0}); scalar_count];
            let fields = vec![
                json!({"name":"a","value":{"kind":"listExpression","elements":children}}),
                json!({"name":"b","value":{"kind":"recordExpression","fields":[]}}),
            ];
            let mut nodes = 0;
            assert_eq!(
                decode_fields(&fields, &mut nodes, "").is_ok(),
                scalar_count <= 61
            );
            assert_eq!(nodes, scalar_count + 2);
        }
        for depth in [31, 32, 33] {
            for mixed in [false, true] {
                let mut term = "[]".to_owned();
                for level in 1..depth {
                    term = if mixed && level % 2 == 1 {
                        format!("close({{\"n\":{term}}})")
                    } else {
                        format!("[{term}]")
                    };
                }
                assert_eq!(schema(&format!("x: {term}")).is_ok(), depth <= 32);
            }
        }
        Ok(())
    }

    #[test]
    fn typed_tail_ast_and_whole_admission() -> Result<(), Error> {
        assert_eq!(
            parser::parse_vet_fields_file("x: [42,...int]", GRAMMAR)
                .map_err(Error::InvalidSchema)?,
            json!({"formatVersion":1,"entry":{"kind":"recordExpression","fields":[
                {"name":"x","value":{"kind":"typedListExpression",
                    "elements":[{"kind":"integer","value":42}],"tail":{"kind":"intType"}}}
            ]}})
        );
        // Admission is independent of demand, including contradictory prefixes.
        for tail in [
            "_",
            ">=0",
            "1+1",
            "int & [1]",
            "[]",
            "close({})",
            "[...int]",
            "int & >9223372036854775807",
            "< -9223372036854775808 & int",
        ] {
            for prefix in ["", "int,", "1 & 2,"] {
                let source = format!("x: [{prefix}...({tail})]");
                for data in [json!({}), json!({"x":[]})] {
                    assert!(
                        matches!(check(&source, data), Err(Error::UnsupportedSchema(_))),
                        "{source}"
                    );
                }
            }
        }
        for source in [
            "x: [...]",
            "x: [...int,]",
            "x: [...int,1]",
            "x: [...ref]",
            "x: [...*1 | 2]",
            "[...int]",
        ] {
            assert!(
                matches!(schema(source), Err(Error::InvalidSchema(_))),
                "{source}"
            );
        }
        // Prefix int cannot qualify an untyped bound tail (or vice versa).
        for source in ["x: [int,...>=0]", "x: [>=0,...int]"] {
            assert!(matches!(schema(source), Err(Error::UnsupportedSchema(_))));
        }
        for node in [
            json!({"kind":"typedListExpression","elements":[]}),
            json!({"kind":"typedListExpression","elements":[],"tail":null}),
            json!({"kind":"typedListExpression","elements":null,"tail":{"kind":"intType"}}),
            json!({"kind":"typedListExpression","elements":[],"tail":{"kind":"intType"},"extra":0}),
            json!({"kind":"typedListExpression","elements":[],"tail":{"kind":"intType","extra":0}}),
            json!({"kind":"typedListExpression","elements":[],"tail":{"kind":"unify","left":{"kind":"intType"}}}),
            json!({"kind":"typedListExpression","elements":[],"tail":{"kind":"listExpression","elements":[]}}),
            json!({"kind":"listExpression","elements":[],"tail":{"kind":"intType"}}),
        ] {
            assert!(matches!(
                decode_term(&node, &mut 0, "x"),
                Err(Error::UnsupportedSchema(_))
            ));
        }
        // Both container and whole scalar tail count toward the original cap.
        let tail = json!({"kind":"unify","left":{"kind":"intType"},"right":{"kind":"lowerBound","value":0}});
        for count in [59, 60] {
            let node = json!({"kind":"typedListExpression",
                "elements":vec![json!({"kind":"integer","value":0});count],"tail":tail});
            let mut nodes = 0;
            assert_eq!(decode_term(&node, &mut nodes, "x").is_ok(), count == 59);
            assert_eq!(nodes, count + 4);
        }
        Ok(())
    }

    #[test]
    fn typed_tail_absence_prefix_null_and_paths() -> Result<(), Error> {
        for (source, data) in [
            ("x: [...int]", json!({})),
            ("x: [...int]", json!({"x":[]})),
            ("x: [42,...int]", json!({})),
            ("x: [42,...int]", json!({"x":[42]})),
            ("x: [42,...int]", json!({"x":[42,1,2]})),
            ("x: [...(1 & 2)]", json!({"x":[]})),
            ("x: [42,...(int & string)]", json!({"x":[42]})),
            ("x: [...null]", json!({"x":[null,null]})),
            ("x: [...bool]", json!({"x":[true,false]})),
            ("x: [...string]", json!({"x":["é","\n"]})),
            (
                "x: [...int]",
                json!({"x":[i64::MIN,i64::MAX,9007199254740993_i64]}),
            ),
            (
                r#"x: [close({"a":[42,...int]}),[...string],...bool]"#,
                json!({"x":[{},[],true,false]}),
            ),
        ] {
            check(source, data)?;
        }
        for source in ["x: [...(1 & 2)]", "x: [42,...(int & string)]"] {
            check(source, json!({}))?;
            let data = if source.contains("42") {
                json!({"x":[42,0]})
            } else {
                json!({"x":[0]})
            };
            assert!(matches!(check(source, data), Err(Error::Evaluation { .. })));
        }
        for (source, data, expected) in [
            ("x: [int,...bool]", json!({}), "x[0]"),
            ("x: [1 & 2,...bool]", json!({}), "x[0]"),
            ("x: [42,...int]", json!({"x":[42,1,null]}), "x[2]"),
            (
                r#"r: close({"items":[[42,...int]]})"#,
                json!({"r":{"items":[[42,false]]}}),
                "r.items[0][1]",
            ),
        ] {
            assert!(
                matches!(check(source, data), Err(Error::Evaluation { field, .. }) if field == expected)
            );
        }
        assert!(
            matches!(check("x: [...int]", json!({"x":null})), Err(Error::ListType { field }) if field == "x")
        );
        let error = check("x: [int,true,...bool]", json!({"x":[false]})).expect_err("short prefix");
        assert!(matches!(
            &error,
            Error::ListMinimum {
                expected: 2,
                actual: 1,
                ..
            }
        ));
        assert_eq!(
            error.to_string(),
            "vet conflict: x: incompatible list lengths (expected at least 2, got 1)"
        );
        for term in ["int & >=0 & <10", "<10 & >=0 & int"] {
            let source = format!("x: [42,...({term})]");
            check(&source, json!({"x":[42,0,9]}))?;
            assert!(
                matches!(check(&source, json!({"x":[42,0,10]})), Err(Error::Evaluation { field, .. }) if field == "x[2]")
            );
        }
        // Merely singleton bounds remain absent-tail lazy, unlike prefix bounds.
        check("x: [...(int & >=42 & <=42)]", json!({}))?;
        check("x: [...(int & >=42 & <=42)]", json!({"x":[42]}))?;
        Ok(())
    }

    #[test]
    fn typed_tail_shapes_before_structure_and_semantics() {
        for (source, data) in [
            ("x: [int,int,...bool]", json!({"x":[{}]})),
            ("x: [int,...bool]", json!({"x":[1,{}]})),
            ("x: [int,...bool]", json!({"x":[false,true,[]]})),
            ("first: 1 & 2\nx: [...int]", json!({"x":[0,{}]})),
            ("r: close({})\nx: [...int]", json!({"r":null,"x":[[]]})),
            (
                r#"x: [close({"n":[...int]}),...bool]"#,
                json!({"x":[{"n":[{}]}]}),
            ),
        ] {
            assert!(matches!(
                check(source, data),
                Err(Error::UnsupportedData(_))
            ));
        }
        // Wrong-kind containers are not traversed; fixed extras stay unpaired.
        assert!(
            matches!(check("x: [[...int],...bool]", json!({"x":[null]})), Err(Error::ListType { field }) if field == "x[0]")
        );
        assert!(matches!(
            check("x: []", json!({"x":[{}]})),
            Err(Error::ListLength { .. })
        ));
        assert!(matches!(
            check("x: [1,...bool]", json!({"x":[]})),
            Err(Error::ListMinimum { .. })
        ));
    }

    fn planned(fields: &[Field], data: &Value, limit: usize) -> Result<usize, Error> {
        let mut work = InstanceWork { used: 0, limit };
        work.fields(fields, data.as_object().expect("test object"))?;
        Ok(work.used)
    }

    #[test]
    fn tail_work_exact_bytes_and_checked_arithmetic() -> Result<(), Error> {
        // Independent compact byte lengths, not the production size calculation.
        let fields = schema(r#"x: [..."é\n"]"#)?;
        let Term::List {
            tail: Some(tail), ..
        } = &fields[0].term
        else {
            panic!("tail")
        };
        assert_eq!(tail.nodes, 1);
        assert_eq!(tail.bytes, 32); // {"kind":"string","value":"é\n"}
        // Decoded é + newline + quote = 4 UTF-8 bytes, not source/JSON length.
        let data = json!({"x":["é\n\""]});
        assert_eq!(planned(&fields, &data, 185)?, 185); // 128+1+32+6*4
        assert_eq!(
            planned(&fields, &data, 184)
                .expect_err("over exact allowance")
                .to_string(),
            "vet setup: instance work limit exceeded"
        );
        assert_eq!(planned(&fields, &json!({"x":[{}]}), 161)?, 161);
        // A multi-node tail charges every admitted AST node, not just its root.
        let conjunction = schema("x: [...(int & >=0)]")?;
        let Term::List {
            tail: Some(composite),
            ..
        } = &conjunction[0].term
        else {
            panic!("tail")
        };
        assert_eq!((composite.nodes, composite.bytes), (3, 82));
        assert_eq!(planned(&conjunction, &json!({"x":[0]}), 213)?, 213);
        assert!(planned(&conjunction, &json!({"x":[0]}), 212).is_err());
        let mut work = InstanceWork {
            used: usize::MAX,
            limit: usize::MAX,
        };
        assert!(work.tail(tail, &Value::Null).is_err());
        let huge = Tail {
            scalar: json!({"kind":"null"}),
            nodes: 1,
            bytes: usize::MAX,
        };
        assert!(
            InstanceWork {
                used: 0,
                limit: usize::MAX
            }
            .tail(&huge, &Value::Null)
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn tail_work_associations_and_zero_demand() -> Result<(), Error> {
        let fields = schema(
            r#"a: [close({"x":[42,...int]}),...bool]
b: close({"y":[...string]})"#,
        )?;
        // intType: 18 bytes; boolType: 19; stringType: 21.
        let data = json!({"a":[{"x":[42,1,2]},true],"b":{"y":["é"]},"unknown":[1,2]});
        assert_eq!(planned(&fields, &data, 604)?, 2 * 147 + 148 + 162);
        assert!(planned(&fields, &data, 603).is_err());
        for data in [
            json!({}),
            json!({"a":[{"x":[42]}],"b":{"y":[]}}),
            json!({"a":null,"b":[]}),
            json!({"a":[null],"b":{"y":null}}),
        ] {
            assert_eq!(planned(&fields, &data, 0)?, 0);
        }
        let fixed = schema("x: [int]\nr: close({})")?;
        assert_eq!(
            planned(&fixed, &json!({"x":[1,"large",{}],"r":{"extra":[]}}), 0)?,
            0
        );
        // Short prefix does not conceal nested reached tail applications.
        let fields = schema("x: [[...int],true,...bool]")?;
        assert_eq!(planned(&fields, &json!({"x":[[0,1]]}), 294)?, 294);
        Ok(())
    }

    // Exercise the same DATA reader as run(), not merely serde_json's decoder.
    fn admitted_data(data: Value) -> Value {
        let input = serde_json::to_vec(&data).expect("serialize test DATA");
        super::super::json::read(input.as_slice()).expect("fully admitted DATA")
    }

    #[test]
    fn large_tail_real_allowance_and_phase_order() -> Result<(), Error> {
        let literal = "a".repeat(16_384);
        let source = format!("a: [...{literal:?}]\nb: [...{literal:?}]");
        let fields = schema(&schema_text(source.as_bytes())?)?;
        let Term::List {
            tail: Some(tail), ..
        } = &fields[0].term
        else {
            panic!("tail")
        };
        assert_eq!((tail.nodes, tail.bytes), (1, 16_412));
        // Independent issue arithmetic: each "z" costs 16,547 units.
        let below = admitted_data(json!({"a":vec!["z";1_013]}));
        assert_eq!(planned(&fields, &below, MAX_INSTANCE_WORK)?, 16_762_111);
        assert!(matches!(validate(&fields, &below), Err(Error::Evaluation {
            field, source: RuntimeError::StringConflict { .. } }) if field == "a[0]"));
        // Both sibling lists individually fit, but their shared sum does not.
        let over = admitted_data(json!({"a":vec!["z";507],"b":vec!["z";507]}));
        assert_eq!(planned(&fields, &over, usize::MAX)?, 16_778_658);
        assert_eq!(
            validate(&fields, &over)
                .expect_err("shared sibling allowance")
                .to_string(),
            "vet setup: instance work limit exceeded"
        );
        // Genuine equal literal execution: no hidden scanner/runtime byte cap.
        validate(&fields, &admitted_data(json!({"a":[literal]})))?;
        // Exact production cap: 1,010*(128+1+16,412) + 6*11,801.
        let mut candidates = vec![String::new(); 1_010];
        candidates[1_009] = "z".repeat(11_801);
        let exact = admitted_data(json!({"a":candidates}));
        assert_eq!(planned(&fields, &exact, MAX_INSTANCE_WORK)?, 16_777_216);
        assert!(matches!(
            validate(&fields, &exact),
            Err(Error::Evaluation {
                source: RuntimeError::StringConflict { .. },
                ..
            })
        ));
        // One decoded byte over fails before the first conflict.
        let mut over = exact;
        over["a"][1_009] = json!("z".repeat(11_802));
        assert_eq!(
            validate(&fields, &over)
                .expect_err("one byte over cap")
                .to_string(),
            "vet setup: instance work limit exceeded"
        );
        // Failures do not leak the ledger into a fresh invocation, even when
        // that invocation demands a real (successful) scalar tail evaluation.
        validate(&fields, &admitted_data(json!({"a":["a".repeat(16_384)]})))?;
        validate(&fields, &json!({"a":[]}))?;
        check("x: [...int]", json!({"x":[42]}))?;
        // Reservation beats earlier semantic, shape and structural failures.
        for first in ["first: int", "first: 1 & 2", "first: close({})"] {
            let fields = schema(&format!("{first}\n{source}"))?;
            let data = admitted_data(json!({"first":null,"a":vec!["z";1_014]}));
            assert_eq!(
                validate(&fields, &data)
                    .expect_err("reserve before earlier failure")
                    .to_string(),
                "vet setup: instance work limit exceeded"
            );
        }
        let mut wrong_shape = over;
        wrong_shape["a"][0] = json!({}); // wrong-kind candidates still pay the base
        assert_eq!(
            validate(&fields, &wrong_shape)
                .expect_err("reserve before shape failure")
                .to_string(),
            "vet setup: instance work limit exceeded"
        );
        // Root admission precedes planning.
        assert!(matches!(
            validate(&fields, &json!([])),
            Err(Error::UnsupportedData("object root required"))
        ));
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
