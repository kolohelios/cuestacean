// Test-only context-partitioned finite preference hypothesis. Known falsifications
// are evidence, not permission to normalize away source groups or admit defaults.
use super::default_provenance::{Domain, Error, Expr, Outcome, Preference};

#[derive(Debug, Eq, PartialEq)]
pub struct ContextFact {
    pub choices: Vec<(usize, usize)>,
    pub constraints: Vec<Domain>,
    pub preferred: Preference,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ContextReport {
    pub constraints: Vec<Domain>,
    pub preferred: Preference,
    pub outcome: Outcome,
    pub contexts: Vec<ContextFact>,
    pub work: usize,
}

const MAX_NODES: usize = 128;
const MAX_DEPTH: usize = 16;
const MAX_GROUPS: usize = 16;
const MAX_WIDTH: usize = 16;
const MAX_WORLDS: usize = 64;
const MAX_CONTEXTS: usize = 64;
const MAX_WORK: usize = 4096;

type Choice = (usize, usize);
type Id = usize;

struct Alternative {
    marked: bool,
    child: Id,
    bit: u128,
}

enum Kind {
    Value(i64),
    And(Id, Id),
    Or(Vec<Alternative>),
}

struct Node {
    kind: Kind,
    recipe: bool,
}

struct Source {
    nodes: Vec<Node>,
    // Only alternatives of unmarked groups with recipe-bearing children.
    partition: Vec<(Choice, u128)>,
}

#[derive(Clone, Copy)]
struct World {
    value: i64,
    lineage: u128,
}

struct Context {
    choices: Vec<Choice>,
    worlds: Vec<World>,
}

#[derive(Clone, Copy)]
enum Association {
    Absent,
    Present(u64),
}

struct Work {
    used: usize,
    limit: usize,
}

impl Work {
    fn charge(&mut self, amount: usize) -> Result<(), Error> {
        if amount > self.limit - self.used {
            return Err(Error::Limit("context work"));
        }
        self.used += amount;
        Ok(())
    }
}

// Admission is independently bounded and precedes charged execution. Borrowing
// does not harden destruction of an arbitrarily deep caller-owned Expr.
fn admit(expression: &Expr) -> Result<(), Error> {
    let mut pending = vec![(expression, 1)];
    let mut occurrences = 0;
    let mut groups = 0;
    while let Some((expression, depth)) = pending.pop() {
        occurrences += 1;
        if occurrences > MAX_NODES || depth > MAX_DEPTH {
            return Err(Error::Limit("source topology"));
        }
        match expression {
            Expr::Top | Expr::Int => return Err(Error::Unsupported("finite value source")),
            Expr::Value(_) => {}
            Expr::And(left, right) => {
                if pending.len() + 2 > MAX_NODES {
                    return Err(Error::Limit("source topology"));
                }
                pending.push((right, depth + 1));
                pending.push((left, depth + 1));
            }
            Expr::Or(alternatives) => {
                if alternatives.len() < 2 {
                    return Err(Error::Unsupported("Or arity"));
                }
                groups += 1;
                if groups > MAX_GROUPS || alternatives.len() > MAX_WIDTH {
                    return Err(Error::Limit("source groups or width"));
                }
                if pending.len() + alternatives.len() > MAX_NODES {
                    return Err(Error::Limit("source topology"));
                }
                for (_, child) in alternatives.iter().rev() {
                    pending.push((child, depth + 1));
                }
            }
        }
    }
    Ok(())
}

impl Source {
    fn compile(&mut self, expression: &Expr, next_bit: &mut u32) -> Id {
        let (kind, recipe) = match expression {
            Expr::Value(value) => (Kind::Value(*value), false),
            Expr::And(left, right) => {
                let left = self.compile(left, next_bit);
                let right = self.compile(right, next_bit);
                (
                    Kind::And(left, right),
                    self.nodes[left].recipe || self.nodes[right].recipe,
                )
            }
            Expr::Or(children) => {
                let mut alternatives = Vec::with_capacity(children.len());
                let mut recipe = false;
                for (marked, child) in children {
                    let child = self.compile(child, next_bit);
                    recipe |= *marked || self.nodes[child].recipe;
                    let bit = 1_u128 << *next_bit;
                    *next_bit += 1;
                    alternatives.push(Alternative {
                        marked: *marked,
                        child,
                        bit,
                    });
                }
                (Kind::Or(alternatives), recipe)
            }
            Expr::Top | Expr::Int => unreachable!("admitted finite source"),
        };
        let id = self.nodes.len();
        if let Kind::Or(alternatives) = &kind
            && !alternatives.iter().any(|alternative| alternative.marked)
            && alternatives
                .iter()
                .any(|alternative| self.nodes[alternative.child].recipe)
        {
            for (index, alternative) in alternatives.iter().enumerate() {
                self.partition.push(((id, index), alternative.bit));
            }
        }
        self.nodes.push(Node { kind, recipe });
        id
    }

