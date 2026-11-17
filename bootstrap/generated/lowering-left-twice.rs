// Generated lowering driver. Do not edit.
use bootstrap_executor::lowering::{self, Context, Driver, Error};
use serde_json::Value;

pub struct Compiled;

impl Driver for Compiled {
    fn render(&self, ctx: &mut Context, _node: &Value, _depth: usize) -> Result<String, Error> {
        match lowering::instruction_op(_node)? {
            "bool" => ctx.ok(|ctx| ctx.path("runtime::Value::Bool")),
            "boolean" => ctx.ok(|ctx| {
                ctx.call("runtime::Value::Boolean", |call| {
                    call.arg(|ctx| ctx.boolean(_node, "value"))?;
                    Ok(())
                })
            }),
            "difference" => ctx.call("runtime::difference", |call| {
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "left", _depth, self)))
                })?;
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "right", _depth, self)))
                })?;
                Ok(())
            }),
            "divide" => ctx.call("runtime::divide", |call| {
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "left", _depth, self)))
                })?;
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "right", _depth, self)))
                })?;
                Ok(())
            }),
            "equal" => ctx.call("runtime::equal", |call| {
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "left", _depth, self)))
                })?;
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "right", _depth, self)))
                })?;
                Ok(())
            }),
            "evaluate" => ctx.call("evaluate", |call| {
                call.arg(|ctx| {
                    ctx.try_value(|ctx| {
                        ctx.call("runtime::field", |call| {
                            call.arg(|ctx| ctx.path("node"))?;
                            call.arg(|ctx| ctx.string(_node, "field"))?;
                            Ok(())
                        })
                    })
                })?;
                Ok(())
            }),
            "field" => ctx.call("runtime::integer", |call| {
                call.arg(|ctx| ctx.path("node"))?;
                call.arg(|ctx| ctx.string(_node, "field"))?;
                Ok(())
            }),
            "greater" => ctx.call("runtime::greater", |call| {
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "left", _depth, self)))
                })?;
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "right", _depth, self)))
                })?;
                Ok(())
            }),
            "greaterEqual" => ctx.call("runtime::greater_equal", |call| {
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "left", _depth, self)))
                })?;
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "right", _depth, self)))
                })?;
                Ok(())
            }),
            "int" => ctx.ok(|ctx| ctx.path("runtime::Value::Int")),
            "less" => ctx.call("runtime::less", |call| {
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "left", _depth, self)))
                })?;
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "right", _depth, self)))
                })?;
                Ok(())
            }),
            "lessEqual" => ctx.call("runtime::less_equal", |call| {
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "left", _depth, self)))
                })?;
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "right", _depth, self)))
                })?;
                Ok(())
            }),
            "logicalAnd" => ctx.call("runtime::logical_and", |call| {
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "left", _depth, self)))
                })?;
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "right", _depth, self)))
                })?;
                Ok(())
            }),
            "logicalNot" => ctx.call("runtime::logical_not", |call| {
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "value", _depth, self)))
                })?;
                Ok(())
            }),
            "logicalOr" => ctx.call("runtime::logical_or", |call| {
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "left", _depth, self)))
                })?;
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "right", _depth, self)))
                })?;
                Ok(())
            }),
            "lowerBound" => ctx.call("runtime::lower_bound", |call| {
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "value", _depth, self)))
                })?;
                Ok(())
            }),
            "modulo" => ctx.call("runtime::modulo", |call| {
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "left", _depth, self)))
                })?;
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "right", _depth, self)))
                })?;
                Ok(())
            }),
            "negate" => ctx.call("runtime::negate", |call| {
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "value", _depth, self)))
                })?;
                Ok(())
            }),
            "notEqual" => ctx.call("runtime::not_equal", |call| {
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "left", _depth, self)))
                })?;
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "right", _depth, self)))
                })?;
                Ok(())
            }),
            "null" => ctx.ok(|ctx| ctx.path("runtime::Value::Null")),
            "product" => ctx.call("runtime::product", |call| {
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "left", _depth, self)))
                })?;
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "right", _depth, self)))
                })?;
                Ok(())
            }),
            "quotient" => ctx.call("runtime::quotient", |call| {
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "left", _depth, self)))
                })?;
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "right", _depth, self)))
                })?;
                Ok(())
            }),
            "remainder" => ctx.call("runtime::remainder", |call| {
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "left", _depth, self)))
                })?;
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "right", _depth, self)))
                })?;
                Ok(())
            }),
            "stringType" => ctx.ok(|ctx| ctx.path("runtime::Value::String")),
            "sum" => ctx.call("runtime::sum", |call| {
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "left", _depth, self)))
                })?;
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "left", _depth, self)))
                })?;
                Ok(())
            }),
            "text" => ctx.call("runtime::text", |call| {
                call.arg(|ctx| ctx.path("node"))?;
                call.arg(|ctx| ctx.string(_node, "field"))?;
                Ok(())
            }),
            "top" => ctx.ok(|ctx| ctx.path("runtime::Value::Top")),
            "unify" => ctx.call("runtime::unify", |call| {
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "left", _depth, self)))
                })?;
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "right", _depth, self)))
                })?;
                Ok(())
            }),
            "upperBound" => ctx.call("runtime::upper_bound", |call| {
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "value", _depth, self)))
                })?;
                Ok(())
            }),
            op => Err(lowering::missing_case(op)),
        }
    }
    fn assemble(&self, frame: &mut bootstrap_executor::module::Frame<'_>) -> Result<(), Error> {
        frame.step()?;
        frame.text("// Generated by bootstrap-codegen. Do not edit by hand.\nuse bootstrap_executor::{Error, runtime};\nuse serde_json::Value;\n\npub fn execute(input: &str) -> Result<i64, Error> {\nruntime::concrete(evaluate(&runtime::program(input)?)?)\n}\n\npub fn execute_json(input: &str) -> Result<Value, Error> {\nruntime::concrete_json(evaluate(&runtime::program(input)?)?)\n}\n\nfn evaluate(node: &Value) -> Result<runtime::Value, Error> {\nmatch runtime::kind(node)? {\n")?;
        frame.step()?;
        frame.first_rule()?;
        while {
            frame.step()?;
            frame.has_rule()
        } {
            frame.step()?;
            frame.kind()?;
            frame.step()?;
            frame.text(" => ")?;
            frame.step()?;
            frame.expression(self)?;
            frame.step()?;
            frame.text(",\n")?;
            frame.step()?;
            frame.next_rule()?;
        }
        frame.step()?;
        frame.text("kind => Err(Error::MissingRule { kind: kind.to_string() }),\n}\n}\n")?;
        Ok(())
    }
}

pub fn lower(node: &Value) -> Result<String, Error> {
    Context::run(node, &Compiled)
}
