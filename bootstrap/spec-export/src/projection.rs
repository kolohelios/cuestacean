//! Direct selection from original literal definition trees, not field solving.
use crate::model::{Document, Error, Expr, Id};
use std::collections::BTreeSet;

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
