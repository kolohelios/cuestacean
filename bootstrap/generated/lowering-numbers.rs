// Generated lowering driver. Do not edit.
use bootstrap_executor::lowering::{self, Context, Driver, Error};
use serde_json::Value;

pub struct Compiled;

impl Driver for Compiled {
    fn render(&self, ctx: &mut Context, _node: &Value, _depth: usize) -> Result<String, Error> {
        match lowering::instruction_op(_node)? {
            "item" => ctx.if_zero(
                _node,
                "count",
                |ctx| ctx.signed(_node, "n"),
                |ctx| ctx.text("more"),
            ),
            "root" => ctx.join(_node, "items\"\\\nλ", "|\"\\\nλ|", _depth, self),
            op => Err(lowering::missing_case(op)),
        }
    }
}

pub fn lower(node: &Value) -> Result<String, Error> {
    Context::run(node, &Compiled)
}
