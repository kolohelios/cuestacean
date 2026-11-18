// Generated lowering driver. Do not edit.
use bootstrap_executor::lowering::{self, Context, Driver, Error};
use serde_json::Value;

pub struct Compiled;

impl Driver for Compiled {
    fn render(&self, ctx: &mut Context, _node: &Value, _depth: usize) -> Result<String, Error> {
        match lowering::instruction_op(_node)? {
            "binaryRuntime" | "difference" | "divide" | "index" | "listIndex" | "modulo"
            | "product" | "quotient" | "recordIndex" => ctx.concat(|parts| {
                parts.arg(|ctx| {
                    ctx.select(_node, "op", |ctx, tag| match tag {
                        "difference" => ctx.text("runtime::difference"),
                        "divide" => ctx.text("runtime::divide"),
                        "index" => ctx.text("runtime::index"),
                        "listIndex" => ctx.text("runtime::list_index"),
                        "modulo" => ctx.text("runtime::modulo"),
                        "product" => ctx.text("runtime::product"),
                        "quotient" => ctx.text("runtime::quotient"),
                        "recordIndex" => ctx.text("runtime::record_index"),
                        tag => Err(lowering::missing_selection(tag)),
                    })
                })?;
                parts.arg(|ctx| ctx.text("("))?;
                parts.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "left", _depth, self)))
                })?;
                parts.arg(|ctx| ctx.text(", "))?;
                parts.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "right", _depth, self)))
                })?;
                parts.arg(|ctx| ctx.text(")"))?;
                Ok(())
            }),
            "bool" => ctx.ok(|ctx| ctx.path("runtime::Value::Bool")),
            "boolean" => ctx.ok(|ctx| {
                ctx.call("runtime::Value::Boolean", |call| {
                    call.arg(|ctx| ctx.boolean(_node, "value"))?;
                    Ok(())
                })
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
                call.arg(|ctx| ctx.path("_context"))?;
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
            "list" => ctx.call("runtime::list", |call| {
                call.arg(|ctx| ctx.path("node"))?;
                call.arg(|ctx| ctx.string(_node, "field"))?;
                Ok(())
            }),
            "listExpression" => ctx.call("runtime::list_expression", |call| {
                call.arg(|ctx| ctx.path("node"))?;
                call.arg(|ctx| ctx.string(_node, "field"))?;
                call.arg(|ctx| ctx.path("_context"))?;
                call.arg(|ctx| ctx.path("evaluate"))?;
                Ok(())
            }),
            "listSlice" => ctx.call("runtime::list_slice", |call| {
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "value", _depth, self)))
                })?;
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "lower", _depth, self)))
                })?;
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "upper", _depth, self)))
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
            "logicalOr" => ctx.call("runtime::logical_or", |call| {
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "left", _depth, self)))
                })?;
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "right", _depth, self)))
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
            "record" => ctx.call("runtime::record", |call| {
                call.arg(|ctx| ctx.path("node"))?;
                call.arg(|ctx| ctx.string(_node, "field"))?;
                Ok(())
            }),
            "recordExpression" => ctx.call("runtime::record_expression", |call| {
                call.arg(|ctx| ctx.path("node"))?;
                call.arg(|ctx| ctx.string(_node, "field"))?;
                call.arg(|ctx| ctx.path("_context"))?;
                call.arg(|ctx| ctx.path("evaluate"))?;
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
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "right", _depth, self)))
                })?;
                Ok(())
            }),
            "text" => ctx.call("runtime::text", |call| {
                call.arg(|ctx| ctx.path("node"))?;
                call.arg(|ctx| ctx.string(_node, "field"))?;
                Ok(())
            }),
            "top" => ctx.ok(|ctx| ctx.path("runtime::Value::Top")),
            "unaryRuntime"
            | "length"
            | "listLength"
            | "logicalNot"
            | "lowerBound"
            | "lowerExclusiveBound"
            | "negate"
            | "recordLength"
            | "stringLength"
            | "upperBound"
            | "upperExclusiveBound" => ctx.concat(|parts| {
                parts.arg(|ctx| {
                    ctx.select(_node, "op", |ctx, tag| match tag {
                        "length" => ctx.text("runtime::length"),
                        "listLength" => ctx.text("runtime::list_length"),
                        "logicalNot" => ctx.text("runtime::logical_not"),
                        "lowerBound" => ctx.text("runtime::lower_bound"),
                        "lowerExclusiveBound" => ctx.text("runtime::lower_exclusive_bound"),
                        "negate" => ctx.text("runtime::negate"),
                        "recordLength" => ctx.text("runtime::record_length"),
                        "stringLength" => ctx.text("runtime::string_length"),
                        "upperBound" => ctx.text("runtime::upper_bound"),
                        "upperExclusiveBound" => ctx.text("runtime::upper_exclusive_bound"),
                        tag => Err(lowering::missing_selection(tag)),
                    })
                })?;
                parts.arg(|ctx| ctx.text("("))?;
                parts.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "value", _depth, self)))
                })?;
                parts.arg(|ctx| ctx.text(")"))?;
                Ok(())
            }),
            "unify" => ctx.call("runtime::unify", |call| {
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "left", _depth, self)))
                })?;
                call.arg(|ctx| {
                    ctx.try_value(|ctx| ctx.group(|ctx| ctx.child(_node, "right", _depth, self)))
                })?;
                Ok(())
            }),
            op => Err(lowering::missing_case(op)),
        }
    }
    fn assemble(&self, frame: &mut bootstrap_executor::module::Frame<'_>) -> Result<(), Error> {
        frame.step()?;
        frame.text("// Generated by bootstrap-codegen. Do not edit by hand.\nuse bootstrap_executor::{Error, runtime};\nuse serde_json::Value;\n\npub fn execute(input: &str) -> Result<i64, Error> {\nlet mut _context = runtime::EvaluationContext::default();\nruntime::concrete(evaluate(&runtime::program(input)?, &mut _context)?)\n}\n\npub fn execute_json(input: &str) -> Result<Value, Error> {\nlet mut _context = runtime::EvaluationContext::default();\nruntime::concrete_json(evaluate(&runtime::program(input)?, &mut _context)?)\n}\n\nfn evaluate(node: &Value, _context: &mut runtime::EvaluationContext) -> Result<runtime::Value, Error> {\n_context.visit(|_context| evaluate_node(node, _context))\n}\n\nfn evaluate_node(node: &Value, _context: &mut runtime::EvaluationContext) -> Result<runtime::Value, Error> {\nmatch runtime::kind(node)? {\n")?;
        frame.step()?;
        frame.last_rule()?;
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
            frame.previous_rule()?;
        }
        frame.step()?;
        frame.text("kind => Err(Error::MissingRule { kind: kind.to_string() }),\n}\n}\n")?;
        Ok(())
    }
}

pub fn lower(node: &Value) -> Result<String, Error> {
    Context::run(node, &Compiled)
}
