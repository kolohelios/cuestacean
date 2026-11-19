// Test-only executable dispatch/projection over checked facts, not a solver.
use super::{Error, Work};
use serde::Deserialize;

const MAX_BYTES: usize = 16 * 1024;
const MAX_NODES: usize = 64;
const MAX_DEPTH: usize = 16;

pub struct Policy {
    scope: Scope,
    body: Decision,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Scope {
    Dispatch,
    Projection,
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
    Ordinary,
    Preferred,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
enum Field {
    Empty,
    Forced,
    Recipe,
    Nested,
    Absent,
    Bottom,
}

// Constructed only from the graph's complete bounded observation, never from
// a reference action/readiness or caller-supplied policy input.
pub(super) struct Facts {
    pub empty: bool,
    pub forced: bool,
    pub recipe: bool,
    pub nested: bool,
}

pub(super) struct ProjectionFacts {
    pub absent: bool,
    pub bottom: bool,
}

impl Policy {
    pub fn from_json(text: &str) -> Result<Self, Error> {
        Self::decode(text, Scope::Dispatch)
    }

    pub fn from_projection_json(text: &str) -> Result<Self, Error> {
        Self::decode(text, Scope::Projection)
    }

    pub(super) fn require_dispatch(&self) -> Result<(), Error> {
        self.require_scope(Scope::Dispatch)
    }

    pub(super) fn require_projection(&self) -> Result<(), Error> {
        self.require_scope(Scope::Projection)
    }

    fn require_scope(&self, scope: Scope) -> Result<(), Error> {
        if self.scope != scope {
            return Err(Error::Unsupported("wrong policy scope"));
        }
        Ok(())
    }

    fn decode(text: &str, scope: Scope) -> Result<Self, Error> {
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
            match decision {
                Decision::Return { action } => {
                    let allowed = match action {
                        Action::Constraints | Action::Flat => scope == Scope::Dispatch,
                        Action::Ordinary | Action::Preferred => scope == Scope::Projection,
                        Action::Defer => true,
                    };
                    if !allowed {
                        return Err(Error::Unsupported("wrong-scope policy action"));
                    }
                }
                Decision::If {
                    field,
                    then,
                    otherwise,
                } => {
                    let allowed = match field {
                        Field::Empty | Field::Forced | Field::Recipe | Field::Nested => {
                            scope == Scope::Dispatch
                        }
                        Field::Absent | Field::Bottom => scope == Scope::Projection,
                    };
                    if !allowed {
                        return Err(Error::Unsupported("wrong-scope policy fact"));
                    }
                    pending.push((otherwise, depth + 1));
                    pending.push((then, depth + 1));
                }
            }
        }
        Ok(Self {
            scope,
            body: program.body,
        })
    }

    pub(super) fn execute(&self, facts: &Facts, work: &mut Work) -> Result<Action, Error> {
        self.require_dispatch()?;
        self.run(work, |field| match field {
            Field::Empty => Ok(facts.empty),
            Field::Forced => Ok(facts.forced),
            Field::Recipe => Ok(facts.recipe),
            Field::Nested => Ok(facts.nested),
            Field::Absent | Field::Bottom => Err(Error::Unsupported("non-dispatch fact")),
        })
    }

    pub(super) fn execute_projection(
        &self,
        facts: &ProjectionFacts,
        work: &mut Work,
    ) -> Result<Action, Error> {
        self.require_projection()?;
        self.run(work, |field| match field {
            Field::Absent => Ok(facts.absent),
            Field::Bottom => Ok(facts.bottom),
            Field::Empty | Field::Forced | Field::Recipe | Field::Nested => {
                Err(Error::Unsupported("non-projection fact"))
            }
        })
    }

    fn run(
        &self,
        work: &mut Work,
        read: impl Fn(&Field) -> Result<bool, Error>,
    ) -> Result<Action, Error> {
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
                    let value = read(field)?;
                    decision = if value { then } else { otherwise };
                }
            }
        }
    }
}
