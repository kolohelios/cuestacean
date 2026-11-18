//! Test-only native graph experiment for #162, not a CUE-hosted solver.
//!
//! This is deliberately not a generic fixed-point engine: aliases refine both
//! endpoints, addition is forward-only, and defaults require a distinct phase.
//! Only the finite Any -> Int -> Value lattice justifies worklist convergence.
use std::collections::{BTreeMap, VecDeque};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    Any,
    Int,
    Value(i64),
    Ref(String),
    Add(Box<Expr>, Box<Expr>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub constraints: Vec<Expr>,
    pub preferred: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Domain {
    Any,
    Int,
    Value(i64),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Concrete,
    Incomplete,
    Cycle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Schedule {
    Fifo,
    Lifo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub constraints: BTreeMap<String, Domain>,
    pub selected: BTreeMap<String, Domain>,
    pub status: Status,
    pub work: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Limit {
    Fields,
    Expressions,
    Edges,
    Depth,
    Work,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Invalid(&'static str),
    UnknownReference(String),
    Conflict,
    Overflow,
    Unsupported(&'static str),
    Limit(Limit),
}

// Compile names once; reference edges never own other arena nodes.
#[derive(Debug)]
enum Tree {
    Any,
    Int,
    Value(i64),
    Ref(usize),
    Add(Box<Tree>, Box<Tree>),
}

#[derive(Debug)]
struct Task {
    target: usize,
    tree: Tree,
}

#[derive(Debug)]
pub struct Graph {
    names: BTreeMap<String, usize>,
    tasks: Vec<Task>,
    watches: Vec<Vec<usize>>,
    // (read field, inside arithmetic); alias dependencies also run backwards.
    dependencies: Vec<Vec<(usize, bool)>>,
    preferred: Option<(usize, i64)>,
    edges: usize,
}

struct Work {
    used: usize,
    limit: usize,
}

impl Work {
    fn tick(&mut self) -> Result<(), Error> {
        if self.used == self.limit {
            return Err(Error::Limit(Limit::Work));
        }
        self.used += 1;
        Ok(())
    }
}

fn meet(left: &Domain, right: &Domain) -> Result<Domain, Error> {
    match (left, right) {
        (Domain::Any, other) | (other, Domain::Any) => Ok(other.clone()),
        (Domain::Int, other) | (other, Domain::Int) => Ok(other.clone()),
        (Domain::Value(a), Domain::Value(b)) if a == b => Ok(left.clone()),
        _ => Err(Error::Conflict),
    }
}

impl Graph {
    pub fn new(mut fields: Vec<Field>) -> Result<Self, Error> {
        if fields.len() > 64 {
            return Err(Error::Limit(Limit::Fields));
        }
        // Validate the entire structure before allocating topology or recursing.
        let (mut expressions, mut edges) = (0, 0);
        for field in &fields {
            if field.name.is_empty() || field.name.len() > 64 {
                return Err(Error::Invalid("field name must contain 1..64 bytes"));
            }
            for root in &field.constraints {
                let mut stack = vec![(root, 1)];
                while let Some((expr, depth)) = stack.pop() {
                    if depth > 16 {
                        return Err(Error::Limit(Limit::Depth));
                    }
                    expressions += 1;
                    if expressions > 256 {
                        return Err(Error::Limit(Limit::Expressions));
                    }
                    match expr {
                        Expr::Ref(_) => {
                            edges += 1;
                            if edges > 128 {
                                return Err(Error::Limit(Limit::Edges));
                            }
                        }
                        Expr::Add(a, b) => {
                            stack.push((a, depth + 1));
                            stack.push((b, depth + 1));
                        }
                        _ => {}
                    }
                }
            }
        }
        fields.sort_by(|a, b| a.name.cmp(&b.name));
        let mut names = BTreeMap::new();
        for (id, field) in fields.iter().enumerate() {
            if names.insert(field.name.clone(), id).is_some() {
                return Err(Error::Invalid("duplicate field"));
            }
        }
        let mut graph = Self {
            watches: vec![Vec::new(); fields.len()],
            dependencies: vec![Vec::new(); fields.len()],
            names,
            tasks: Vec::new(),
            preferred: None,
            edges,
        };
        for (target, field) in fields.into_iter().enumerate() {
            if let Some(value) = field.preferred
                && graph.preferred.replace((target, value)).is_some()
            {
                return Err(Error::Unsupported("multiple defaults"));
            }
            for expr in field.constraints {
                let task = graph.tasks.len();
                let tree = graph.compile(expr, target, task, false)?;
                if let Tree::Ref(source) = &tree {
                    graph.watches[target].push(task);
                    graph.dependencies[*source].push((target, false));
                }
                graph.tasks.push(Task { target, tree });
            }
        }
        for watches in &mut graph.watches {
            watches.sort_unstable();
            watches.dedup();
        }
        Ok(graph)
    }

    fn compile(
        &mut self,
        expr: Expr,
        target: usize,
        task: usize,
        arithmetic: bool,
    ) -> Result<Tree, Error> {
        Ok(match expr {
            // A domain alone cannot retain an unresolved inline computation.
            // Until residual expressions are modeled, reject rather than let a
            // concrete target mask an abstract arithmetic operand.
            Expr::Any | Expr::Int if arithmetic => {
                return Err(Error::Unsupported("abstract arithmetic operand"));
            }
            Expr::Any => Tree::Any,
            Expr::Int => Tree::Int,
            Expr::Value(value) => Tree::Value(value),
            Expr::Ref(name) => {
                let source = self.id(&name).ok_or(Error::UnknownReference(name))?;
                self.watches[source].push(task);
                self.dependencies[target].push((source, arithmetic));
                Tree::Ref(source)
            }
            Expr::Add(a, b) => Tree::Add(
                Box::new(self.compile(*a, target, task, true)?),
                Box::new(self.compile(*b, target, task, true)?),
            ),
        })
    }

    pub fn id(&self, name: &str) -> Option<usize> {
        self.names.get(name).copied()
    }

    pub fn field_count(&self) -> usize {
        self.names.len()
    }

    /// Reference occurrences, before watch deduplication or reverse alias edges.
    pub fn edge_count(&self) -> usize {
        self.edges
    }

    pub fn solve(
        &self,
        schedule: Schedule,
        reverse_initial: bool,
        work_limit: usize,
    ) -> Result<Report, Error> {
        if work_limit > 4096 {
            return Err(Error::Invalid("work limit exceeds 4096"));
        }
        let mut work = Work {
            used: 0,
            limit: work_limit,
        };
        let constraints = self.propagate(schedule, reverse_initial, None, &mut work)?;
        let mut selected = constraints.clone();
        if let Some((id, value)) = self.preferred {
            match constraints[id] {
                Domain::Any => return Err(Error::Invalid("untyped default")),
                Domain::Value(exact) if exact != value => {}
                _ => {
                    selected = self
                        .propagate(schedule, reverse_initial, Some((id, value)), &mut work)
                        .map_err(|error| match error {
                            Error::Conflict => Error::Unsupported("default interaction"),
                            other => other,
                        })?;
                }
            }
        }
        let status = self.classify(&selected, &mut work)?;
        let map = |domains: Vec<Domain>| self.names.keys().cloned().zip(domains).collect();
        Ok(Report {
            constraints: map(constraints),
            selected: map(selected),
            status,
            work: work.used,
        })
    }

    fn propagate(
        &self,
        schedule: Schedule,
        reverse: bool,
        seed: Option<(usize, i64)>,
        work: &mut Work,
    ) -> Result<Vec<Domain>, Error> {
        let mut domains = vec![Domain::Any; self.field_count()];
        if let Some((id, value)) = seed {
            domains[id] = Domain::Value(value);
        }
        let mut queue: VecDeque<_> = (0..self.tasks.len()).collect();
        if reverse {
            queue.make_contiguous().reverse();
        }
        let mut pending = vec![true; self.tasks.len()];
        loop {
            let next = match schedule {
                Schedule::Fifo => queue.pop_front(),
                Schedule::Lifo => queue.pop_back(),
            };
            let Some(id) = next else { break };
            work.tick()?;
            pending[id] = false;
            let task = &self.tasks[id];
            let result = evaluate(&task.tree, &domains, work)?;
            let result = meet(&domains[task.target], &result)?;
            // Equality is symmetric only for a direct reference conjunct.
            let mut updates = vec![(task.target, result.clone())];
            if let Tree::Ref(source) = &task.tree {
                updates.push((*source, result));
            }
            for (field, value) in updates {
                if domains[field] == value {
                    continue;
                }
                domains[field] = value;
                for &watch in &self.watches[field] {
                    work.tick()?;
                    if !pending[watch] {
                        pending[watch] = true;
                        queue.push_back(watch);
                    }
                }
            }
        }
        Ok(domains)
    }

    fn classify(&self, domains: &[Domain], work: &mut Work) -> Result<Status, Error> {
        // Kosaraju on unresolved fields only. Both DFS stacks are bounded by 64;
        // computational edges within an SCC diagnose cycles, not contradiction.
        let n = domains.len();
        let mut seen = vec![false; n];
        let mut order = Vec::new();
        let mut reverse = vec![Vec::new(); n];
        for (from, edges) in self.dependencies.iter().enumerate() {
            work.tick()?;
            for &(to, _) in edges {
                work.tick()?;
                reverse[to].push(from);
            }
        }
        for node in 0..n {
            work.tick()?;
            finish(
                node,
                &self.dependencies,
                domains,
                &mut seen,
                &mut order,
                work,
            )?;
        }
        let mut components = vec![None; n];
        for node in order.into_iter().rev() {
            work.tick()?;
            assign(node, node, &reverse, domains, &mut components, work)?;
        }
        let mut incomplete = false;
        for (from, edges) in self.dependencies.iter().enumerate() {
            work.tick()?;
            if matches!(domains[from], Domain::Value(_)) {
                continue;
            }
            incomplete = true;
            for &(to, arithmetic) in edges {
                work.tick()?;
                if arithmetic && components[from].is_some() && components[from] == components[to] {
                    return Ok(Status::Cycle);
                }
            }
        }
        Ok(if incomplete {
            Status::Incomplete
        } else {
            Status::Concrete
        })
    }
}

fn evaluate(tree: &Tree, domains: &[Domain], work: &mut Work) -> Result<Domain, Error> {
    work.tick()?;
    Ok(match tree {
        Tree::Any => Domain::Any,
        Tree::Int => Domain::Int,
        Tree::Value(value) => Domain::Value(*value),
        Tree::Ref(id) => domains[*id].clone(),
        Tree::Add(a, b) => {
            let a = evaluate(a, domains, work)?;
            let b = evaluate(b, domains, work)?;
            match (a, b) {
                (Domain::Value(a), Domain::Value(b)) => {
                    Domain::Value(a.checked_add(b).ok_or(Error::Overflow)?)
                }
                _ => Domain::Int,
            }
        }
    })
}

fn finish(
    node: usize,
    edges: &[Vec<(usize, bool)>],
    domains: &[Domain],
    seen: &mut [bool],
    order: &mut Vec<usize>,
    work: &mut Work,
) -> Result<(), Error> {
    work.tick()?;
    if seen[node] || matches!(domains[node], Domain::Value(_)) {
        return Ok(());
    }
    seen[node] = true;
    for &(next, _) in &edges[node] {
        work.tick()?;
        if !seen[next] && !matches!(domains[next], Domain::Value(_)) {
            finish(next, edges, domains, seen, order, work)?;
        }
    }
    order.push(node);
    Ok(())
}

fn assign(
    node: usize,
    component: usize,
    edges: &[Vec<usize>],
    domains: &[Domain],
    components: &mut [Option<usize>],
    work: &mut Work,
) -> Result<(), Error> {
    work.tick()?;
    if components[node].is_some() || matches!(domains[node], Domain::Value(_)) {
        return Ok(());
    }
    components[node] = Some(component);
    for &next in &edges[node] {
        work.tick()?;
        if components[next].is_none() && !matches!(domains[next], Domain::Value(_)) {
            assign(next, component, edges, domains, components, work)?;
        }
    }
    Ok(())
}
