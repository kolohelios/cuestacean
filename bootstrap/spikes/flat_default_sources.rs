// Copyright 2024 CUE Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// Test-only translation of flat scheduling/history rules from CUE
// 6d609d768f1686f9a3a2a20197cacdbb70e5c79d, internal/core/adt/
// conjunct.go and disjunct2.go (processDisjunctions/doDisjunct/appendDisjunct).
// Exact-i64 equality abstracts complete scalar nodes, not partial vertices.
// Mode transitions are delegated unchanged to the separate batch primitive.

use crate::batch_defaults::{Body, Error, Mode, Trial, execute as execute_batch};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Alternative {
    pub marked: bool,
    pub value: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Clause {
    Value(i64),
    Or(Vec<Alternative>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct State {
    pub value: i64,
    pub mode: Mode,
    pub origin: Mode,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Batch {
    pub trials: Vec<Trial>,
    pub left_drops: bool,
    pub right_drops: bool,
    pub states: Vec<State>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Report {
    pub states: Vec<State>,
    pub batches: Vec<Batch>,
    pub work: usize,
}

const MAX_WIDTH: usize = 16;
const MAX_OCCURRENCES: usize = 128;
const MAX_PRODUCTS: usize = 64;
const MAX_WORK: usize = 4096;

struct Work {
    used: usize,
    limit: usize,
}

impl Work {
    fn charge(&mut self, amount: usize) -> Result<(), Error> {
        if amount > self.limit - self.used {
            return Err(Error::Limit("flat source work"));
        }
        self.used += amount;
        Ok(())
    }
}

// Admission is bounded and does not copy input. It precedes charged execution.
fn admit(clauses: &[Clause]) -> Result<(), Error> {
    if clauses.is_empty() {
        return Err(Error::Unsupported("empty clauses"));
    }
    if clauses.len() > MAX_WIDTH {
        return Err(Error::Limit("flat clauses"));
    }
    let mut groups = 0;
    let mut occurrences = clauses.len();
    for clause in clauses {
        if let Clause::Or(alternatives) = clause {
            if alternatives.len() < 2 {
                return Err(Error::Unsupported("flat disjunction arity"));
            }
            if alternatives.len() > MAX_WIDTH {
                return Err(Error::Limit("flat alternatives"));
            }
            groups += 1;
            occurrences += alternatives.len();
        }
        if groups > MAX_WIDTH || occurrences > MAX_OCCURRENCES {
            return Err(Error::Limit("flat source occurrences"));
        }
    }
    Ok(())
}

// Only the initial predecessor may lack a value. It never becomes public Top.
struct Predecessor {
    value: Option<i64>,
    mode: Mode,
    origin: Mode,
}

fn construct_trials(
    predecessors: &[Predecessor],
    alternatives: &[Alternative],
    work: &mut Work,
) -> Result<Vec<Trial>, Error> {
    // Enforce the raw product bound BEFORE equality filtering or coalescing.
    if predecessors.len() * alternatives.len() > MAX_PRODUCTS {
        return Err(Error::Limit("raw flat products"));
    }
    let mut marked = false;
    for alternative in alternatives {
        work.charge(1)?;
        marked |= alternative.marked;
    }
    let mut trials = Vec::new();
    for predecessor in predecessors {
        work.charge(1)?;
        for alternative in alternatives {
            work.charge(1)?;
            if predecessor
                .value
                .is_some_and(|value| value != alternative.value)
            {
                continue;
            }
            let incoming = if !marked {
                Mode::Maybe
            } else if alternative.marked {
                Mode::Default
            } else {
                Mode::Not
            };
            work.charge(1)?;
            trials.push(Trial {
                mode: predecessor.mode,
                left_mode: predecessor.mode,
                left_origin: predecessor.origin,
                incoming,
                body: Body::Scalar(alternative.value),
            });
        }
    }
    Ok(trials)
}

fn first_origin(trials: &[Trial], value: i64, work: &mut Work) -> Result<Mode, Error> {
    for trial in trials {
        work.charge(1)?;
        if matches!(trial.body, Body::Scalar(candidate) if candidate == value) {
            return Ok(trial.incoming);
        }
    }
    Err(Error::Unsupported("missing flat trial origin"))
}

fn finish(states: Vec<State>, batches: Vec<Batch>, mut work: Work) -> Result<Report, Error> {
    // Charge report output, including the ordered vector entries being moved.
    work.charge(1 + states.len() + batches.len())?;
    Ok(Report {
        states,
        batches,
        work: work.used,
    })
}

/// Construct flat scalar successful trials, not a parser or default selector.
/// Errors expose no partial report; input and the old batch primitive stay intact.
pub fn execute(clauses: &[Clause], work_limit: usize) -> Result<Report, Error> {
    if work_limit > MAX_WORK {
        return Err(Error::Limit("work allowance"));
    }
    admit(clauses)?;
    let mut work = Work {
        used: 0,
        limit: work_limit,
    };
    work.charge(1)?;

    // All ordinary literal constraints meet before any Or is scheduled.
    let mut literal = None;
    let mut conflict = false;
    for clause in clauses {
        work.charge(1)?;
        if let Clause::Value(value) = clause {
            conflict |= literal.is_some_and(|previous| previous != *value);
            literal = Some(*value);
        }
    }
    let mut predecessors = Vec::new();
    if !conflict {
        work.charge(1)?;
        predecessors.push(Predecessor {
            value: literal,
            mode: Mode::Maybe,
            origin: Mode::Maybe,
        });
    }
    // An empty literal frontier still reaches the first scheduled Or. Upstream
    // terminates after its empty cross-product, not before recording that batch.
    let mut batches = Vec::new();
    for clause in clauses {
        work.charge(1)?;
        let Clause::Or(alternatives) = clause else {
            continue;
        };
        let trials = construct_trials(&predecessors, alternatives, &mut work)?;
        let result = execute_batch(&trials, work.limit - work.used)?;
        work.charge(result.work)?;
        let mut states = Vec::new();
        predecessors.clear();
        for leaf in result.results {
            work.charge(1)?;
            let origin = first_origin(&trials, leaf.value, &mut work)?;
            work.charge(2)?;
            states.push(State {
                value: leaf.value,
                mode: leaf.mode,
                origin,
            });
            predecessors.push(Predecessor {
                value: Some(leaf.value),
                mode: leaf.mode,
                origin,
            });
        }
        let empty = states.is_empty();
        work.charge(1)?;
        batches.push(Batch {
            trials,
            left_drops: result.left_drops,
            right_drops: result.right_drops,
            states,
        });
        if empty {
            break;
        }
    }

    let mut states = Vec::new();
    if let Some(batch) = batches.last() {
        for state in &batch.states {
            work.charge(1)?;
            states.push(state.clone());
        }
    } else if let Some(value) = literal.filter(|_| !conflict) {
        work.charge(1)?;
        states.push(State {
            value,
            mode: Mode::Maybe,
            origin: Mode::Maybe,
        });
    }
    finish(states, batches, work)
}
