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

/// Encounter position in the supplied batch, not a source identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Occurrence {
    pub trial: usize,
    pub child: Option<usize>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Comparison {
    /// First insertion into the compared destination slot, even after promotion.
    pub occurrence: Occurrence,
    pub existing: Leaf,
    pub matched: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Append {
    pub occurrence: Occurrence,
    pub candidate: Leaf,
    pub index: usize,
    pub coalesced: bool,
    pub comparisons: Vec<Comparison>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct AuditedReport {
    pub report: Report,
    pub appends: Vec<Append>,
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

struct Audit {
    appends: Vec<Append>,
    first: Vec<Occurrence>,
    comparisons: usize,
}

// Extra audit work, always before the covered snapshots/allocations/insertion:
// initialization 3; candidate (occurrence, leaf, comparison-vector header) 3;
// comparison (first occurrence, existing leaf, insertion) 3; first insertion 1;
// completed append insertion 1; final retention 1 + appends + comparisons.
// Successful total = old work + 4 + 5*A + 4*C + U. Retention is logical work
// even when buffers move. No quadratic preallocation or separate audit budget.
fn append(
    results: &mut Vec<Leaf>,
    leaf: Leaf,
    position: (usize, Option<usize>),
    audit: &mut Option<Audit>,
    work: &mut Work,
) -> Result<(), Error> {
    work.charge(1)?;
    let mut event = if audit.is_some() {
        work.charge(3)?;
        Some(Append {
            occurrence: Occurrence {
                trial: position.0,
                child: position.1,
            },
            candidate: leaf.clone(),
            index: results.len(),
            coalesced: false,
            comparisons: Vec::new(),
        })
    } else {
        None
    };
    for (index, existing) in results.iter_mut().enumerate() {
        work.charge(1)?;
        // Reserve evidence work before copying the actual pre-promotion state.
        if let (Some(audit), Some(event)) = (audit.as_mut(), event.as_mut()) {
            work.charge(3)?;
            event.comparisons.push(Comparison {
                occurrence: audit.first[index],
                existing: existing.clone(),
                matched: false,
            });
            audit.comparisons += 1;
        }
        let matched = existing.value == leaf.value;
        if let Some(event) = event.as_mut() {
            event.comparisons[index].matched = matched;
        }
        if matched {
            if leaf.mode == Mode::Default {
                existing.mode = Mode::Default;
            }
            if let Some(event) = event.as_mut() {
                event.index = index;
                event.coalesced = true;
            }
            if let (Some(audit), Some(event)) = (audit.as_mut(), event) {
                work.charge(1)?;
                audit.appends.push(event);
            }
            return Ok(());
        }
    }
    work.charge(1)?;
    if let (Some(audit), Some(event)) = (audit.as_mut(), event.as_ref()) {
        work.charge(1)?;
        audit.first.push(event.occurrence);
    }
    results.push(leaf);
    if let (Some(audit), Some(event)) = (audit.as_mut(), event) {
        work.charge(1)?;
        audit.appends.push(event);
    }
    Ok(())
}

/// Execute one already-successful batch, not source scheduling or a solver.
/// Admission precedes charged execution; errors never publish a partial report.
pub fn execute(trials: &[Trial], work_limit: usize) -> Result<Report, Error> {
    execute_inner(trials, work_limit, false).map(|(report, _)| report)
}

/// Capture the actual append comparisons in this invocation, not a replay.
/// The returned work includes all audit retention; errors expose no transcript.
pub fn execute_audited(trials: &[Trial], work_limit: usize) -> Result<AuditedReport, Error> {
    let (report, audit) = execute_inner(trials, work_limit, true)?;
    Ok(AuditedReport {
        report,
        appends: audit.map_or_else(Vec::new, |audit| audit.appends),
    })
}

fn execute_inner(
    trials: &[Trial],
    work_limit: usize,
    audited: bool,
) -> Result<(Report, Option<Audit>), Error> {
    if work_limit > MAX_WORK {
        return Err(Error::Limit("work allowance"));
    }
    admit(trials)?;
    let mut work = Work {
        used: 0,
        limit: work_limit,
    };
    work.charge(1)?;
    let mut audit = if audited {
        work.charge(3)?;
        Some(Audit {
            appends: Vec::new(),
            first: Vec::new(),
            comparisons: 0,
        })
    } else {
        None
    };
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
    for (trial_index, trial) in trials.iter().enumerate() {
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
                    (trial_index, None),
                    &mut audit,
                    &mut work,
                )?;
            }
            Body::Nested(children) => {
                for (child_index, child) in children.iter().enumerate() {
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
                        (trial_index, Some(child_index)),
                        &mut audit,
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
    if let Some(audit) = &audit {
        work.charge(1 + audit.appends.len() + audit.comparisons)?;
    }
    Ok((
        Report {
            left_drops,
            right_drops,
            has_non_maybe,
            results: results.clone(),
            work: work.used,
        },
        audit,
    ))
}
