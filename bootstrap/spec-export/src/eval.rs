//! Bounded native bootstrap constraint evaluation, not the production CUE solver.
use crate::model::{Document, Error, Expr, Id, Type};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Shape {
    Scalar(Type),
    Struct,
    List,
}

#[derive(Clone)]
struct Term {
    id: Id,
    // A closure identity is local to a definition instantiation, not its template.
    groups: Vec<usize>,
}

#[derive(Clone)]
enum Task {
    Visit(Term),
    Leave(Id),
}

#[derive(Clone, Default)]
struct State {
    tasks: Vec<Task>,
    atoms: Vec<Term>,
    active: Vec<Id>,
}

// JSON projection alone loses container bounds. Entries follow canonical JSON
// container preorder; scalars contribute no entry. Closure identities and tail
// syntax are deliberately not part of this bounded export-readiness witness.
struct Materialized {
    value: Value,
    closed: Vec<bool>,
}

struct Evaluator<'a> {
    doc: &'a Document,
    remaining: usize,
    values: usize,
    strings: usize,
    next_group: usize,
    admitted: bool,
}

pub(crate) fn evaluate(doc: &Document, work_limit: usize) -> Result<Value, Error> {
    if work_limit > 4096 {
        return Err(Error::Limit("evaluation steps"));
    }
    let mut evaluator = Evaluator {
        doc,
        remaining: work_limit,
        values: 0,
        strings: 0,
        next_group: 0,
        admitted: false,
    };
    if doc.marked {
        evaluator.admit()?;
        evaluator.admitted = true;
        evaluator.step(doc.roots.len())?;
    }
    let terms = doc
        .roots
        .iter()
        .map(|&id| Term { id, groups: vec![] })
        .collect();
    evaluator.demand(terms, 1, false).map(|result| result.value)
}

