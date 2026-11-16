use serde_json::to_value;
use snafu::ResultExt;

use crate::{
    Error, JsonSnafu, Rules,
    lowering::{Driver, Policy},
    module::Frame,
    parse_rules,
};

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
