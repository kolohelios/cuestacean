// Test-only residual preference hypothesis, not a production CUE evaluator.
// Keep ordinary occurrence worlds intact until the final report projection.

#[path = "demand_policy.rs"]
pub mod policy;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expr {
    Top,
    Int,
    Value(i64),
    And(Box<Expr>, Box<Expr>),
    Or(Vec<(bool, Expr)>),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Domain {
    Top,
    Int,
    Value(i64),
}

#[derive(Debug, Eq, PartialEq)]
pub enum Preference {
    Absent,
    Bottom,
    Values(Vec<Domain>),
}

#[derive(Debug, Eq, PartialEq)]
pub enum Outcome {
    Concrete(i64),
    Incomplete,
    Conflict,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    Unsupported(&'static str),
    Limit(&'static str),
}

#[derive(Debug, Eq, PartialEq)]
pub struct Report {
    pub constraints: Vec<Domain>,
    pub preferred: Preference,
    pub outcome: Outcome,
    pub work: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Support {
    Unentered,
    Unmarked,
    MarkedLive,
    MarkedEliminated,
}

#[derive(Debug, Eq, PartialEq)]
pub struct GroupFact {
    pub group: usize,
    pub support: Support,
    pub retained: Vec<usize>,
    pub suppressed: Vec<usize>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Readiness {
    ConstraintOnly,
    FlatPreference,
    DeferredNested,
    PolicyDeferred,
    ProjectionDeferred,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Demand {
    Concrete(i64),
    Incomplete,
    Conflict,
    Suspended,
}

#[derive(Debug, Eq, PartialEq)]
pub struct DemandReport {
    pub constraints: Vec<Domain>,
    pub groups: Vec<GroupFact>,
    pub preferred: Option<Preference>,
    pub readiness: Readiness,
    pub demand: Demand,
    pub work: usize,
}

const MAX_NODES: usize = 128;
const MAX_DEPTH: usize = 16;
const MAX_GROUPS: usize = 16;
const MAX_ALTERNATIVES: usize = 16;
const MAX_WORLDS: usize = 64;
const MAX_WORK: usize = 4096;

type Id = usize;

#[derive(Debug)]
enum NodeKind {
    Atom(Domain),
    And(Id, Id),
    Or(Vec<Alternative>),
}

#[derive(Debug)]
struct Alternative {
    marked: bool,
    child: Id,
    choice: u128,
}

#[derive(Debug)]
struct Node {
    kind: NodeKind,
    // Syntactic existence is deliberately distinct from contextual presence.
    recipe: bool,
}

#[derive(Debug)]
pub struct Graph {
    nodes: Vec<Node>,
    root: Id,
}

#[derive(Clone, Copy)]
struct World {
    domain: Domain,
    lineage: u128,
}

// Each bit selects one *complete ordinary world*, not a scalar domain. Thus
// int and 1 can intersect through an ordinary meet, but equal values from an
// unentered branch cannot activate that branch's preference. There are at most
// 64 ordinary worlds; all occurrence choices live in World::lineage instead.
#[derive(Clone, Copy)]
enum Association {
    Absent,
    Present(u64),
}

struct Observation {
    worlds: Vec<World>,
    constraints: Vec<Domain>,
    groups: Vec<GroupFact>,
    nested: bool,
}

struct Work {
    used: usize,
    limit: usize,
}

impl Work {
    fn charge(&mut self, amount: usize) -> Result<(), Error> {
        if amount > self.limit - self.used {
            return Err(Error::Limit("solve work"));
        }
        self.used += amount;
        Ok(())
    }
}

impl Graph {
    pub fn new(expression: Expr) -> Result<Self, Error> {
        // Validate borrowed nodes with a bounded stack before compilation.
        // Destruction of the caller-built owned tree is not a hardened boundary.
        let mut pending = vec![(&expression, 1)];
        let mut occurrences = 0;
        let mut groups = 0;
        while let Some((expression, depth)) = pending.pop() {
            if depth > MAX_DEPTH {
                return Err(Error::Limit("syntax depth"));
            }
            occurrences += 1;
            if occurrences > MAX_NODES {
                return Err(Error::Limit("IR occurrences"));
            }
            match expression {
                Expr::Top | Expr::Int | Expr::Value(_) => {}
                Expr::And(left, right) => {
                    if occurrences + pending.len() + 2 > MAX_NODES {
                        return Err(Error::Limit("IR occurrences"));
                    }
                    pending.push((right, depth + 1));
                    pending.push((left, depth + 1));
                }
                Expr::Or(alternatives) => {
                    if alternatives.len() < 2 {
                        return Err(Error::Unsupported("Or requires at least two alternatives"));
                    }
                    if alternatives.len() > MAX_ALTERNATIVES {
                        return Err(Error::Limit("Or alternatives"));
                    }
                    groups += 1;
                    if groups > MAX_GROUPS {
                        return Err(Error::Limit("Or groups"));
                    }
                    if occurrences + pending.len() + alternatives.len() > MAX_NODES {
                        return Err(Error::Limit("IR occurrences"));
                    }
                    pending.extend(
                        alternatives
                            .iter()
                            .rev()
                            .map(|(_, child)| (child, depth + 1)),
                    );
                }
            }
        }
        let mut graph = Self {
            nodes: Vec::with_capacity(occurrences),
            root: 0,
        };
        let mut choices = 0;
        graph.root = graph.compile(&expression, &mut choices);
        Ok(graph)
    }

    fn compile(&mut self, expression: &Expr, choices: &mut u32) -> Id {
        let (kind, recipe) = match expression {
            Expr::Top => (NodeKind::Atom(Domain::Top), false),
            Expr::Int => (NodeKind::Atom(Domain::Int), false),
            Expr::Value(value) => (NodeKind::Atom(Domain::Value(*value)), false),
            Expr::And(left, right) => {
                let left = self.compile(left, choices);
                let right = self.compile(right, choices);
                (
                    NodeKind::And(left, right),
                    self.nodes[left].recipe || self.nodes[right].recipe,
                )
            }
            Expr::Or(terms) => {
                let mut recipe = false;
                let mut alternatives = Vec::with_capacity(terms.len());
                for (marked, child) in terms {
                    // A tree has at most 127 nonroot occurrences, and each
                    // alternative consumes one such edge. No shift can reach 128.
                    let choice = 1_u128 << *choices;
                    *choices += 1;
                    let child = self.compile(child, choices);
                    recipe |= *marked || self.nodes[child].recipe;
                    alternatives.push(Alternative {
                        marked: *marked,
                        child,
                        choice,
                    });
                }
                (NodeKind::Or(alternatives), recipe)
            }
        };
        let id = self.nodes.len();
        self.nodes.push(Node { kind, recipe });
        id
    }

    pub fn solve(&self, work_limit: usize) -> Result<Report, Error> {
        if work_limit > MAX_WORK {
            return Err(Error::Limit("work allowance exceeds hard cap"));
        }
        let mut work = Work {
            used: 0,
            limit: work_limit,
        };
        let worlds = self.ordinary(self.root, &mut work)?;
        let constraints = project(&worlds, None, &mut work)?;
        if worlds.is_empty() {
            return Ok(Report {
                constraints,
                preferred: Preference::Absent,
                outcome: Outcome::Conflict,
                work: work.used,
            });
        }
        let association = self.preference(self.root, 0, &worlds, &mut work)?;
        let preferred = match association {
            Association::Absent => Preference::Absent,
            Association::Present(0) => Preference::Bottom,
            Association::Present(selected) => {
                Preference::Values(project(&worlds, Some(selected), &mut work)?)
            }
        };
        let selected = match &preferred {
            Preference::Values(values) => values,
            Preference::Absent | Preference::Bottom => &constraints,
        };
        let outcome = match selected.as_slice() {
            [Domain::Value(value)] => Outcome::Concrete(*value),
            _ => Outcome::Incomplete,
        };
        Ok(Report {
            constraints,
            preferred,
            outcome,
            work: work.used,
        })
    }

    pub fn demand(&self, refinement: Domain, work_limit: usize) -> Result<DemandReport, Error> {
        if work_limit > MAX_WORK {
            return Err(Error::Limit("work allowance exceeds hard cap"));
        }
        let mut work = Work {
            used: 0,
            limit: work_limit,
        };
        let Observation {
            worlds,
            constraints,
            groups,
            nested,
        } = self.observe(refinement, &mut work)?;
        let (readiness, demand, preferred) = if worlds.is_empty() {
            (Readiness::ConstraintOnly, Demand::Conflict, None)
        } else if let [Domain::Value(value)] = constraints.as_slice() {
            (Readiness::ConstraintOnly, Demand::Concrete(*value), None)
        } else if !self.nodes[self.root].recipe {
            (Readiness::ConstraintOnly, Demand::Incomplete, None)
        } else if nested {
            (Readiness::DeferredNested, Demand::Suspended, None)
        } else {
            let (demand, preferred) = self.flat(&worlds, &constraints, &mut work)?;
            (Readiness::FlatPreference, demand, Some(preferred))
        };
        work.charge(1)?;
        Ok(DemandReport {
            constraints,
            groups,
            preferred,
            readiness,
            demand,
            work: work.used,
        })
    }

    pub fn demand_with_policy(
        &self,
        refinement: Domain,
        work_limit: usize,
        policy: &policy::Policy,
    ) -> Result<DemandReport, Error> {
        policy.require_dispatch()?;
        self.demand_policies(refinement, work_limit, policy, None)
    }

    pub fn demand_with_policies(
        &self,
        refinement: Domain,
        work_limit: usize,
        dispatch: &policy::Policy,
        projection: &policy::Policy,
    ) -> Result<DemandReport, Error> {
        // Structural typing precedes work/allowance checks, even on paths that
        // will never execute the projection policy. Admission itself is free.
        dispatch.require_dispatch()?;
        projection.require_projection()?;
        self.demand_policies(refinement, work_limit, dispatch, Some(projection))
    }

    fn demand_policies(
        &self,
        refinement: Domain,
        work_limit: usize,
        policy: &policy::Policy,
        projection: Option<&policy::Policy>,
    ) -> Result<DemandReport, Error> {
        if work_limit > MAX_WORK {
            return Err(Error::Limit("work allowance exceeds hard cap"));
        }
        let mut work = Work {
            used: 0,
            limit: work_limit,
        };
        let Observation {
            worlds,
            constraints,
            groups,
            nested,
        } = self.observe(refinement, &mut work)?;
        let facts = policy::Facts {
            empty: worlds.is_empty(),
            forced: matches!(constraints.as_slice(), [Domain::Value(_)]),
            recipe: self.nodes[self.root].recipe,
            nested,
        };
        let action = policy.execute(&facts, &mut work)?;
        work.charge(1)?; // Response certificate, before any action execution.
        let (readiness, demand, preferred) = match action {
            policy::Action::Constraints => {
                if !(facts.empty || facts.forced || !facts.recipe) {
                    return Err(Error::Unsupported("uncertified constraint action"));
                }
                let demand = if facts.empty {
                    Demand::Conflict
                } else if let [Domain::Value(value)] = constraints.as_slice() {
                    Demand::Concrete(*value)
                } else {
                    Demand::Incomplete
                };
                (Readiness::ConstraintOnly, demand, None)
            }
            policy::Action::Flat => {
                if facts.nested {
                    return Err(Error::Unsupported("uncertified flat action"));
                }
                if let Some(projection) = projection {
                    let (readiness, demand, preferred) =
                        self.project_flat(&worlds, &constraints, projection, &mut work)?;
                    (readiness, demand, Some(preferred))
                } else {
                    let (demand, preferred) = self.flat(&worlds, &constraints, &mut work)?;
                    (Readiness::FlatPreference, demand, Some(preferred))
                }
            }
            policy::Action::Defer => (Readiness::PolicyDeferred, Demand::Suspended, None),
            policy::Action::Ordinary | policy::Action::Preferred => {
                return Err(Error::Unsupported("non-dispatch action"));
            }
        };
        work.charge(1)?;
        Ok(DemandReport {
            constraints,
            groups,
            preferred,
            readiness,
            demand,
            work: work.used,
        })
    }

    fn observe(&self, refinement: Domain, work: &mut Work) -> Result<Observation, Error> {
        let ordinary = self.ordinary(self.root, work)?;
        // Inspect, meet and copy each world before allocating its refined table.
        // Do not merge equal domains: their occurrence lineages remain distinct.
        work.charge(ordinary.len() * 3)?;
        let mut worlds = Vec::with_capacity(ordinary.len());
        for world in ordinary {
            if let Some(domain) = meet(world.domain, refinement) {
                worlds.push(World {
                    domain,
                    lineage: world.lineage,
                });
            }
        }
        let mut groups = Vec::new();
        let mut nested = false;
        self.audit(self.root, 0, &worlds, &mut groups, &mut nested, work)?;
        let constraints = project(&worlds, None, work)?;
        Ok(Observation {
            worlds,
            constraints,
            groups,
            nested,
        })
    }

    fn flat(
        &self,
        worlds: &[World],
        constraints: &[Domain],
        work: &mut Work,
    ) -> Result<(Demand, Preference), Error> {
        let association = self.preference(self.root, 0, worlds, work)?;
        let preferred = match association {
            Association::Absent => Preference::Absent,
            Association::Present(0) => Preference::Bottom,
            Association::Present(selected) => {
                Preference::Values(project(worlds, Some(selected), work)?)
            }
        };
        let selected = match &preferred {
            Preference::Values(values) => values,
            Preference::Absent | Preference::Bottom => constraints,
        };
        let demand = if worlds.is_empty() {
            Demand::Conflict
        } else if let [Domain::Value(value)] = selected {
            Demand::Concrete(*value)
        } else {
            Demand::Incomplete
        };
        Ok((demand, preferred))
    }

    // Called only after the dispatch flat certificate has excluded entered
    // nested recipes. Association computes facts, not a preselected domain.
    fn project_flat(
        &self,
        worlds: &[World],
        constraints: &[Domain],
        policy: &policy::Policy,
        work: &mut Work,
    ) -> Result<(Readiness, Demand, Preference), Error> {
        let association = self.preference(self.root, 0, worlds, work)?;
        let facts = policy::ProjectionFacts {
            absent: matches!(association, Association::Absent),
            bottom: matches!(association, Association::Present(0)),
        };
        let action = policy.execute_projection(&facts, work)?;
        work.charge(1)?; // Projection certificate, before report allocation.
        match action {
            policy::Action::Ordinary if facts.absent || facts.bottom => {}
            policy::Action::Preferred if !facts.absent && !facts.bottom => {}
            policy::Action::Defer => {}
            _ => return Err(Error::Unsupported("uncertified projection action")),
        }
        // Even deferral publishes the observed flat preference. Unsafe actions
        // above must reject before projecting/copying preferred candidates.
        let preferred = match association {
            Association::Absent => Preference::Absent,
            Association::Present(0) => Preference::Bottom,
            Association::Present(selected) => {
                Preference::Values(project(worlds, Some(selected), work)?)
            }
        };
        if let policy::Action::Defer = action {
            return Ok((Readiness::ProjectionDeferred, Demand::Suspended, preferred));
        }
        work.charge(1)?; // New-path scalar extraction; no extraction on defer.
        let selected = match action {
            policy::Action::Ordinary => constraints,
            policy::Action::Preferred => match &preferred {
                Preference::Values(values) => values,
                _ => return Err(Error::Unsupported("uncertified preferred domain")),
            },
            _ => return Err(Error::Unsupported("non-projection action")),
        };
        let demand = if worlds.is_empty() {
            Demand::Conflict
        } else if let [Domain::Value(value)] = selected {
            Demand::Concrete(*value)
        } else {
            Demand::Incomplete
        };
        Ok((Readiness::FlatPreference, demand, preferred))
    }

    // Traverse every source occurrence, including impossible and M3-suppressed
    // children. This observes syntax and ordinary support, never preferences.
    // Compilation uses postorder IDs, so postorder emission sorts group facts.
    fn audit(
        &self,
        id: Id,
        context: u128,
        worlds: &[World],
        groups: &mut Vec<GroupFact>,
        nested: &mut bool,
        work: &mut Work,
    ) -> Result<(), Error> {
        work.charge(1)?;
        match &self.nodes[id].kind {
            NodeKind::Atom(_) => {}
            NodeKind::And(left, right) => {
                self.audit(*left, context, worlds, groups, nested, work)?;
                self.audit(*right, context, worlds, groups, nested, work)?;
            }
            NodeKind::Or(alternatives) => {
                let entered = support(context, worlds, work)? != 0;
                let mut marked = false;
                for alternative in alternatives {
                    work.charge(1)?;
                    marked |= alternative.marked;
                }
                let mut live_mark = false;
                let mut retained = Vec::new();
                let mut suppressed = Vec::new();
                for alternative in alternatives {
                    work.charge(1)?;
                    let child_context = context | alternative.choice;
                    if alternative.marked {
                        live_mark |= support(child_context, worlds, work)? != 0;
                    }
                    if self.nodes[alternative.child].recipe {
                        *nested |= entered;
                        // Charge the copied ID and prospective list allocation
                        // before enqueue, even for unentered source recipes.
                        work.charge(2)?;
                        if marked && !alternative.marked {
                            suppressed.push(alternative.child);
                        } else {
                            retained.push(alternative.child);
                        }
                    }
                    self.audit(
                        alternative.child,
                        child_context,
                        worlds,
                        groups,
                        nested,
                        work,
                    )?;
                }
                let support = if !entered {
                    Support::Unentered
                } else if !marked {
                    Support::Unmarked
                } else if live_mark {
                    Support::MarkedLive
                } else {
                    Support::MarkedEliminated
                };
                work.charge(1)?;
                groups.push(GroupFact {
                    group: id,
                    support,
                    retained,
                    suppressed,
                });
            }
        }
        Ok(())
    }

    fn ordinary(&self, id: Id, work: &mut Work) -> Result<Vec<World>, Error> {
        work.charge(1)?;
        match &self.nodes[id].kind {
            NodeKind::Atom(domain) => {
                work.charge(1)?;
                Ok(vec![World {
                    domain: *domain,
                    lineage: 0,
                }])
            }
            NodeKind::And(left, right) => {
                let left = self.ordinary(*left, work)?;
                let right = self.ordinary(*right, work)?;
                let trials = left.len() * right.len();
                check_worlds(trials)?;
                // Charge all candidate inspections and lineage combinations
                // before allocating, even when every candidate will conflict.
                work.charge(trials * 3)?;
                let mut result = Vec::with_capacity(trials);
                for left in &left {
                    for right in &right {
                        if let Some(domain) = meet(left.domain, right.domain) {
                            result.push(World {
                                domain,
                                lineage: left.lineage | right.lineage,
                            });
                        }
                    }
                }
                Ok(result)
            }
            NodeKind::Or(alternatives) => {
                let mut result = Vec::new();
                for alternative in alternatives {
                    work.charge(1)?;
                    let child = self.ordinary(alternative.child, work)?;
                    check_worlds(result.len() + child.len())?;
                    work.charge(child.len() * 2)?;
                    result.reserve(child.len());
                    for world in child {
                        result.push(World {
                            domain: world.domain,
                            lineage: world.lineage | alternative.choice,
                        });
                    }
                }
                Ok(result)
            }
        }
    }

    fn preference(
        &self,
        id: Id,
        context: u128,
        worlds: &[World],
        work: &mut Work,
    ) -> Result<Association, Error> {
        work.charge(1)?;
        match &self.nodes[id].kind {
            NodeKind::Atom(_) => Ok(Association::Absent),
            NodeKind::And(left, right) => {
                let left = self.preference(*left, context, worlds, work)?;
                let right = self.preference(*right, context, worlds, work)?;
                work.charge(1)?;
                Ok(match (left, right) {
                    (Association::Absent, other) | (other, Association::Absent) => other,
                    (Association::Present(left), Association::Present(right)) => {
                        Association::Present(left & right)
                    }
                })
            }
            NodeKind::Or(alternatives) => {
                let mut marked = false;
                for alternative in alternatives {
                    work.charge(1)?;
                    marked |= alternative.marked;
                }
                let mut live_mark = false;
                let mut present = false;
                let mut selected = 0;
                for alternative in alternatives {
                    work.charge(1)?;
                    if marked && !alternative.marked {
                        // M3 strips this term's top-level recipe, not its
                        // ordinary worlds or any other occurrence's recipe.
                        continue;
                    }
                    let context = context | alternative.choice;
                    let support = support(context, worlds, work)?;
                    if marked {
                        live_mark |= support != 0;
                    }
                    let child = if marked && !self.nodes[alternative.child].recipe {
                        // M1: no syntactic recipe, so prefer its value worlds.
                        Association::Present(support)
                    } else {
                        // M2 is selected syntactically, even if this evaluates
                        // to Absent. A live outer mark must not revive M1.
                        self.preference(alternative.child, context, worlds, work)?
                    };
                    work.charge(1)?;
                    if let Association::Present(values) = child {
                        present = true;
                        selected |= values & support;
                    }
                }
                if marked {
                    Ok(if live_mark {
                        Association::Present(selected)
                    } else {
                        Association::Absent
                    })
                } else if present {
                    Ok(Association::Present(selected))
                } else {
                    Ok(Association::Absent)
                }
            }
        }
    }
}

fn check_worlds(count: usize) -> Result<(), Error> {
    if count > MAX_WORLDS {
        Err(Error::Limit("world table or raw product"))
    } else {
        Ok(())
    }
}

fn meet(left: Domain, right: Domain) -> Option<Domain> {
    match (left, right) {
        (Domain::Top, other) | (other, Domain::Top) => Some(other),
        (Domain::Int, other) | (other, Domain::Int) => Some(other),
        (Domain::Value(left), Domain::Value(right)) if left == right => Some(Domain::Value(left)),
        (Domain::Value(_), Domain::Value(_)) => None,
    }
}

fn support(context: u128, worlds: &[World], work: &mut Work) -> Result<u64, Error> {
    work.charge(worlds.len())?;
    let mut selected = 0;
    for (index, world) in worlds.iter().enumerate() {
        if world.lineage & context == context {
            selected |= 1_u64 << index;
        }
    }
    Ok(selected)
}

fn project(worlds: &[World], selected: Option<u64>, work: &mut Work) -> Result<Vec<Domain>, Error> {
    // Charge each scan/copy, and each comparison/shift during sorted insertion.
    // Only this final report representation discards identical scalar domains.
    work.charge(worlds.len())?;
    let mut domains = Vec::new();
    for (index, world) in worlds.iter().enumerate() {
        if selected.is_some_and(|mask| mask & (1_u64 << index) == 0) {
            continue;
        }
        let mut position = 0;
        while position < domains.len() {
            work.charge(1)?;
            if domains[position] >= world.domain {
                break;
            }
            position += 1;
        }
        if domains.get(position) != Some(&world.domain) {
            work.charge(1 + domains.len() - position)?;
            domains.insert(position, world.domain);
        }
    }
    Ok(domains)
}
