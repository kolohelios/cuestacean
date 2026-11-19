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
// Test-only translation of CUE 6d609d768f1686f9a3a2a20197cacdbb70e5c79d:
// internal/core/adt/disjunct2.go crossProduct/combineDefault2/appendDisjunct
// and disjunct.go combineDefault. Only the zero-priority successful-batch phase
// is modeled, with an initially empty destination. Exact scalar value equality
// abstracts complete nodes; this is not partial vertex or source identity.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mode {
    Maybe,
    Default,
    Not,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Leaf {
    pub value: i64,
    pub mode: Mode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Body {
    Scalar(i64),
    Nested(Vec<Leaf>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Trial {
    pub mode: Mode,
    pub left_mode: Mode,
    pub left_origin: Mode,
    pub incoming: Mode,
    pub body: Body,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Report {
    pub left_drops: bool,
    pub right_drops: bool,
    pub has_non_maybe: bool,
    pub results: Vec<Leaf>,
    pub work: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    Unsupported(&'static str),
    Limit(&'static str),
}

const MAX_WIDTH: usize = 64;
const MAX_RECORDS: usize = 128;
const MAX_WORK: usize = 4096;

struct Work {
    used: usize,
    limit: usize,
}

impl Work {
    fn charge(&mut self, amount: usize) -> Result<(), Error> {
        if amount > self.limit - self.used {
            return Err(Error::Limit("batch work"));
        }
        self.used += amount;
        Ok(())
    }
}

// Structural admission is separately bounded, without copying caller input.
fn admit(trials: &[Trial]) -> Result<(), Error> {
    if trials.len() > MAX_WIDTH {
        return Err(Error::Limit("batch trials"));
    }
    let mut records = trials.len();
    let mut expanded = 0;
    for trial in trials {
        expanded += match &trial.body {
            Body::Scalar(_) => 1,
            Body::Nested(children) => {
                if children.len() < 2 {
                    return Err(Error::Unsupported("nested arity"));
                }
                if children.len() > MAX_WIDTH {
                    return Err(Error::Limit("nested children"));
                }
                records += children.len();
                children.len()
            }
        };
        if records > MAX_RECORDS {
            return Err(Error::Limit("batch records"));
        }
        if expanded > MAX_WIDTH {
            return Err(Error::Limit("raw expanded leaves"));
        }
    }
    Ok(())
}

fn maximum(a: Mode, b: Mode) -> Mode {
    match (a, b) {
        (Mode::Not, _) | (_, Mode::Not) => Mode::Not,
        (Mode::Default, _) | (_, Mode::Default) => Mode::Default,
        _ => Mode::Maybe,
    }
}

fn combine(a: Mode, b: Mode, drop_a: bool, drop_b: bool, work: &mut Work) -> Result<Mode, Error> {
    work.charge(1)?;
    Ok(maximum(
        if drop_a { Mode::Maybe } else { a },
        if drop_b { Mode::Maybe } else { b },
    ))
}

fn append(results: &mut Vec<Leaf>, leaf: Leaf, work: &mut Work) -> Result<(), Error> {
    work.charge(1)?;
    for existing in results.iter_mut() {
        work.charge(1)?;
        if existing.value == leaf.value {
            if leaf.mode == Mode::Default {
                existing.mode = Mode::Default;
            }
            return Ok(());
        }
    }
    work.charge(1)?;
    results.push(leaf);
    Ok(())
}

/// Execute one already-successful batch, not source scheduling or a solver.
/// Admission precedes charged execution; errors never publish a partial report.
pub fn execute(trials: &[Trial], work_limit: usize) -> Result<Report, Error> {
    if work_limit > MAX_WORK {
        return Err(Error::Limit("work allowance"));
    }
    admit(trials)?;
    let mut work = Work {
        used: 0,
        limit: work_limit,
    };
    work.charge(1)?;
    let mut left_drops = true;
    let mut right_drops = true;
    for trial in trials {
        work.charge(1)?;
        if trial.left_mode == Mode::Default || trial.left_origin == Mode::Default {
            left_drops = false;
        }
        if trial.incoming == Mode::Default {
            right_drops = false;
        }
    }

    let mut results = Vec::new();
    let mut has_non_maybe = false;
    for trial in trials {
        work.charge(1)?;
        match &trial.body {
            Body::Scalar(value) => {
                let mode = combine(
                    trial.mode,
                    trial.incoming,
                    left_drops,
                    right_drops,
                    &mut work,
                )?;
                append(
                    &mut results,
                    Leaf {
                        value: *value,
                        mode,
                    },
                    &mut work,
                )?;
            }
            Body::Nested(children) => {
                for child in children {
                    work.charge(1)?;
                    // This maximum is a separate upstream combineDefault call.
                    work.charge(1)?;
                    let incoming = maximum(trial.incoming, child.mode);
                    let mode = combine(trial.mode, incoming, left_drops, false, &mut work)?;
                    has_non_maybe |= mode != Mode::Maybe;
                    append(
                        &mut results,
                        Leaf {
                            value: child.value,
                            mode,
                        },
                        &mut work,
                    )?;
                }
            }
        }
    }
    if has_non_maybe {
        for result in &mut results {
            work.charge(1)?;
            if result.mode == Mode::Maybe {
                result.mode = Mode::Not;
            }
        }
    }
    // Charge the header and all leaf copies before report allocation/copying.
    work.charge(1 + results.len())?;
    Ok(Report {
        left_drops,
        right_drops,
        has_non_maybe,
        results: results.clone(),
        work: work.used,
    })
}
