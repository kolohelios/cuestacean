// Generated lowering driver. Do not edit.
use bootstrap_executor::lowering::{self, Context, Driver, Error};
use serde_json::Value;

pub struct Compiled;

impl Driver for Compiled {
    fn render(&self, ctx: &mut Context, _node: &Value, _depth: usize) -> Result<String, Error> {
        match lowering::instruction_op(_node)? {
            "empty" => ctx.call("module::empty", |_call| Ok(())),
            "outer" => ctx.call("module::record", |call| {
                call.arg(|ctx| ctx.child(_node, "child\"\\\nλ", _depth, self))?;
                call.arg(|ctx| ctx.boolean(_node, "flag\"\\\nλ"))?;
                Ok(())
            }),
            "text\"\\\nλ" => ctx.string(_node, "payload\"\\\nλ"),
            op => Err(lowering::missing_case(op)),
        }
    }
    fn assemble(&self, _frame: &mut bootstrap_executor::module::Frame<'_>) -> Result<(), Error> {
        Ok(())
    }
}

pub fn lower(node: &Value) -> Result<String, Error> {
    Context::run(node, &Compiled)
}
