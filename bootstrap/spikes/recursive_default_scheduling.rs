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
// Test-only scheduling hypothesis translated from CUE
// 6d609d768f1686f9a3a2a20197cacdbb70e5c79d, internal/core/adt/disjunct2.go
// (processDisjunctions/doDisjunct/crossProduct/appendDisjunct). Exact-i64
// coalescing and wrapper retention are experimental abstractions: run modes,
// partial-vertex equality, layers and references are not modeled. All mode
// transitions delegate to the unchanged successful-batch primitive.

use crate::batch_defaults::{Body, Error, Leaf, Mode, Trial, execute as execute_batch};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expr {
    Value(i64),
    And(Vec<Expr>),
    Or(Vec<Alternative>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Alternative {
    pub marked: bool,
    pub expr: Expr,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct State {
    pub value: i64,
    pub mode: Mode,
    pub origin: Mode,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Batch {
    pub id: usize,
    pub depth: usize,
    pub trials: Vec<Trial>,
    pub nested_origins: Vec<Vec<Mode>>,
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
const MAX_DEPTH: usize = 8;
const MAX_GROUPS: usize = 16;
const MAX_PRODUCTS: usize = 64;
const MAX_BATCHES: usize = 128;
const MAX_WORK: usize = 4096;

// Structural admission precedes execution, even for a zero work allowance.
fn admit(
    expr: &Expr,
    depth: usize,
    occurrences: &mut usize,
    groups: &mut usize,
) -> Result<(), Error> {
    *occurrences += 1;
    if *occurrences > MAX_OCCURRENCES || depth > MAX_DEPTH {
        return Err(Error::Limit("recursive IR shape"));
    }
    match expr {
        Expr::Value(_) => {}
        Expr::And(children) => {
            admit_width(children.len())?;
            for child in children {
                admit(child, depth + 1, occurrences, groups)?;
            }
        }
        Expr::Or(alternatives) => {
            admit_width(alternatives.len())?;
            *groups += 1;
            if *groups > MAX_GROUPS {
                return Err(Error::Limit("recursive Or groups"));
            }
            for alternative in alternatives {
                // Alternative wrappers count as occurrences, not Expr levels.
                *occurrences += 1;
                if *occurrences > MAX_OCCURRENCES {
                    return Err(Error::Limit("recursive IR occurrences"));
                }
                admit(&alternative.expr, depth + 1, occurrences, groups)?;
            }
        }
    }
    Ok(())
}

fn admit_width(width: usize) -> Result<(), Error> {
    if width < 2 {
        return Err(Error::Unsupported("recursive vector arity"));
    }
    if width > MAX_WIDTH {
        return Err(Error::Limit("recursive vector width"));
    }
    Ok(())
}

struct Work {
    used: usize,
    limit: usize,
}

impl Work {
    fn charge(&mut self, amount: usize) -> Result<(), Error> {
        if amount > self.limit - self.used {
            return Err(Error::Limit("recursive scheduling work"));
        }
        self.used += amount;
        Ok(())
    }
}

struct Seed {
    value: Option<i64>,
    mode: Mode,
    origin: Mode,
}

// A multi-result frame retains its wrapper separately from its child states.
// Empty is a semantic failed attempt, not an admission/work error.
enum Frame {
    Empty,
    Scalar(State),
    Nested {
        mode: Mode,
        origin: Mode,
        children: Vec<State>,
    },
}

struct Scheduler {
    work: Work,
    // Reserve slots on entry; children complete before their parent's slot.
    batches: Vec<Option<Batch>>,
}

struct Constraints<'a> {
    value: Option<i64>,
    conflict: bool,
    groups: Vec<&'a [Alternative]>,
}

impl Constraints<'_> {
    fn meet(&mut self, value: i64) {
        self.conflict |= self.value.is_some_and(|previous| previous != value);
        self.value = Some(value);
    }
}

fn collect<'a>(expr: &'a Expr, facts: &mut Constraints<'a>, work: &mut Work) -> Result<(), Error> {
    work.charge(1)?;
    match expr {
        Expr::Value(value) => facts.meet(*value),
        Expr::And(children) => {
            for child in children {
                collect(child, facts, work)?;
            }
        }
        Expr::Or(alternatives) => {
            work.charge(1)?;
            facts.groups.push(alternatives);
        }
    }
    Ok(())
}

impl Scheduler {
    fn eval_frame(&mut self, expr: &Expr, seed: Seed, depth: usize) -> Result<Frame, Error> {
        self.work.charge(1)?;
        let mut facts = Constraints {
            value: seed.value,
            conflict: false,
            groups: Vec::new(),
        };
        collect(expr, &mut facts, &mut self.work)?;
        let mut predecessors = Vec::new();
        if !facts.conflict {
            self.work.charge(1)?;
            predecessors.push(Seed {
                value: facts.value,
                mode: seed.mode,
                origin: seed.origin,
            });
        }
        let mut states = Vec::new();
        for alternatives in &facts.groups {
            self.work.charge(1)?;
            states = self.phase(&predecessors, alternatives, depth)?;
            if states.is_empty() {
                return Ok(Frame::Empty);
            }
            predecessors.clear();
            for state in &states {
                self.work.charge(1)?;
                predecessors.push(Seed {
                    value: Some(state.value),
                    mode: state.mode,
                    origin: state.origin,
                });
            }
        }
        if facts.groups.is_empty() && !facts.conflict {
            let value = facts
                .value
                .ok_or(Error::Unsupported("unconstrained frame"))?;
            self.work.charge(1)?;
            states.push(State {
                value,
                mode: seed.mode,
                origin: seed.origin,
            });
        }
        self.work.charge(1)?;
        Ok(match states.len() {
            0 => Frame::Empty,
            // Frame forwarding does NOT perform doDisjunct's origin write.
            1 => Frame::Scalar(
                states
                    .pop()
                    .ok_or(Error::Unsupported("missing singleton"))?,
            ),
            _ => Frame::Nested {
                mode: seed.mode,
                origin: seed.origin,
                children: states,
            },
        })
    }

    fn do_attempt(
        &mut self,
        expr: &Expr,
        predecessor: &Seed,
        incoming: Mode,
        depth: usize,
    ) -> Result<Frame, Error> {
        self.work.charge(1)?;
        let mut result = self.eval_frame(
            expr,
            Seed {
                value: predecessor.value,
                mode: predecessor.mode,
                origin: incoming,
            },
            depth,
        )?;
        // The actual enclosing doDisjunct overwrites the dereferenced state's
        // origin once, AFTER frame completion. Batch snapshots are independent.
        self.work.charge(1)?;
        match &mut result {
            Frame::Empty => {}
            Frame::Scalar(state) => state.origin = incoming,
            Frame::Nested { origin, .. } => *origin = incoming,
        }
        Ok(result)
    }

    fn phase(
        &mut self,
        predecessors: &[Seed],
        alternatives: &[Alternative],
        depth: usize,
    ) -> Result<Vec<State>, Error> {
        self.work.charge(1)?;
        if self.batches.len() == MAX_BATCHES {
            return Err(Error::Limit("dynamic batches"));
        }
        let id = self.batches.len();
        self.work.charge(1)?;
        self.batches.push(None);
        // Before filtering OR recursive attempts, not just successful trials.
        if predecessors.len() * alternatives.len() > MAX_PRODUCTS {
            return Err(Error::Limit("raw recursive products"));
        }
        let mut marked = false;
        for alternative in alternatives {
            self.work.charge(1)?;
            marked |= alternative.marked;
        }
        let mut trials = Vec::new();
        let mut nested_origins = Vec::new();
        for predecessor in predecessors {
            self.work.charge(1)?;
            for alternative in alternatives {
                self.work.charge(1)?;
                let incoming = if !marked {
                    Mode::Maybe
                } else if alternative.marked {
                    Mode::Default
                } else {
                    Mode::Not
                };
                let result =
                    self.do_attempt(&alternative.expr, predecessor, incoming, depth + 1)?;
                let (mode, body, origins) = match result {
                    Frame::Empty => continue,
                    Frame::Scalar(state) => {
                        self.work.charge(1)?;
                        (state.mode, Body::Scalar(state.value), Vec::new())
                    }
                    Frame::Nested { mode, children, .. } => {
                        let mut leaves = Vec::new();
                        let mut origins = Vec::new();
                        for child in children {
                            self.work.charge(3)?;
                            leaves.push(Leaf {
                                value: child.value,
                                mode: child.mode,
                            });
                            origins.push(child.origin);
                        }
                        (mode, Body::Nested(leaves), origins)
                    }
                };
                self.work.charge(2)?;
                trials.push(Trial {
                    mode,
                    left_mode: predecessor.mode,
                    left_origin: predecessor.origin,
                    incoming,
                    body,
                });
                nested_origins.push(origins);
            }
        }
        let result = execute_batch(&trials, self.work.limit - self.work.used)?;
        self.work.charge(result.work)?;
        let mut states = Vec::new();
        for leaf in result.results {
            self.work.charge(1)?;
            let origin = first_origin(&trials, &nested_origins, leaf.value, &mut self.work)?;
            self.work.charge(1)?;
            states.push(State {
                value: leaf.value,
                mode: leaf.mode,
                origin,
            });
        }
        let mut snapshot = Vec::new();
        for state in &states {
            self.work.charge(1)?;
            snapshot.push(state.clone());
        }
        self.work.charge(1)?;
        self.batches[id] = Some(Batch {
            id,
            depth,
            trials,
            nested_origins,
            left_drops: result.left_drops,
            right_drops: result.right_drops,
            states: snapshot,
        });
        Ok(states)
    }
}

// Mirror encounter order only to recover history, never to infer source identity
// or recompute modes. Default promotion in execute_batch retains the first origin.
fn first_origin(
    trials: &[Trial],
    nested_origins: &[Vec<Mode>],
    value: i64,
    work: &mut Work,
) -> Result<Mode, Error> {
    for (trial, origins) in trials.iter().zip(nested_origins) {
        work.charge(1)?;
        match &trial.body {
            Body::Scalar(candidate) if *candidate == value => return Ok(trial.incoming),
            Body::Scalar(_) => {}
            Body::Nested(children) => {
                for (child, origin) in children.iter().zip(origins) {
                    work.charge(1)?;
                    if child.value == value {
                        return Ok(*origin);
                    }
                }
            }
        }
    }
    Err(Error::Unsupported("missing recursive trial origin"))
}

/// Run the bounded recursive scalar scheduling hypothesis, not a CUE evaluator.
/// No oracle inputs, preferred-value projection, fallback or partial error report.
pub fn execute(expr: &Expr, work_limit: usize) -> Result<Report, Error> {
    if work_limit > MAX_WORK {
        return Err(Error::Limit("work allowance"));
    }
    admit(expr, 1, &mut 0, &mut 0)?;
    let mut scheduler = Scheduler {
        work: Work {
            used: 0,
            limit: work_limit,
        },
        batches: Vec::new(),
    };
    scheduler.work.charge(1)?;
    let frame = scheduler.eval_frame(
        expr,
        Seed {
            value: None,
            mode: Mode::Maybe,
            origin: Mode::Maybe,
        },
        0,
    )?;
    // Root has no enclosing doAttempt: retain final-phase child history.
    let states = match frame {
        Frame::Empty => Vec::new(),
        Frame::Scalar(state) => {
            scheduler.work.charge(1)?;
            vec![state]
        }
        Frame::Nested { children, .. } => children,
    };
    let mut batches = Vec::new();
    for batch in scheduler.batches {
        scheduler.work.charge(1)?;
        batches.push(batch.ok_or(Error::Unsupported("unfinished batch"))?);
    }
    scheduler.work.charge(1 + states.len() + batches.len())?;
    Ok(Report {
        states,
        batches,
        work: scheduler.work.used,
    })
}
