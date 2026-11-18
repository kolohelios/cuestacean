// Test-only executable dispatch over checked predecision facts, not a solver.
use super::{Error, Work};
use serde::Deserialize;

const MAX_BYTES: usize = 16 * 1024;
const MAX_NODES: usize = 64;
const MAX_DEPTH: usize = 16;

pub struct Policy {
    body: Decision,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Program {
    #[serde(rename = "formatVersion")]
    format_version: u64,
    body: Decision,
}

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "camelCase", deny_unknown_fields)]
enum Decision {
    Return {
        action: Action,
    },
    If {
        field: Field,
        then: Box<Decision>,
        #[serde(rename = "else")]
        otherwise: Box<Decision>,
    },
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) enum Action {
    Constraints,
    Flat,
    Defer,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
enum Field {
    Empty,
    Forced,
    Recipe,
    Nested,
}

// Constructed only from the graph's complete bounded observation, never from
// a reference action/readiness or caller-supplied policy input.
pub(super) struct Facts {
    pub empty: bool,
    pub forced: bool,
    pub recipe: bool,
    pub nested: bool,
}

impl Policy {
    pub fn from_json(text: &str) -> Result<Self, Error> {
        if text.len() > MAX_BYTES {
            return Err(Error::Limit("policy bytes"));
        }
        // Original-text typed decoding rejects duplicates, unknown/dead arms,
        // and trailing input. Retain serde's ordinary recursion guard. Logical
        // tree limits below apply after decoding, not before parser allocation.
        let program: Program = serde_json::from_str(text)
            .map_err(|_| Error::Unsupported("invalid demand policy JSON"))?;
        if program.format_version != 1 {
            return Err(Error::Unsupported("demand policy version"));
        }
        let mut pending = vec![(&program.body, 1)];
        let mut nodes = 0;
        while let Some((decision, depth)) = pending.pop() {
            nodes += 1;
            if nodes > MAX_NODES {
                return Err(Error::Limit("policy nodes"));
            }
            if depth > MAX_DEPTH {
                return Err(Error::Limit("policy depth"));
            }
            if let Decision::If {
                then, otherwise, ..
            } = decision
            {
                pending.push((otherwise, depth + 1));
                pending.push((then, depth + 1));
            }
        }
        Ok(Self { body: program.body })
    }

    pub(super) fn execute(&self, facts: &Facts, work: &mut Work) -> Result<Action, Error> {
        let mut decision = &self.body;
        loop {
            work.charge(1)?; // Instruction visit, including return.
            match decision {
                Decision::Return { action } => return Ok(*action),
                Decision::If {
                    field,
                    then,
                    otherwise,
                } => {
                    work.charge(1)?; // Only the executed fact read.
                    let value = match field {
                        Field::Empty => facts.empty,
                        Field::Forced => facts.forced,
                        Field::Recipe => facts.recipe,
                        Field::Nested => facts.nested,
                    };
                    decision = if value { then } else { otherwise };
                }
            }
        }
    }
}