    fn ordinary(&self, id: Id, work: &mut Work) -> Result<Vec<World>, Error> {
        work.charge(1)?;
        match &self.nodes[id].kind {
            Kind::Value(value) => {
                work.charge(1)?;
                Ok(vec![World {
                    value: *value,
                    lineage: 0,
                }])
            }
            Kind::And(left, right) => {
                let left = self.ordinary(*left, work)?;
                let right = self.ordinary(*right, work)?;
                let trials = left.len() * right.len();
                check_worlds(trials)?;
                work.charge(trials * 3)?;
                let mut result = Vec::with_capacity(trials);
                for left in &left {
                    for right in &right {
                        if left.value == right.value {
                            result.push(World {
                                value: left.value,
                                lineage: left.lineage | right.lineage,
                            });
                        }
                    }
                }
                Ok(result)
            }
            Kind::Or(alternatives) => {
                let mut result = Vec::new();
                for alternative in alternatives {
                    work.charge(1)?;
                    let child = self.ordinary(alternative.child, work)?;
                    check_worlds(result.len() + child.len())?;
                    work.charge(child.len() * 2)?;
                    result.reserve(child.len());
                    for world in child {
                        result.push(World {
                            value: world.value,
                            lineage: world.lineage | alternative.bit,
                        });
                    }
                }
                Ok(result)
            }
        }
    }

    fn partition(&self, worlds: &[World], work: &mut Work) -> Result<Vec<Context>, Error> {
        let mut contexts: Vec<Context> = Vec::new();
        for world in worlds {
            work.charge(1)?;
            let mut choices = Vec::new();
            // Compilation emits partition groups postorder and alternatives in
            // source order, hence signatures already have sorted pair keys.
            for (choice, bit) in &self.partition {
                work.charge(1)?;
                if world.lineage & bit != 0 {
                    work.charge(1)?;
                    choices.push(*choice);
                }
            }
            let mut position = 0;
            while position < contexts.len() {
                match compare_keys(&contexts[position].choices, &choices, work)? {
                    std::cmp::Ordering::Less => position += 1,
                    std::cmp::Ordering::Equal => break,
                    std::cmp::Ordering::Greater => break,
                }
            }
            let existing = if let Some(context) = contexts.get(position) {
                compare_keys(&context.choices, &choices, work)?.is_eq()
            } else {
                false
            };
            if !existing {
                if contexts.len() == MAX_CONTEXTS {
                    return Err(Error::Limit("context count"));
                }
                work.charge(1 + contexts.len() - position)?;
                contexts.insert(
                    position,
                    Context {
                        choices,
                        worlds: Vec::new(),
                    },
                );
            }
            check_worlds(contexts[position].worlds.len() + 1)?;
            work.charge(1)?;
            contexts[position].worlds.push(*world);
        }
        Ok(contexts)
    }

