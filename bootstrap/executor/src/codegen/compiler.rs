use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::to_value;
use snafu::{ResultExt, Snafu};

use crate::{
    lowering::{Driver, Policy},
    module::Frame,
};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("invalid {document} JSON: {source}"))]
    Json {
        document: &'static str,
        source: serde_json::Error,
    },
    #[snafu(display("unsupported {document} formatVersion: {version}"))]
    Version {
        document: &'static str,
        version: u32,
    },
    #[snafu(display("{source}"))]
    Lowering { source: crate::lowering::Error },
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Rules {
    format_version: u32,
    pub(crate) rules: BTreeMap<String, Rule>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Rule {
    pub(crate) result: Instruction,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "op", rename_all = "lowercase", deny_unknown_fields)]
pub(crate) enum Instruction {
    Conjunction {
        field: String,
    },
    Null {},
    Top {},
    Int {},
    Bool {},
    Text {
        field: String,
    },
    List {
        field: String,
    },
    Record {
        field: String,
    },
    #[serde(rename = "recordExpression")]
    RecordExpression {
        field: String,
    },
    #[serde(rename = "recordIndex")]
    RecordIndex {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    #[serde(rename = "recordLength")]
    RecordLength {
        value: Box<Instruction>,
    },
    #[serde(rename = "listExpression")]
    ListExpression {
        field: String,
    },
    #[serde(rename = "stringType")]
    StringType {},
    Boolean {
        value: bool,
    },
    #[serde(rename = "lowerBound")]
    LowerBound {
        value: Box<Instruction>,
    },
    #[serde(rename = "lowerExclusiveBound")]
    LowerExclusiveBound {
        value: Box<Instruction>,
    },
    #[serde(rename = "upperExclusiveBound")]
    UpperExclusiveBound {
        value: Box<Instruction>,
    },
    #[serde(rename = "upperBound")]
    UpperBound {
        value: Box<Instruction>,
    },
    Field {
        field: String,
    },
    Evaluate {
        field: String,
    },
    Negate {
        value: Box<Instruction>,
    },
    Index {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    Slice {
        #[serde(rename = "operandField")]
        operand_field: String,
        #[serde(rename = "lowerField")]
        lower_field: String,
        #[serde(rename = "upperField")]
        upper_field: String,
    },
    #[serde(rename = "listSlice")]
    ListSlice {
        value: Box<Instruction>,
        lower: Box<Instruction>,
        upper: Box<Instruction>,
    },
    #[serde(rename = "listIndex")]
    ListIndex {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    Sum {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    Difference {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    Product {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    Divide {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    Modulo {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    Quotient {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    Remainder {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    Length {
        value: Box<Instruction>,
    },
    #[serde(rename = "stringLength")]
    StringLength {
        value: Box<Instruction>,
    },
    #[serde(rename = "listLength")]
    ListLength {
        value: Box<Instruction>,
    },
    #[serde(rename = "logicalNot")]
    LogicalNot {
        value: Box<Instruction>,
    },
    #[serde(rename = "logicalAnd")]
    LogicalAnd {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    #[serde(rename = "logicalOr")]
    LogicalOr {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    Less {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    #[serde(rename = "lessEqual")]
    LessEqual {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    Greater {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    #[serde(rename = "greaterEqual")]
    GreaterEqual {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    Equal {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    #[serde(rename = "notEqual")]
    NotEqual {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
    Unify {
        left: Box<Instruction>,
        right: Box<Instruction>,
    },
}

pub(crate) fn parse_rules(input: &str) -> Result<Rules, Error> {
    let rules: Rules = serde_json::from_str(input).context(JsonSnafu { document: "rules" })?;
    if rules.format_version != 1 {
        return VersionSnafu {
            document: "rules",
            version: rules.format_version,
        }
        .fail();
    }
    Ok(rules)
}

/// Run CUE-owned module assembly and instruction lowering.
pub fn generate(input: &str) -> Result<String, Error> {
    #[cfg(feature = "compiled-lowering")]
    {
        emit(parse_rules(input)?, &crate::compiled_lowering::Compiled)
    }
    #[cfg(not(feature = "compiled-lowering"))]
    {
        generate_with_policy(
            input,
            include_str!(concat!(env!("CUESTACEAN_BOOTSTRAP_JSON"), "/lowering.json")),
        )
    }
}

/// Override the trusted compiler program, not the input instruction language.
pub fn generate_with_policy(input: &str, policy: &str) -> Result<String, Error> {
    let rules = parse_rules(input)?;
    let policy = Policy::parse(policy).map_err(|source| Error::Lowering { source })?;
    emit(rules, &policy)
}

fn emit(rules: Rules, driver: &dyn Driver) -> Result<String, Error> {
    // Typed validation/serialization and canonical rule storage remain native.
    // Cursor traversal, source layout and expression selection belong to CUE.
    let rules = rules
        .rules
        .into_iter()
        .map(|(kind, rule)| {
            to_value(&rule.result)
                .context(JsonSnafu { document: "rules" })
                .map(|node| (kind, node))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Frame::run(&rules, driver).map_err(|source| Error::Lowering { source })
}
