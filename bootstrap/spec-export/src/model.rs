use std::{collections::BTreeMap, fmt};

pub(crate) type Id = usize;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Type {
    String,
    Int,
}
#[derive(Clone, Debug)]
pub(crate) enum Expr {
    Literal(serde_json::Value),
    Type(Type),
    NotEqual(serde_json::Value),
    Struct(Vec<(String, Id)>),
    List(Vec<Id>, Option<Id>),
    Ref(String),
    And(Id, Id),
    Or(Id, Id),
}
#[derive(Clone, Debug)]
pub(crate) struct Document {
    pub nodes: Vec<Expr>,
    pub definitions: BTreeMap<String, Id>,
    pub roots: Vec<Id>,
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
            Self::Unknown(s) => write!(f, "unknown definition: {s}"),
            Self::Conflict => f.write_str("conflicting constraints"),
            Self::Incomplete => f.write_str("incomplete export demand"),
            Self::Limit(s) => write!(f, "resource limit: {s}"),
        }
    }
}
impl std::error::Error for Error {}
