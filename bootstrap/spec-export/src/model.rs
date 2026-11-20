use std::{collections::BTreeMap, fmt};

pub(crate) type Id = usize;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Type {
    String,
    Int,
    Bool,
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum StringSlot {
    Key,
    Value,
}
#[derive(Clone, Debug)]
pub(crate) enum Template {
    Literal(serde_json::Value),
    Key,
    Value,
    Interpolated {
        prefix: String,
        slot: StringSlot,
        suffix: String,
    },
    Struct(Vec<(String, Template)>),
    List(Vec<Template>),
}
impl Template {
    pub(crate) fn height(&self) -> usize {
        1 + match self {
            Self::Struct(fields) => fields.iter().map(|(_, t)| t.height()).max().unwrap_or(0),
            Self::List(items) => items.iter().map(Self::height).max().unwrap_or(0),
            _ => 0,
        }
    }
}
#[derive(Clone, Debug)]
pub(crate) enum Expr {
    Top,
    // Syntax wrapper; only whole-document admission may authorize selection.
    Marked(Id),
    Literal(serde_json::Value),
    Type(Type),
    NotEqual(serde_json::Value),
    Struct(Vec<(String, Id)>),
    List(Vec<Id>, Option<Id>),
    Ref(String),
    // Parser-only lexical context; admission replaces every Name before evaluation.
    Name {
        scope: usize,
        excluded: Vec<usize>,
        name: String,
    },
    // Direct body arena ID, not an index into the parser's binding table.
    Binding(Id),
    Comprehension {
        source: Id,
        template: Template,
        scope: usize,
        key: String,
        value: Option<String>,
    },
    And(Id, Id),
    Or(Id, Id),
}
#[derive(Clone, Debug)]
pub(crate) struct Document {
    pub nodes: Vec<Expr>,
    pub definitions: BTreeMap<String, Id>,
    pub roots: Vec<Id>,
    // Token-derived dispatch signal, not an admission certificate.
    pub marked: bool,
    // Preserve declarations even when resolution removes their names.
    pub lets: usize,
    pub comprehensions: usize,
    pub syntax: usize,
    pub preparation_steps: usize,
    pub preparation_strings: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Syntax(&'static str),
    Unsupported(&'static str),
    Range,
    Unknown(String),
    Conflict,
    Incomplete,
    Limit(&'static str),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax(s) => write!(f, "syntax: {s}"),
            Self::Unsupported(s) => write!(f, "unsupported: {s}"),
            Self::Range => f.write_str("unsupported integer range"),
            Self::Unknown(s) => write!(f, "unknown reference: {s}"),
            Self::Conflict => f.write_str("conflicting constraints"),
            Self::Incomplete => f.write_str("incomplete export demand"),
            Self::Limit(s) => write!(f, "resource limit: {s}"),
        }
    }
}
impl std::error::Error for Error {}