impl Evaluator<'_> {
    // This certificate is invocation-local and is set only after the entire
    // syntax arena and every root occurrence have passed admission.
    fn admit(&mut self) -> Result<(), Error> {
        self.step(1)?;
        self.step(self.doc.definitions.len() + self.doc.lets)?;
        if !self.doc.definitions.is_empty() || self.doc.lets != 0 {
            return Err(Error::Unsupported("tagged declarations"));
        }
        let mut ors = 0;
        let mut marks = 0;
        for node in &self.doc.nodes {
            self.step(1)?;
            match node {
                Expr::Literal(value) => {
                    Self::literal_type(value)?;
                }
                Expr::Type(_) | Expr::Top | Expr::And(_, _) => {}
                Expr::Or(_, _) => ors += 1,
                Expr::Marked(_) => marks += 1,
                Expr::Struct(fields) if fields.len() <= 64 => {}
                Expr::List(items, None) if items.len() <= 64 => {}
                _ => return Err(Error::Unsupported("tagged document profile")),
            }
        }
        if ors != 1 || marks != 1 {
            return Err(Error::Unsupported("one tagged choice required"));
        }
        let mut choices = 0;
        for &root in &self.doc.roots {
            self.step(1)?;
            self.context(root, 1, true, true, &mut choices)?;
        }
        if choices != 1 {
            return Err(Error::Unsupported("one tagged occurrence required"));
        }
        Ok(())
    }

    fn context(
        &mut self,
        id: Id,
        depth: usize,
        root: bool,
        field_path: bool,
        choices: &mut usize,
    ) -> Result<(), Error> {
        self.step(1)?;
        if depth > 32 {
            return Err(Error::Limit("syntax depth"));
        }
        match &self.doc.nodes[id] {
            Expr::And(a, b) => {
                self.context(*a, depth + 1, root, field_path, choices)?;
                self.context(*b, depth + 1, root, field_path, choices)
            }
            Expr::Struct(fields) => {
                for (_, child) in fields {
                    self.step(1)?;
                    self.context(*child, depth + 1, false, field_path, choices)?;
                }
                Ok(())
            }
            Expr::Or(a, b) if !root && field_path => {
                *choices += 1;
                if *choices != 1 {
                    return Err(Error::Unsupported("multiple tagged occurrences"));
                }
                self.step(2)?;
                let (p, q) = match (&self.doc.nodes[*a], &self.doc.nodes[*b]) {
                    (Expr::Marked(_), Expr::Marked(_)) => {
                        return Err(Error::Unsupported("two marked alternatives"));
                    }
                    (Expr::Marked(p), _) => (*p, *b),
                    (_, Expr::Marked(p)) => (*p, *a),
                    _ => return Err(Error::Unsupported("missing immediate mark")),
                };
                self.step(2)?;
                let (p_fields, q_fields) = match (&self.doc.nodes[p], &self.doc.nodes[q]) {
                    // One literal preference under default-free context needs no
                    // disjointness witness, even when Q overlaps or equals P.
                    (Expr::Literal(_), Expr::Literal(_) | Expr::Type(_) | Expr::Top) => {
                        // Preserve syntax depth on the marked side, including
                        // its wrapper, and charge both constructor inspections.
                        self.constructor(p, depth + 2, true)?;
                        return self.constructor(q, depth + 1, false);
                    }
                    (Expr::Struct(p_fields), Expr::Struct(q_fields)) => (p_fields, q_fields),
                    _ => return Err(Error::Unsupported("tagged struct alternatives")),
                };
                // Include the mark wrapper in syntax depth, on its own side.
                self.constructor(p, depth + 2, true)?;
                self.constructor(q, depth + 1, false)?;
                let mut disjoint = false;
                for (p_name, p_id) in p_fields {
                    self.step(1)?;
                    for (q_name, q_id) in q_fields {
                        self.step(1)?;
                        if p_name == q_name {
                            self.step(2)?;
                            if let (Expr::Literal(p), Expr::Literal(q)) =
                                (&self.doc.nodes[*p_id], &self.doc.nodes[*q_id])
                            {
                                disjoint |= p != q;
                            }
                        }
                    }
                }
                if !disjoint {
                    return Err(Error::Unsupported("disjoint literal witness required"));
                }
                Ok(())
            }
            Expr::List(items, None) if !root => {
                for child in items {
                    self.step(1)?;
                    self.context(*child, depth + 1, false, false, choices)?;
                }
                Ok(())
            }
            Expr::Literal(_) | Expr::Type(_) | Expr::Top if !root => Ok(()),
            _ => Err(Error::Unsupported("tagged context profile")),
        }
    }

    fn constructor(&mut self, id: Id, depth: usize, concrete: bool) -> Result<(), Error> {
        self.step(1)?;
        if depth > 32 {
            return Err(Error::Limit("syntax depth"));
        }
        match &self.doc.nodes[id] {
            Expr::Literal(_) => Ok(()),
            Expr::Type(_) | Expr::Top if !concrete => Ok(()),
            Expr::Struct(fields) => {
                let mut names = BTreeSet::new();
                for (name, child) in fields {
                    self.step(1)?;
                    if !names.insert(name.as_str()) {
                        return Err(Error::Unsupported("duplicate alternative field"));
                    }
                    self.constructor(*child, depth + 1, concrete)?;
                }
                Ok(())
            }
            Expr::List(items, None) => {
                for child in items {
                    self.step(1)?;
                    self.constructor(*child, depth + 1, concrete)?;
                }
                Ok(())
            }
            _ => Err(Error::Unsupported("tagged constructor profile")),
        }
    }

    fn copy_terms(&mut self, terms: &[Term]) -> Result<Vec<Term>, Error> {
        self.step(terms.len())?;
        for term in terms {
            self.step(term.groups.len())?;
        }
        Ok(terms.to_vec())
    }

    fn flatten(&mut self, terms: &[Term]) -> Result<Vec<Term>, Error> {
        let mut pending = self.copy_terms(terms)?;
        let mut flat = Vec::new();
        while let Some(term) = pending.pop() {
            self.step(1)?;
            match self.doc.nodes[term.id] {
                Expr::And(a, b) => {
                    self.step(2)?;
                    pending.push(Term {
                        id: b,
                        groups: vec![],
                    });
                    pending.push(Term {
                        id: a,
                        groups: vec![],
                    });
                }
                Expr::Top => {}
                _ => flat.push(term),
            }
        }
        Ok(flat)
    }

    fn choice(&mut self, terms: &[Term]) -> Result<Option<(usize, Id, Id)>, Error> {
        for (index, term) in terms.iter().enumerate() {
            self.step(1)?;
            if let Expr::Or(a, b) = self.doc.nodes[term.id] {
                self.step(2)?;
                let pair = match self.doc.nodes[a] {
                    Expr::Marked(p) => Some((index, p, b)),
                    _ => match self.doc.nodes[b] {
                        Expr::Marked(p) => Some((index, p, a)),
                        _ => None,
                    },
                };
                return Ok(pair);
            }
        }
        Ok(None)
    }

    fn substitute(&mut self, terms: &[Term], index: usize, id: Id) -> Result<Vec<Term>, Error> {
        let mut trial = self.copy_terms(terms)?;
        trial[index].id = id;
        Ok(trial)
    }

    // Existence, not export readiness: abstract leaves have completions but do
    // not supply concrete values. Every required child participates in the meet.
    fn possible(&mut self, terms: &[Term], depth: usize) -> Result<bool, Error> {
        self.step(1)?;
        if depth > 32 {
            return Err(Error::Limit("output depth"));
        }
        let terms = self.flatten(terms)?;
        self.step(terms.len())?;
        if let Some((index, p, q)) = self.choice(&terms)? {
            let p = self.substitute(&terms, index, p)?;
            let q = self.substitute(&terms, index, q)?;
            let p_ok = self.possible(&p, depth)?;
            let q_ok = self.possible(&q, depth)?;
            return Ok(p_ok || q_ok);
        }
        let mut shape = None;
        let mut literal = None;
        let mut fields: BTreeMap<&str, Vec<Term>> = BTreeMap::new();
        let mut lists = Vec::new();
        let mut length = None;
        for term in &terms {
            self.step(1)?;
            let kind = match &self.doc.nodes[term.id] {
                Expr::Literal(value) => {
                    if literal.is_some_and(|old| old != value) {
                        return Ok(false);
                    }
                    literal = Some(value);
                    Shape::Scalar(Self::literal_type(value)?)
                }
                Expr::Type(ty) => Shape::Scalar(*ty),
                Expr::Struct(entries) => {
                    for (name, id) in entries {
                        self.step(1)?;
                        fields.entry(name).or_default().push(Term {
                            id: *id,
                            groups: vec![],
                        });
                    }
                    Shape::Struct
                }
                Expr::List(items, None) => {
                    if length.is_some_and(|old| old != items.len()) {
                        return Ok(false);
                    }
                    length = Some(items.len());
                    self.step(1)?;
                    lists.push(items);
                    Shape::List
                }
                _ => return Err(Error::Unsupported("unadmitted feasibility term")),
            };
            if shape.is_some_and(|old| old != kind) {
                return Ok(false);
            }
            shape = Some(kind);
        }
        for children in fields.values() {
            self.step(1)?;
            if !self.possible(children, depth + 1)? {
                return Ok(false);
            }
        }
        for index in 0..length.unwrap_or(0) {
            self.step(1)?;
            let mut children = Vec::new();
            for items in &lists {
                self.step(1)?;
                children.push(Term {
                    id: items[index],
                    groups: vec![],
                });
            }
            if !self.possible(&children, depth + 1)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn step(&mut self, count: usize) -> Result<(), Error> {
        self.remaining = self
            .remaining
            .checked_sub(count)
            .ok_or(Error::Limit("evaluation steps"))?;
        Ok(())
    }

    fn text(&mut self, text: &str) -> Result<String, Error> {
        self.strings = self
            .strings
            .checked_add(text.len())
            .ok_or(Error::Limit("copied strings"))?;
        if self.strings > 65536 {
            return Err(Error::Limit("copied strings"));
        }
        Ok(text.to_owned())
    }

    fn value(&mut self) -> Result<(), Error> {
        self.values += 1;
        if self.values > 2048 {
            return Err(Error::Limit("materialized values"));
        }
        Ok(())
    }

    fn literal_type(value: &Value) -> Result<Type, Error> {
        match value {
            Value::Bool(_) => Ok(Type::Bool),
            Value::String(_) => Ok(Type::String),
            Value::Number(n) if n.as_i64().is_some() => Ok(Type::Int),
            _ => Err(Error::Unsupported("literal type")),
        }
    }

    fn demand(
        &mut self,
        terms: Vec<Term>,
        depth: usize,
        shallow: bool,
    ) -> Result<Materialized, Error> {
        self.step(1)?;
        if depth > 32 {
            return Err(Error::Limit("output depth"));
        }
        let terms = if self.admitted {
            let terms = self.flatten(&terms)?;
            if shallow {
                return if self.possible(&terms, depth)? {
                    Err(Error::Incomplete)
                } else {
                    Err(Error::Conflict)
                };
            }
            if let Some((index, p, q)) = self.choice(&terms)? {
                let preferred = self.substitute(&terms, index, p)?;
                let ordinary = self.substitute(&terms, index, q)?;
                let p_ok = self.possible(&preferred, depth)?;
                let q_ok = self.possible(&ordinary, depth)?;
                return if p_ok {
                    self.demand(preferred, depth, false)
                } else if q_ok {
                    self.demand(ordinary, depth, false)
                } else {
                    Err(Error::Conflict)
                };
            }
            terms
        } else {
            terms
        };
        if self.admitted {
            self.step(terms.len())?;
        }
        let mut pending = vec![State {
            tasks: terms.into_iter().rev().map(Task::Visit).collect(),
            ..State::default()
        }];
        let mut result: Option<Materialized> = None;
        let mut incomplete = false;
        let mut distinct = false;
        while let Some(mut state) = pending.pop() {
            let mut split = false;
            while let Some(task) = state.tasks.pop() {
                self.step(1)?;
                let term = match task {
                    Task::Leave(id) => {
                        let exited = state.active.pop();
                        debug_assert_eq!(exited, Some(id));
                        continue;
                    }
                    Task::Visit(term) => term,
                };
                match &self.doc.nodes[term.id] {
                    Expr::And(left, right) => {
                        state.tasks.push(Task::Visit(Term {
                            id: *right,
                            groups: term.groups.clone(),
                        }));
                        state.tasks.push(Task::Visit(Term {
                            id: *left,
                            groups: term.groups,
                        }));
                    }
                    Expr::Or(left, right) => {
                        // Clone only bounded syntax IDs/closure identities, never trial JSON.
                        self.step(state.tasks.len() + state.atoms.len() + state.active.len() + 1)?;
                        let mut alternative = state.clone();
                        alternative.tasks.push(Task::Visit(Term {
                            id: *right,
                            groups: term.groups.clone(),
                        }));
                        state.tasks.push(Task::Visit(Term {
                            id: *left,
                            groups: term.groups,
                        }));
                        pending.push(alternative);
                        pending.push(std::mem::take(&mut state));
                        split = true;
                        break;
                    }
                    Expr::Top | Expr::Marked(_) => {
                        return Err(Error::Unsupported("unadmitted preference syntax"));
                    }
                    Expr::Name { .. } => {
                        return Err(Error::Unsupported("unresolved binding name"));
                    }
                    Expr::Binding(id) => {
                        self.step(1)?;
                        if state.active.contains(id) {
                            return Err(Error::Unsupported("binding cycle"));
                        }
                        state.active.push(*id);
                        state.tasks.push(Task::Leave(*id));
                        state.tasks.push(Task::Visit(Term {
                            id: *id,
                            groups: term.groups,
                        }));
                    }
                    Expr::Ref(name) => {
                        self.step(1)?;
                        let id = *self
                            .doc
                            .definitions
                            .get(name)
                            .ok_or_else(|| Error::Unknown(name.clone()))?;
                        if state.active.contains(&id) {
                            return Err(Error::Unsupported("unproductive recursive definition"));
                        }
                        let mut groups = term.groups;
                        groups.push(self.next_group);
                        self.next_group += 1;
                        state.active.push(id);
                        state.tasks.push(Task::Leave(id));
                        state.tasks.push(Task::Visit(Term { id, groups }));
                    }
                    _ => state.atoms.push(term),
                }
            }
            if split {
                continue;
            }
            match self.materialize(&state.atoms, depth, shallow) {
                Ok(value) => {
                    if let Some(previous) = &result {
                        // Charge both JSON and witness traversal even when an earlier
                        // difference could short-circuit the actual comparison.
                        self.inspect(&previous.value)?;
                        self.inspect(&value.value)?;
                        self.step(previous.closed.len() + value.closed.len())?;
                        distinct |=
                            previous.value != value.value || previous.closed != value.closed;
                    } else {
                        result = Some(value);
                    }
                }
                Err(Error::Conflict) => {}
                Err(Error::Incomplete) => incomplete = true,
                Err(error) => return Err(error),
            }
        }
        if incomplete || distinct {
            Err(Error::Incomplete)
        } else {
            result.ok_or(Error::Conflict)
        }
    }

    fn inspect(&mut self, value: &Value) -> Result<(), Error> {
        self.step(1)?;
        match value {
            Value::Array(items) => {
                for item in items {
                    self.inspect(item)?;
                }
            }
            Value::Object(fields) => {
                for value in fields.values() {
                    self.inspect(value)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn materialize(
        &mut self,
        atoms: &[Term],
        depth: usize,
        shallow: bool,
    ) -> Result<Materialized, Error> {
        // First inspect every shallow constraint. A missing deep field must not
        // conceal an incompatible tag, type, length, or closed field admission.
        let mut shape = None;
        let mut literal: Option<&Value> = None;
        let mut unequal = vec![];
        let mut fields: BTreeMap<&str, Vec<Term>> = BTreeMap::new();
        let mut admitted: BTreeMap<usize, BTreeSet<&str>> = BTreeMap::new();
        let mut lists = vec![];
        let mut length = None;
        let mut minimum = 0;
        for term in atoms {
            self.step(1)?;
            let kind = match &self.doc.nodes[term.id] {
                Expr::Literal(value) => {
                    let ty = Self::literal_type(value)?;
                    if literal.is_some_and(|old| old != value) {
                        return Err(Error::Conflict);
                    }
                    literal = Some(value);
                    Shape::Scalar(ty)
                }
                Expr::Type(ty) => Shape::Scalar(*ty),
                Expr::NotEqual(value) => {
                    let ty = Self::literal_type(value)?;
                    unequal.push(value);
                    Shape::Scalar(ty)
                }
                Expr::Struct(entries) => {
                    for (name, id) in entries {
                        self.step(1)?;
                        fields.entry(name).or_default().push(Term {
                            id: *id,
                            groups: term.groups.clone(),
                        });
                    }
                    for group in &term.groups {
                        self.step(1)?;
                        let allowed = admitted.entry(*group).or_default();
                        for (name, _) in entries {
                            self.step(1)?;
                            allowed.insert(name.as_str());
                        }
                    }
                    Shape::Struct
                }
                Expr::List(items, tail) => {
                    minimum = minimum.max(items.len());
                    if tail.is_none() {
                        if length.is_some_and(|old| old != items.len()) {
                            return Err(Error::Conflict);
                        }
                        length = Some(items.len());
                    }
                    lists.push((term, items, tail));
                    Shape::List
                }
                _ => unreachable!("conjunctions and references were flattened"),
            };
            if shape.is_some_and(|old| old != kind) {
                return Err(Error::Conflict);
            }
            shape = Some(kind);
        }
        if let Some(value) = literal {
            for excluded in unequal {
                self.step(1)?;
                if excluded == value {
                    return Err(Error::Conflict);
                }
            }
            if shallow {
                return Err(Error::Incomplete);
            }
            self.value()?;
            let value = match value {
                Value::String(text) => Value::String(self.text(text)?),
                Value::Number(number) => Value::Number(number.clone()),
                Value::Bool(value) => Value::Bool(*value),
                _ => unreachable!(),
            };
            return Ok(Materialized {
                value,
                closed: vec![],
            });
        }
        match shape {
            Some(Shape::Struct) => {
                for allowed in admitted.values() {
                    for name in fields.keys() {
                        self.step(1)?;
                        if !allowed.contains(name) {
                            return Err(Error::Conflict);
                        }
                    }
                }
                if shallow {
                    return Err(Error::Incomplete);
                }
                // Inspect every immediate field before recursive demands, so a
                // tag mismatch prunes a branch before a deep payload is visited.
                // No field label receives privileged treatment.
                let mut deferred = None;
                for terms in fields.values() {
                    let trial = if self.admitted {
                        self.copy_terms(terms)?
                    } else {
                        terms.clone()
                    };
                    match self.demand(trial, depth + 1, true) {
                        Err(Error::Incomplete) | Ok(_) => {}
                        Err(Error::Conflict) => return Err(Error::Conflict),
                        Err(error @ Error::Limit(_)) => return Err(error),
                        Err(error) => {
                            deferred.get_or_insert(error);
                        }
                    }
                }
                if let Some(error) = deferred {
                    return Err(error);
                }
                self.value()?;
                self.step(1)?;
                // Even a group admitting no fields closes an empty struct.
                let mut closed = vec![!admitted.is_empty()];
                let mut output = Map::new();
                let mut incomplete = false;
                for (name, terms) in fields {
                    match self.demand(terms, depth + 1, false) {
                        Ok(value) => {
                            self.step(value.closed.len())?;
                            closed.extend(value.closed);
                            output.insert(self.text(name)?, value.value);
                        }
                        Err(Error::Incomplete) => incomplete = true,
                        Err(error) => return Err(error),
                    }
                }
                if incomplete {
                    return Err(Error::Incomplete);
                }
                Ok(Materialized {
                    value: Value::Object(output),
                    closed,
                })
            }
            Some(Shape::List) => {
                // Project only the known prefix of an open list, retaining every
                // prefix/tail conjunct until all demanded positions are met.
                // A fixed conjunct selects the exact length instead.
                let count = length.unwrap_or(minimum);
                for (_, items, _) in &lists {
                    self.step(1)?;
                    if items.len() > count {
                        return Err(Error::Conflict);
                    }
                }
                if shallow {
                    return Err(Error::Incomplete);
                }
                self.value()?;
                self.step(1)?;
                // Use the final normalized length, not an earlier open projection.
                let mut closed = vec![length.is_some()];
                let mut output = Vec::new();
                let mut incomplete = false;
                for index in 0..count {
                    let mut terms = vec![];
                    for (term, items, tail) in &lists {
                        self.step(1)?;
                        if let Some(id) = items.get(index).or(tail.as_ref()) {
                            terms.push(Term {
                                id: *id,
                                groups: term.groups.clone(),
                            });
                        }
                    }
                    match self.demand(terms, depth + 1, false) {
                        Ok(value) => {
                            self.step(value.closed.len())?;
                            closed.extend(value.closed);
                            output.push(value.value);
                        }
                        Err(Error::Incomplete) => incomplete = true,
                        Err(error) => return Err(error),
                    }
                }
                if incomplete {
                    return Err(Error::Incomplete);
                }
                Ok(Materialized {
                    value: Value::Array(output),
                    closed,
                })
            }
            _ => Err(Error::Incomplete),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_unresolved_or_cyclic_binding_nodes_fail_defensively() {
        let document = |nodes| Document {
            nodes,
            definitions: BTreeMap::new(),
            roots: vec![0],
            marked: false,
            lets: 0,
        };
        let unresolved = document(vec![Expr::Name {
            scope: 0,
            excluded: vec![],
            name: "x".into(),
        }]);
        assert_eq!(
            evaluate(&unresolved, 4096),
            Err(Error::Unsupported("unresolved binding name"))
        );
        for node in [Expr::Top, Expr::Marked(0)] {
            assert_eq!(
                evaluate(&document(vec![node]), 4096),
                Err(Error::Unsupported("unadmitted preference syntax"))
            );
        }
        let cyclic = document(vec![Expr::Binding(0)]);
        assert_eq!(
            evaluate(&cyclic, 4096),
            Err(Error::Unsupported("binding cycle"))
        );
    }
}
