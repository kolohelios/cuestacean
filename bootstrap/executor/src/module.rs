//! Checked primitives and policy statements for module assembly.
use std::{fmt, fmt::Write as _};

use serde::Deserialize;
use serde_json::Value;

use crate::lowering::{self, Context, Driver, Error};

const MAX_OUTPUT: usize = 1024 * 1024;

/// Mutable state shared by the selected assembly and instruction drivers.
pub struct Frame<'a> {
    rules: &'a [(String, Value)],
    cursor: Option<usize>,
    output: String,
    steps: usize,
}

impl<'a> Frame<'a> {
    pub fn run(rules: &'a [(String, Value)], driver: &dyn Driver) -> Result<String, Error> {
        let mut frame = Self {
            rules,
            cursor: None,
            output: String::new(),
            steps: 0,
        };
        driver.assemble(&mut frame)?;
        Ok(frame.output)
    }

    pub fn step(&mut self) -> Result<(), Error> {
        if self.steps >= 4096 {
            return Err(lowering::invalid("module exceeds 4096 steps"));
        }
        self.steps += 1;
        Ok(())
    }

    pub fn text(&mut self, text: &str) -> Result<(), Error> {
        if text.len() > MAX_OUTPUT - self.output.len() {
            return Err(lowering::invalid("module exceeds 1 MiB output bytes"));
        }
        self.output.push_str(text);
        Ok(())
    }

    pub fn first_rule(&mut self) -> Result<(), Error> {
        self.cursor = if self.rules.is_empty() { None } else { Some(0) };
        Ok(())
    }
    pub fn last_rule(&mut self) -> Result<(), Error> {
        self.cursor = self.rules.len().checked_sub(1);
        Ok(())
    }
    pub fn next_rule(&mut self) -> Result<(), Error> {
        let index = self.current()?;
        self.cursor = (index + 1 < self.rules.len()).then_some(index + 1);
        Ok(())
    }
    pub fn previous_rule(&mut self) -> Result<(), Error> {
        self.cursor = self.current()?.checked_sub(1);
        Ok(())
    }
    pub fn has_rule(&self) -> bool {
        self.cursor.is_some()
    }
    fn current(&self) -> Result<usize, Error> {
        self.cursor
            .ok_or_else(|| lowering::invalid("no current rule"))
    }
    pub fn kind(&mut self) -> Result<(), Error> {
        let value = &self.rules[self.current()?].0;
        if value.len() > 64 * 1024 {
            return Err(lowering::invalid("rule kind exceeds 64 KiB"));
        }
        write!(OutputWriter(self), "{value:?}")
            .map_err(|_| lowering::invalid("module exceeds 1 MiB output bytes"))
    }
    pub fn expression(&mut self, driver: &dyn Driver) -> Result<(), Error> {
        let node = &self.rules[self.current()?].1;
        let rendered = Context::run(node, driver)?;
        self.text(&rendered)
    }
}

struct OutputWriter<'a, 'rules>(&'a mut Frame<'rules>);
impl fmt::Write for OutputWriter<'_, '_> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.0.text(text).map_err(|_| fmt::Error)
    }
}

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "camelCase", deny_unknown_fields)]
pub(crate) enum Statement {
    Text { value: String },
    FirstRule {},
    LastRule {},
    NextRule {},
    PreviousRule {},
    Kind {},
    Expression {},
    WhileRules { body: Vec<Statement> },
}

pub(crate) fn validate(body: &[Statement]) -> Result<(), Error> {
    fn visit(body: &[Statement], depth: usize, count: &mut usize) -> Result<(), Error> {
        for statement in body {
            *count += 1;
            if *count > 64 || depth > 16 {
                return Err(lowering::invalid(
                    "module exceeds 64 statements or 16 nesting levels",
                ));
            }
            match statement {
                Statement::Text { value } if value.len() > 64 * 1024 => {
                    return Err(lowering::invalid("module text exceeds 64 KiB"));
                }
                Statement::WhileRules { body } => visit(body, depth + 1, count)?,
                _ => {}
            }
        }
        Ok(())
    }
    visit(body, 1, &mut 0)
}

pub(crate) fn execute(
    body: &[Statement],
    frame: &mut Frame<'_>,
    driver: &dyn Driver,
) -> Result<(), Error> {
    for statement in body {
        if let Statement::WhileRules { body } = statement {
            loop {
                frame.step()?;
                if !frame.has_rule() {
                    break;
                }
                execute(body, frame, driver)?;
            }
        } else {
            frame.step()?;
            match statement {
                Statement::Text { value } => frame.text(value)?,
                Statement::FirstRule {} => frame.first_rule()?,
                Statement::LastRule {} => frame.last_rule()?,
                Statement::NextRule {} => frame.next_rule()?,
                Statement::PreviousRule {} => frame.previous_rule()?,
                Statement::Kind {} => frame.kind()?,
                Statement::Expression {} => frame.expression(driver)?,
                Statement::WhileRules { .. } => unreachable!(),
            }
        }
    }
    Ok(())
}

