// Generated lowering driver. Do not edit.
use bootstrap_executor::lowering::{self, Context, Driver, Error};
use serde_json::Value;

pub struct Compiled;

impl Driver for Compiled {
    fn render(&self, ctx: &mut Context, _node: &Value, _depth: usize) -> Result<String, Error> {
        match lowering::instruction_op(_node)? {
            "item\"\\\nλ" => ctx.concat(|parts| {
                parts.arg(|ctx| ctx.text("raw\"\\\nλ"))?;
                parts.arg(|ctx| ctx.unsigned(_node, "number\"\\\nλ"))?;
                Ok(())
            }),
            "root" => ctx.if_empty(
                _node,
                "items\"\\\nλ",
                |ctx| ctx.concat(|_parts| Ok(())),
                |ctx| ctx.list(_node, "items\"\\\nλ", _depth, self),
            ),
            "string" => ctx.string(_node, "value"),
            op => Err(lowering::missing_case(op)),
        }
    }
}

pub fn lower(node: &Value) -> Result<String, Error> {
    Context::run(node, &Compiled)
}
