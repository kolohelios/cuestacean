//! Original literal selection and static string computation, not field solving.
use crate::model::{Document, Error, Expr, Id};
use std::collections::BTreeSet;

#[derive(Clone, Copy)]
enum Operand {
    Selected(Id),
    Joined(usize),
}

enum Replacement {
    Selection {
        id: Id,
        selected: Id,
        container: bool,
    },
    Concatenation {
        id: Id,
        left: Id,
        right: Id,
        bytes: usize,
    },
}

// Called exactly once per original operand, under its inspection charge. The
// original kind, not a rewritten/evaluated value, determines eligibility.
fn operand_length(doc: &Document, metadata: &[Option<Operand>], id: Id) -> Result<usize, Error> {
    match (&doc.nodes[id], metadata[id]) {
        (Expr::Literal(serde_json::Value::String(text)), _) => Ok(text.len()),
        (Expr::Projection { .. }, Some(Operand::Selected(selected))) => {
            match &doc.nodes[selected] {
                Expr::Literal(serde_json::Value::String(text)) => Ok(text.len()),
                _ => Err(Error::Unsupported("string concatenation operand")),
            }
        }
        (Expr::Concat(_, _), Some(Operand::Joined(bytes))) => Ok(bytes),
        _ => Err(Error::Unsupported("string concatenation operand")),
    }
}

fn lower_concatenations(doc: &mut Document, work: usize) -> Result<(), Error> {
    if work > 4096 {
        return Err(Error::Limit("evaluation steps"));
    }
    let mut remaining = work
        .checked_sub(doc.preparation_steps)
        .ok_or(Error::Limit("evaluation steps"))?;
    let mut step = || {
        remaining = remaining
            .checked_sub(1)
            .ok_or(Error::Limit("evaluation steps"))?;
        Ok::<(), Error>(())
    };
    step()?; // entry BEFORE metadata allocation or any original-arena inspection
    if doc.marked || doc.comprehensions != 0 {
        return Err(Error::Unsupported("string concatenation document profile"));
    }
    let mut metadata = vec![None; doc.nodes.len()];
    let mut replacements = Vec::new();
    for (id, node) in doc.nodes.iter().enumerate() {
        step()?;
        match node {
            Expr::Projection { definition, field } => {
                step()?; // projection descriptor BEFORE allocation
                step()?; // original definition lookup
                let body = *doc
                    .definitions
                    .get(definition)
                    .ok_or_else(|| Error::Unknown(definition.clone()))?;
                step()?; // original target root inspection
                let Expr::Struct(fields) = &doc.nodes[body] else {
                    return Err(Error::Unsupported("literal projection struct required"));
                };
                let (selected, container) = literal_fields(doc, fields, Some(field), &mut step)?
                    .ok_or_else(|| Error::Unknown(format!("{definition}.{field}")))?;
                metadata[id] = Some(Operand::Selected(selected));
                replacements.push(Replacement::Selection {
                    id,
                    selected,
                    container,
                });
            }
            Expr::Concat(left, right) => {
                step()?; // concat descriptor BEFORE allocation
                step()?; // EACH original operand kind/metadata/length inspection
                let left_bytes = operand_length(doc, &metadata, *left)?;
                step()?;
                let right_bytes = operand_length(doc, &metadata, *right)?;
                let bytes = left_bytes
                    .checked_add(right_bytes)
                    .ok_or(Error::Limit("copied strings"))?;
                metadata[id] = Some(Operand::Joined(bytes));
                replacements.push(Replacement::Concatenation {
                    id,
                    left: *left,
                    right: *right,
                    bytes,
                });
            }
            Expr::Literal(value)
                if value.is_string() || value.is_boolean() || value.as_i64().is_some() => {}
            Expr::Struct(_) | Expr::List(_, None) | Expr::Ref(_) | Expr::Binding(_) => {}
            _ if doc.projections == 0 => {}
            _ => return Err(Error::Unsupported("scalar projection document profile")),
        }
    }
    // All operands and WHOLE targets passed against the SAME original arena.
    // Descriptors retain arena order: every concat child precedes its parent.
    let mut strings = doc.preparation_strings;
    for replacement in replacements {
        match replacement {
            Replacement::Selection {
                id,
                selected,
                container,
            } => {
                step()?;
                doc.nodes[id] = if container {
                    Expr::SelectedContainer(selected)
                } else {
                    Expr::Binding(selected)
                };
            }
            Replacement::Concatenation {
                id,
                left,
                right,
                bytes,
            } => {
                step()?; // result allocation, including an empty result
                strings = strings
                    .checked_add(bytes)
                    .filter(|n| *n <= 65536)
                    .ok_or(Error::Limit("copied strings"))?;
                let mut text = String::with_capacity(bytes);
                for child in [left, right] {
                    step()?; // EACH borrowed append, even empty
                    let selected = match metadata[child] {
                        Some(Operand::Selected(selected)) => selected,
                        _ => child,
                    };
                    let Expr::Literal(serde_json::Value::String(operand)) = &doc.nodes[selected]
                    else {
                        unreachable!("validated string operand in child-before-parent order")
                    };
                    text.push_str(operand);
                }
                step()?; // replacement BEFORE mutation; no new syntax nodes
                doc.nodes[id] = Expr::Literal(text.into());
            }
        }
    }
    doc.preparation_steps = work - remaining;
    doc.preparation_strings = strings;
    Ok(())
}

