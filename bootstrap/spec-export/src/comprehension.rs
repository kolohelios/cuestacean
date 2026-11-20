//! Finite static specialization; no iterator environment reaches evaluation.
use crate::model::{Document, Error, Expr, Id, Template};
use std::collections::BTreeSet;

struct Preparation {
    remaining: usize,
    steps: usize,
    strings: usize,
    syntax: usize,
    base: usize,
    nodes: Vec<Expr>,
}
impl Preparation {
    fn step(&mut self, n: usize) -> Result<(), Error> {
        self.remaining = self
            .remaining
            .checked_sub(n)
            .ok_or(Error::Limit("evaluation steps"))?;
        self.steps += n;
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
    fn instantiate(&mut self, template: &Template, key: &str, value: &str) -> Result<Id, Error> {
        self.step(1)?;
        if self.syntax >= 2048 {
            return Err(Error::Limit("syntax nodes"));
        }
        self.syntax += 1;
        let expr = match template {
            Template::Key => Expr::Literal(self.text(key)?.into()),
            Template::Value => Expr::Literal(self.text(value)?.into()),
            Template::Literal(serde_json::Value::String(text)) => {
                Expr::Literal(self.text(text)?.into())
            }
            Template::Literal(value) => Expr::Literal(value.clone()),
            Template::Struct(fields) => {
                let mut result = Vec::new();
                for (name, child) in fields {
                    result.push((self.text(name)?, self.instantiate(child, key, value)?));
                }
                Expr::Struct(result)
            }
            Template::List(items) => {
                let mut result = Vec::new();
                for child in items {
                    result.push(self.instantiate(child, key, value)?);
                }
                Expr::List(result, None)
            }
        };
        let id = self.base + self.nodes.len();
        self.nodes.push(expr);
        Ok(id)
    }
}

pub(crate) fn lower(doc: &mut Document, work: usize) -> Result<(), Error> {
    if doc.comprehensions == 0 {
        return Ok(());
    }
    if doc.marked || !doc.definitions.is_empty() {
        return Err(Error::Unsupported("comprehension document profile"));
    }
    if work > 4096 {
        return Err(Error::Limit("evaluation steps"));
    }
    let mut prep = Preparation {
        remaining: work,
        steps: 0,
        strings: 0,
        syntax: doc.syntax,
        base: doc.nodes.len(),
        nodes: Vec::new(),
    };
    // Validate EVERY source in the original resolved arena before replacing ANY
    // node. Descriptors contain IDs only, never speculative string/map copies.
    let mut sources = Vec::new();
    for (id, node) in doc.nodes.iter().enumerate() {
        if let Expr::Comprehension { source, .. } = node {
            prep.step(1)?;
            prep.step(1)?;
            let Expr::Binding(body) = doc.nodes[*source] else {
                return Err(Error::Unsupported("direct table binding required"));
            };
            prep.step(1)?;
            let Expr::Struct(fields) = &doc.nodes[body] else {
                return Err(Error::Unsupported("literal table struct required"));
            };
            if fields.len() > 64 {
                return Err(Error::Limit("container items"));
            }
            let mut names = BTreeSet::new();
            for (name, child) in fields {
                prep.step(1)?;
                if !names.insert(name.as_str()) {
                    return Err(Error::Unsupported("duplicate table field"));
                }
                prep.step(1)?;
                if !matches!(
                    &doc.nodes[*child],
                    Expr::Literal(serde_json::Value::String(_))
                ) {
                    return Err(Error::Unsupported("literal table string required"));
                }
            }
            sources.push((id, body));
        }
    }
    for (id, body) in sources {
        let Expr::Comprehension { template, .. } = std::mem::replace(&mut doc.nodes[id], Expr::Top)
        else {
            unreachable!("validated comprehension descriptor")
        };
        let Expr::Struct(fields) = &doc.nodes[body] else {
            unreachable!()
        };
        let mut result = Vec::new();
        for (name, child) in fields {
            prep.step(1)?; // emitted field, before copying/allocating it
            let Expr::Literal(serde_json::Value::String(value)) = &doc.nodes[*child] else {
                unreachable!()
            };
            result.push((prep.text(name)?, prep.instantiate(&template, name, value)?));
        }
        doc.nodes[id] = Expr::Struct(result);
    }
    doc.nodes.extend(prep.nodes);
    doc.syntax = prep.syntax;
    doc.preparation_steps = prep.steps;
    doc.preparation_strings = prep.strings;
    Ok(())
}
