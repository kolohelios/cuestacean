//! File-local lexical admission; no scope lookup survives into evaluation.
use crate::model::{Document, Error, Expr, Id, Type};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) struct Scope {
    pub(crate) parent: Option<usize>,
    pub(crate) bindings: BTreeMap<String, usize>,
    pub(crate) fields: BTreeSet<String>,
    pub(crate) excluded: Vec<usize>,
}

pub(crate) struct Binding {
    pub(crate) name: String,
    pub(crate) scope: usize,
    pub(crate) body: Option<Id>,
}

fn ancestor(scopes: &[Scope], ancestor: usize, mut child: usize) -> bool {
    loop {
        if ancestor == child {
            return true;
        }
        match scopes[child].parent {
            Some(parent) => child = parent,
            None => return false,
        }
    }
}

pub(crate) fn resolve(
    doc: &mut Document,
    scopes: &[Scope],
    bindings: &[Binding],
) -> Result<(), Error> {
    // Check both directions, including fields declared after an alias or in a
    // descendant scope. Quoted labels were deliberately not collected here.
    for (binding_id, binding) in bindings.iter().enumerate() {
        for (index, scope) in scopes.iter().enumerate() {
            if !scope.excluded.contains(&binding_id)
                && scope.fields.contains(&binding.name)
                && (ancestor(scopes, binding.scope, index)
                    || ancestor(scopes, index, binding.scope))
            {
                return Err(Error::Unsupported("binding/field collision"));
            }
        }
    }
    for node in &doc.nodes {
        if let Expr::Comprehension {
            scope, key, value, ..
        } = node
        {
            for name in std::iter::once(key).chain(value.iter()) {
                if bindings.iter().enumerate().any(|(id, binding)| {
                    binding.name == *name
                        && ancestor(scopes, binding.scope, *scope)
                        && !scopes[*scope].excluded.contains(&id)
                }) || scopes.iter().enumerate().any(|(id, other)| {
                    other.fields.contains(name)
                        && (ancestor(scopes, *scope, id) || ancestor(scopes, id, *scope))
                }) {
                    return Err(Error::Unsupported("iterator declaration collision"));
                }
            }
        }
    }
    let mut used = vec![false; bindings.len()];
    for node in &mut doc.nodes {
        match node {
            Expr::Name {
                scope,
                excluded,
                name,
            } => {
                let mut current = Some(*scope);
                let mut found = None;
                while let Some(index) = current {
                    let scope = &scopes[index];
                    if scope.fields.contains(name) {
                        return Err(Error::Unsupported("field value reference"));
                    }
                    if let Some(&binding) = scope.bindings.get(name)
                        && !excluded.contains(&binding)
                    {
                        used[binding] = true;
                        found = Some(Expr::Binding(
                            bindings[binding]
                                .body
                                .ok_or(Error::Syntax("missing binding body"))?,
                        ));
                        break;
                    }
                    current = scope.parent;
                }
                *node = match found {
                    Some(expr) => expr,
                    None => match name.as_str() {
                        "string" => Expr::Type(Type::String),
                        "int" => Expr::Type(Type::Int),
                        "bool" => Expr::Type(Type::Bool),
                        "bytes" | "number" | "float" | "uint" | "int8" | "int16" | "int32"
                        | "int64" | "uint8" | "uint16" | "uint32" | "uint64" => {
                            return Err(Error::Unsupported("predeclared type"));
                        }
                        _ => return Err(Error::Unknown(name.clone())),
                    },
                };
            }
            Expr::Ref(name)
            | Expr::Projection {
                definition: name, ..
            } if !doc.definitions.contains_key(name) => {
                return Err(Error::Unknown(name.clone()));
            }
            _ => {}
        }
    }
    reject_cycles(doc)?;
    if used.contains(&false) {
        return Err(Error::Unsupported("unused binding"));
    }
    Ok(())
}

fn reject_cycles(doc: &Document) -> Result<(), Error> {
    // Iterative Kosaraju over the finite syntax graph. Looking only for a DFS
    // back-edge labelled Binding misses cycles through nested data/definitions.
    let mut edges = vec![Vec::new(); doc.nodes.len()];
    let mut reverse = vec![Vec::new(); doc.nodes.len()];
    for (id, node) in doc.nodes.iter().enumerate() {
        match node {
            Expr::Binding(body) | Expr::Marked(body) => edges[id].push(*body),
            Expr::Comprehension { source, .. } => edges[id].push(*source),
            Expr::Ref(name)
            | Expr::Projection {
                definition: name, ..
            } => {
                edges[id].push(doc.definitions[name]);
            }
            Expr::And(a, b) | Expr::Or(a, b) | Expr::Concat(a, b) => {
                edges[id].extend([*a, *b]);
            }
            Expr::Struct(fields) => edges[id].extend(fields.iter().map(|(_, id)| *id)),
            Expr::List(items, tail) => edges[id].extend(items.iter().chain(tail).copied()),
            _ => {}
        }
        for &child in &edges[id] {
            reverse[child].push(id);
        }
    }
    let mut seen = vec![false; doc.nodes.len()];
    let mut order = Vec::new();
    for root in 0..doc.nodes.len() {
        if seen[root] {
            continue;
        }
        seen[root] = true;
        let mut stack = vec![(root, 0)];
        while let Some((id, next)) = stack.last_mut() {
            if let Some(&child) = edges[*id].get(*next) {
                *next += 1;
                if !seen[child] {
                    seen[child] = true;
                    stack.push((child, 0));
                }
            } else {
                order.push(*id);
                stack.pop();
            }
        }
    }
    let mut component = vec![usize::MAX; doc.nodes.len()];
    for root in order.into_iter().rev() {
        if component[root] != usize::MAX {
            continue;
        }
        component[root] = root;
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            for &parent in &reverse[id] {
                if component[parent] == usize::MAX {
                    component[parent] = root;
                    stack.push(parent);
                }
            }
        }
    }
    for (id, node) in doc.nodes.iter().enumerate() {
        if let Expr::Binding(body) = node
            && component[id] == component[*body]
        {
            return Err(Error::Unsupported("cyclic binding dependency"));
        }
    }
    Ok(())
}
