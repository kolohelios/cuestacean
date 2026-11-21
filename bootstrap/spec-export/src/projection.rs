//! Direct scalar selection from original literal definitions, not field solving.
use crate::model::{Document, Error, Expr};
use std::collections::BTreeSet;

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
                let mut names = BTreeSet::new();
                let mut selected = None;
                for (name, child) in fields {
                    step()?; // field inspection BEFORE borrowed-name insertion
                    if !names.insert(name.as_str()) {
                        return Err(Error::Unsupported("duplicate projection target field"));
                    }
                    step()?;
                    if !matches!(&doc.nodes[*child], Expr::Literal(value)
                        if value.is_string() || value.is_boolean() || value.as_i64().is_some())
                    {
                        return Err(Error::Unsupported("literal projection scalar required"));
                    }
                    if name == field {
                        selected = Some(*child);
                    }
                }
                let selected =
                    selected.ok_or_else(|| Error::Unknown(format!("{definition}.{field}")))?;
                selections.push((id, selected));
            }
            _ => return Err(Error::Unsupported("scalar projection document profile")),
        }
    }
    // Every target was checked against the SAME original arena before ANY rewrite.
    for (id, selected) in selections {
        step()?;
        doc.nodes[id] = Expr::Binding(selected);
    }
    doc.preparation_steps = work - remaining;
    Ok(())
}
