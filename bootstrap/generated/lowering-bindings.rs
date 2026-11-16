// Generated lowering driver. Do not edit.
use bootstrap_executor::lowering::{self, Context, Driver, Error};
use serde_json::Value;

pub struct Compiled;

impl Driver for Compiled {
    fn render(&self, ctx: &mut Context, _node: &Value, _depth: usize) -> Result<String, Error> {
        match lowering::instruction_op(_node)? {
            "root" => ctx.bind(
                |ctx| ctx.string(_node, "text\"\\\nλ"),
                |ctx, _render_saved| {
                    ctx.bind(
                        |ctx| ctx.rendered(_render_saved),
                        |ctx, _render_saved| {
                            ctx.if_equal(
                                |ctx| ctx.rendered(_render_saved),
                                |ctx| ctx.text("\"same\""),
                                |ctx| {
                                    ctx.concat(|parts| {
                                        parts.arg(|ctx| ctx.rendered(_render_saved))?;
                                        parts.arg(|ctx| ctx.text("!"))?;
                                        Ok(())
                                    })
                                },
                                |ctx| {
                                    ctx.if_true(
                                        _node,
                                        "flag",
                                        |ctx| ctx.rendered(_render_saved),
                                        |ctx| ctx.text("no"),
                                    )
                                },
                            )
                        },
                    )
                },
            ),
            op => Err(lowering::missing_case(op)),
        }
    }
}

pub fn lower(node: &Value) -> Result<String, Error> {
    Context::run(node, &Compiled)
}
