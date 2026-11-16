// Generated lowering driver. Do not edit.
use bootstrap_executor::lowering::{self, Context, Driver, Error};
use serde_json::Value;

pub struct Compiled;

impl Driver for Compiled {
    fn render(&self, ctx: &mut Context, _node: &Value, _depth: usize) -> Result<String, Error> {
        match lowering::instruction_op(_node)? {
            "root" => ctx.text("unused"),
            op => Err(lowering::missing_case(op)),
        }
    }
    fn assemble(&self, frame: &mut bootstrap_executor::module::Frame<'_>) -> Result<(), Error> {
        frame.step()?;
        frame.first_rule()?;
        while {
            frame.step()?;
            frame.has_rule()
        } {
            frame.step()?;
            frame.kind()?;
            frame.step()?;
            frame.previous_rule()?;
        }
        Ok(())
    }
}

pub fn lower(node: &Value) -> Result<String, Error> {
    Context::run(node, &Compiled)
}
