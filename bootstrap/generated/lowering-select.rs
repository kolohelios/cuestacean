// Generated lowering driver. Do not edit.
use bootstrap_executor::lowering::{self, Context, Driver, Error};
use serde_json::Value;

pub struct Compiled;

impl Driver for Compiled {
    fn render(&self, ctx: &mut Context, _node: &Value, _depth: usize) -> Result<String, Error> {
        match lowering::instruction_op(_node)? {
            "leaf" => ctx.select(_node, "mode", |ctx, tag| match tag {
                "no" => ctx.text("other"),
                "yes" => ctx.text("chosen"),
                tag => Err(lowering::missing_selection(tag)),
            }),
            "root\"\\\nλ" => ctx.select(_node, "kind\"\\\nλ", |ctx, tag| match tag {
                "fallback" => ctx.text("fallback"),
                "go\"\\\nλ" => ctx.child(_node, "child\"\\\nλ", _depth, self),
                tag => Err(lowering::missing_selection(tag)),
            }),
            op => Err(lowering::missing_case(op)),
        }
    }
}

pub fn lower(node: &Value) -> Result<String, Error> {
    Context::run(node, &Compiled)
}