    fn preference(
        &self,
        id: Id,
        path: u128,
        worlds: &[World],
        work: &mut Work,
    ) -> Result<Association, Error> {
        work.charge(1)?;
        match &self.nodes[id].kind {
            Kind::Value(_) => Ok(Association::Absent),
            Kind::And(left, right) => {
                let left = self.preference(*left, path, worlds, work)?;
                let right = self.preference(*right, path, worlds, work)?;
                work.charge(1)?;
                Ok(match (left, right) {
                    (Association::Absent, other) | (other, Association::Absent) => other,
                    (Association::Present(left), Association::Present(right)) => {
                        Association::Present(left & right)
                    }
                })
            }
            Kind::Or(alternatives) => {
                let mut marked = false;
                let mut live = false;
                for alternative in alternatives {
                    work.charge(1)?;
                    marked |= alternative.marked;
                    if alternative.marked {
                        live |= support(path | alternative.bit, worlds, work)? != 0;
                    }
                }
                let mut present = false;
                let mut selected = 0;
                for alternative in alternatives {
                    work.charge(1)?;
                    if live && !alternative.marked {
                        continue; // Original syntactic M3 while marked VALUE support lives.
                    }
                    let path = path | alternative.bit;
                    let support = support(path, worlds, work)?;
                    let child = if live && !self.nodes[alternative.child].recipe {
                        Association::Present(support) // M1, chosen syntactically.
                    } else {
                        // M2 never becomes M1 when its inherited recipe is absent.
                        // With all marks dead, inherit ALL children as an unmarked Or.
                        self.preference(alternative.child, path, worlds, work)?
                    };
                    work.charge(1)?;
                    if let Association::Present(values) = child {
                        present = true;
                        selected |= values & support;
                    }
                }
                // Live M2 yielding Absent is nevertheless present Bottom here.
                // Dead marked groups instead use the inherited union's presence.
                Ok(if (marked && live) || present {
                    Association::Present(selected)
                } else {
                    Association::Absent
                })
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

fn compare_keys(
    left: &[Choice],
    right: &[Choice],
    work: &mut Work,
) -> Result<std::cmp::Ordering, Error> {
    work.charge(1)?;
    for (left, right) in left.iter().zip(right) {
        work.charge(1)?;
        let order = left.cmp(right);
        if !order.is_eq() {
            return Ok(order);
        }
    }
    Ok(left.len().cmp(&right.len()))
}

fn support(path: u128, worlds: &[World], work: &mut Work) -> Result<u64, Error> {
    work.charge(worlds.len())?;
    let mut selected = 0;
    for (index, world) in worlds.iter().enumerate() {
        if world.lineage & path == path {
            selected |= 1_u64 << index;
        }
    }
    Ok(selected)
}

fn insert_domain(domains: &mut Vec<Domain>, domain: Domain, work: &mut Work) -> Result<(), Error> {
    let mut position = 0;
    while position < domains.len() {
        work.charge(1)?;
        match domains[position].cmp(&domain) {
            std::cmp::Ordering::Less => position += 1,
            std::cmp::Ordering::Equal => return Ok(()),
            std::cmp::Ordering::Greater => break,
        }
    }
    work.charge(1 + domains.len() - position)?;
    domains.insert(position, domain);
    Ok(())
}

fn project(worlds: &[World], selected: Option<u64>, work: &mut Work) -> Result<Vec<Domain>, Error> {
    work.charge(worlds.len())?;
    let mut domains = Vec::new();
    for (index, world) in worlds.iter().enumerate() {
        if selected.is_none_or(|mask| mask & (1_u64 << index) != 0) {
            insert_domain(&mut domains, Domain::Value(world.value), work)?;
        }
    }
    Ok(domains)
}

fn preference(domains: Vec<Domain>, present: bool) -> Preference {
    if !present {
        Preference::Absent
    } else if domains.is_empty() {
        Preference::Bottom
    } else {
        Preference::Values(domains)
    }
}

pub fn solve(expression: &Expr, work_limit: usize) -> Result<ContextReport, Error> {
    if work_limit > MAX_WORK {
        return Err(Error::Limit("work allowance"));
    }
    admit(expression)?;
    let mut work = Work {
        used: 0,
        limit: work_limit,
    };
    work.charge(1)?;
    // Compilation follows separately bounded admission; at most 127 choice
    // edges exist in a 128-occurrence tree, so the u128 lineage cannot overflow.
    let mut source = Source {
        nodes: Vec::new(),
        partition: Vec::new(),
    };
    let root = source.compile(expression, &mut 0);
    let worlds = source.ordinary(root, &mut work)?;
    let contexts = source.partition(&worlds, &mut work)?;
    let constraints = project(&worlds, None, &mut work)?;
    let mut facts = Vec::new();
    let mut combined = Vec::new();
    let mut present = false;
    for context in contexts {
        work.charge(1)?;
        let association = source.preference(root, 0, &context.worlds, &mut work)?;
        let constraints = project(&context.worlds, None, &mut work)?;
        let (domains, has_preference) = match association {
            Association::Absent => (Vec::new(), false),
            Association::Present(mask) => (project(&context.worlds, Some(mask), &mut work)?, true),
        };
        present |= has_preference;
        for domain in &domains {
            work.charge(1)?;
            insert_domain(&mut combined, *domain, &mut work)?;
        }
        // Move existing keys/domains; charge materialization without pretending
        // that these logical counters measure allocator bytes or wall time.
        work.charge(1 + context.choices.len() + constraints.len() + domains.len())?;
        facts.push(ContextFact {
            choices: context.choices,
            constraints,
            preferred: preference(domains, has_preference),
        });
    }
    // Union associations FIRST. Bottom contributes a present empty set, never
    // an independent per-context fallback to that context's ordinary worlds.
    work.charge(1)?;
    let selected = if combined.is_empty() {
        &constraints
    } else {
        &combined
    };
    let outcome = if constraints.is_empty() {
        Outcome::Conflict
    } else if let [Domain::Value(value)] = selected.as_slice() {
        Outcome::Concrete(*value)
    } else {
        Outcome::Incomplete
    };
    work.charge(1 + constraints.len() + combined.len() + facts.len())?;
    Ok(ContextReport {
        constraints,
        preferred: preference(combined, present),
        outcome,
        contexts: facts,
        work: work.used,
    })
}