/// Emit only the body of a compiled driver's assembly method.
pub(crate) fn emit(body: &[Statement], out: &mut String) {
    for statement in body {
        match statement {
            Statement::WhileRules { body } => {
                out.push_str("while { frame.step()?; frame.has_rule() } {\n");
                emit(body, out);
                out.push_str("}\n");
            }
            _ => {
                out.push_str("frame.step()?; ");
                match statement {
                    Statement::Text { value } => {
                        writeln!(out, "frame.text({value:?})?;").expect("write to string")
                    }
                    Statement::FirstRule {} => out.push_str("frame.first_rule()?;\n"),
                    Statement::LastRule {} => out.push_str("frame.last_rule()?;\n"),
                    Statement::NextRule {} => out.push_str("frame.next_rule()?;\n"),
                    Statement::PreviousRule {} => out.push_str("frame.previous_rule()?;\n"),
                    Statement::Kind {} => out.push_str("frame.kind()?;\n"),
                    Statement::Expression {} => out.push_str("frame.expression(self)?;\n"),
                    Statement::WhileRules { .. } => unreachable!(),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn frame(rules: &[(String, Value)]) -> Frame<'_> {
        Frame {
            rules,
            cursor: None,
            output: String::new(),
            steps: 0,
        }
    }

    #[test]
    fn cursors_and_quoting_are_checked() -> Result<(), Error> {
        let rules = vec![
            ("a\"\n雪".to_owned(), json!({})),
            ("b".to_owned(), json!({})),
        ];
        let mut f = frame(&rules);
        assert!(f.next_rule().is_err());
        f.last_rule()?;
        f.previous_rule()?;
        f.kind()?;
        assert_eq!(f.output, format!("{:?}", rules[0].0));
        f.previous_rule()?;
        assert!(!f.has_rule());
        assert!(f.kind().is_err());
        f.first_rule()?;
        f.next_rule()?;
        f.next_rule()?;
        assert!(f.next_rule().is_err());
        let mut empty = frame(&[]);
        empty.first_rule()?;
        empty.last_rule()?;
        assert!(!empty.has_rule());
        Ok(())
    }

    #[test]
    fn limits_cover_quote_output_and_stalled_loops() -> Result<(), Error> {
        let rules = vec![("\n".repeat(64 * 1024), json!({}))];
        let mut f = frame(&rules);
        f.first_rule()?;
        f.text(&"x".repeat(MAX_OUTPUT - 2))?;
        assert!(f.kind().is_err());
        assert!(f.output.len() <= MAX_OUTPUT);
        let mut f = frame(&rules);
        f.first_rule()?;
        let policy = lowering::Policy::parse(
            r#"{"formatVersion":1,"cases":{"x":{"op":"path","value":"x"}}}"#,
        )?;
        let body = [Statement::WhileRules { body: vec![] }];
        assert!(
            execute(&body, &mut f, &policy)
                .expect_err("stalled loop")
                .to_string()
                .contains("4096")
        );
        Ok(())
    }

    #[test]
    fn exact_statement_depth_and_quote_boundaries() -> Result<(), Error> {
        let mut body: Vec<_> = (0..64)
            .map(|_| Statement::Text {
                value: String::new(),
            })
            .collect();
        validate(&body)?;
        body.push(Statement::Kind {});
        assert!(validate(&body).is_err());
        let mut body = vec![];
        for _ in 0..16 {
            body = vec![Statement::WhileRules { body }];
        }
        validate(&body)?;
        assert!(validate(&[Statement::WhileRules { body }]).is_err());
        let rules = vec![("x".to_owned(), json!({}))];
        for remaining in [3, 2] {
            let mut f = frame(&rules);
            f.first_rule()?;
            f.text(&"x".repeat(MAX_OUTPUT - remaining))?;
            let result = f.kind();
            assert_eq!(result.is_ok(), remaining == 3);
            assert_eq!(f.output.len(), MAX_OUTPUT);
        }
        let rules = vec![("x".repeat(65537), json!({}))];
        let mut f = frame(&rules);
        f.first_rule()?;
        f.text(&"x".repeat(MAX_OUTPUT))?;
        assert_eq!(
            f.kind().expect_err("kind size before output").to_string(),
            "lowering: rule kind exceeds 64 KiB"
        );
        Ok(())
    }

    #[test]
    fn validates_unused_nested_statements() {
        let body = [Statement::WhileRules {
            body: vec![Statement::Text {
                value: "x".repeat(65537),
            }],
        }];
        assert!(validate(&body).is_err());
        assert!(
            serde_json::from_str::<Vec<Statement>>(r#"[{"op":"firstRule","extra":true}]"#).is_err()
        );
    }
}
