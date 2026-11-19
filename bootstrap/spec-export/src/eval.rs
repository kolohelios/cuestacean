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
    };
    let terms = doc
        .roots
        .iter()
        .map(|&id| Term { id, groups: vec![] })
        .collect();
    evaluator.demand(terms, 1, false).map(|result| result.value)
}

impl Evaluator<'_> {
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
                    match self.demand(terms.clone(), depth + 1, true) {
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
        let cyclic = document(vec![Expr::Binding(0)]);
        assert_eq!(
            evaluate(&cyclic, 4096),
            Err(Error::Unsupported("binding cycle"))
        );
    }
}
