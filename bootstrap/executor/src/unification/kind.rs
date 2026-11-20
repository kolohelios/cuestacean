use serde::{Deserialize, Serialize};

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