// The entering edge has already charged this node's kind inspection. Recursive
// descent follows only original syntax children, whose depth is parser-bounded.
fn literal_tree(
    doc: &Document,
    id: Id,
    step: &mut impl FnMut() -> Result<(), Error>,
) -> Result<bool, Error> {
    match &doc.nodes[id] {
        Expr::Literal(value)
            if value.is_string() || value.is_boolean() || value.as_i64().is_some() =>
        {
            Ok(false)
        }
        Expr::Struct(fields) => {
            literal_fields(doc, fields, None, step)?;
            Ok(true)
        }
        Expr::List(items, None) => {
            for &child in items {
                step()?; // item edge and child kind, BEFORE descent
                literal_tree(doc, child, step)?;
            }
            Ok(true)
        }
        _ => Err(Error::Unsupported("literal projection scalar required")),
    }
}

fn literal_fields(
    doc: &Document,
    fields: &[(String, Id)],
    selected_name: Option<&str>,
    step: &mut impl FnMut() -> Result<(), Error>,
) -> Result<Option<(Id, bool)>, Error> {
    let mut names = BTreeSet::new();
    let mut selected = None;
    for (name, child) in fields {
        step()?; // field inspection BEFORE borrowed-name insertion
        if !names.insert(name.as_str()) {
            return Err(Error::Unsupported("duplicate projection target field"));
        }
        step()?; // child kind, BEFORE validating its entire original subtree
        let container = literal_tree(doc, *child, step)?;
        if selected_name == Some(name.as_str()) {
            selected = Some((*child, container));
        }
    }
    Ok(selected)
}

pub(crate) fn lower(doc: &mut Document, work: usize) -> Result<(), Error> {
    if doc.concatenations != 0 {
        return lower_concatenations(doc, work);
    }
    lower_projections(doc, work)
}

// Keep the historical projection-only path and its charging/order unchanged.
fn lower_projections(doc: &mut Document, work: usize) -> Result<(), Error> {
    if doc.projections == 0 {
        return Ok(());
    }
    if work > 4096 {
        return Err(Error::Limit("evaluation steps"));
    }
    let mut remaining = work
        .checked_sub(doc.preparation_steps)
        .ok_or(Error::Limit("evaluation steps"))?;
    let mut step = || {
        remaining = remaining
            .checked_sub(1)
            .ok_or(Error::Limit("evaluation steps"))?;
        Ok::<(), Error>(())
    };
    step()?; // entry, before any preparation allocation
    let mut selections = Vec::new();
    for (id, node) in doc.nodes.iter().enumerate() {
        step()?;
        match node {
            Expr::Literal(value)
                if value.is_string() || value.is_boolean() || value.as_i64().is_some() => {}
            Expr::Struct(_) | Expr::List(_, None) | Expr::Ref(_) | Expr::Binding(_) => {}
            Expr::Projection { definition, field } => {
                step()?; // bounded descriptor, before allocating it
                step()?; // definition lookup
                let body = *doc
                    .definitions
                    .get(definition)
                    .ok_or_else(|| Error::Unknown(definition.clone()))?;
                step()?; // original target inspection, including empty structs
                let Expr::Struct(fields) = &doc.nodes[body] else {
                    return Err(Error::Unsupported("literal projection struct required"));
                };
                let (selected, container) = literal_fields(doc, fields, Some(field), &mut step)?
                    .ok_or_else(|| Error::Unknown(format!("{definition}.{field}")))?;
                selections.push((id, selected, container));
            }
            _ => return Err(Error::Unsupported("scalar projection document profile")),
        }
    }
    // Every target was checked against the SAME original arena before ANY rewrite.
    for (id, selected, container) in selections {
        step()?;
        doc.nodes[id] = if container {
            Expr::SelectedContainer(selected)
        } else {
            Expr::Binding(selected)
        };
    }
    doc.preparation_steps = work - remaining;
    Ok(())
}
