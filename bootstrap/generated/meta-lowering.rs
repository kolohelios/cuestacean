// Generated lowering driver. Do not edit.
use bootstrap_executor::lowering::{self, Context, Driver, Error};
use serde_json::Value;

pub struct Compiled;

impl Driver for Compiled {
    fn render(&self, ctx: &mut Context, _node: &Value, _depth: usize) -> Result<String, Error> {
        match lowering::instruction_op(_node)? {
            "bind" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("ctx.bind(|ctx| "))?;
parts.arg(|ctx| ctx.child(_node, "value", _depth, self))?;
parts.arg(|ctx| ctx.text(", |ctx, _render_"))?;
parts.arg(|ctx| ctx.identifier(_node, "name", false, false))?;
parts.arg(|ctx| ctx.text("| "))?;
parts.arg(|ctx| ctx.child(_node, "body", _depth, self))?;
parts.arg(|ctx| ctx.text(")"))?;
Ok(())
}),
            "call" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("ctx.call("))?;
parts.arg(|ctx| ctx.string(_node, "path"))?;
parts.arg(|ctx| ctx.text(", |"))?;
parts.arg(|ctx| ctx.if_empty(_node, "args", |ctx| ctx.text("_call"), |ctx| ctx.text("call")))?;
parts.arg(|ctx| ctx.text("| { "))?;
parts.arg(|ctx| ctx.list(_node, "args", _depth, self))?;
parts.arg(|ctx| ctx.text("Ok(()) })"))?;
Ok(())
}),
            "callArgument" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("call.arg(|ctx| "))?;
parts.arg(|ctx| ctx.child(_node, "value", _depth, self))?;
parts.arg(|ctx| ctx.text(")?; "))?;
Ok(())
}),
            "concat" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("ctx.concat(|"))?;
parts.arg(|ctx| ctx.if_empty(_node, "parts", |ctx| ctx.text("_parts"), |ctx| ctx.text("parts")))?;
parts.arg(|ctx| ctx.text("| {\n"))?;
parts.arg(|ctx| ctx.list(_node, "parts", _depth, self))?;
parts.arg(|ctx| ctx.text("Ok(())\n})"))?;
Ok(())
}),
            "concatArgument" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("parts.arg(|ctx| "))?;
parts.arg(|ctx| ctx.child(_node, "value", _depth, self))?;
parts.arg(|ctx| ctx.text(")?;\n"))?;
Ok(())
}),
            "dispatchCase" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("            "))?;
parts.arg(|ctx| ctx.join(_node, "keys", " | ", _depth, self))?;
parts.arg(|ctx| ctx.text(" => "))?;
parts.arg(|ctx| ctx.child(_node, "expression", _depth, self))?;
parts.arg(|ctx| ctx.text(",\n"))?;
Ok(())
}),
            "dispatchKey" => ctx.string(_node, "value"),
            "driver" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("// Generated lowering driver. Do not edit.\nuse bootstrap_executor::lowering::{self, Context, Driver, Error};\nuse serde_json::Value;\n\npub struct Compiled;\n\nimpl Driver for Compiled {\n    fn render(&self, ctx: &mut Context, _node: &Value, _depth: usize) -> Result<String, Error> {\n        match lowering::instruction_op(_node)? {\n"))?;
parts.arg(|ctx| ctx.list(_node, "cases", _depth, self))?;
parts.arg(|ctx| ctx.text("            op => Err(lowering::missing_case(op)),\n        }\n    }\n"))?;
parts.arg(|ctx| ctx.if_true(_node, "hasModule", |ctx| ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("    fn assemble(&self, "))?;
parts.arg(|ctx| ctx.if_empty(_node, "module", |ctx| ctx.text("_frame"), |ctx| ctx.text("frame")))?;
parts.arg(|ctx| ctx.text(": &mut bootstrap_executor::module::Frame<'_>) -> Result<(), Error> {\n"))?;
parts.arg(|ctx| ctx.list(_node, "module", _depth, self))?;
parts.arg(|ctx| ctx.text("Ok(())\n}\n"))?;
Ok(())
}), |ctx| ctx.text("")))?;
parts.arg(|ctx| ctx.text("}\n\npub fn lower(node: &Value) -> Result<String, Error> {\n    Context::run(node, &Compiled)\n}\n"))?;
Ok(())
}),
            "emitConditional" | "ifEmpty" | "ifTrue" | "ifZero" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("ctx."))?;
parts.arg(|ctx| ctx.identifier(_node, "op", true, false))?;
parts.arg(|ctx| ctx.text("(_node, "))?;
parts.arg(|ctx| ctx.string(_node, "field"))?;
parts.arg(|ctx| ctx.text(", |ctx| "))?;
parts.arg(|ctx| ctx.child(_node, "then", _depth, self))?;
parts.arg(|ctx| ctx.text(", |ctx| "))?;
parts.arg(|ctx| ctx.child(_node, "else", _depth, self))?;
parts.arg(|ctx| ctx.text(")"))?;
Ok(())
}),
            "emitField" | "boolean" | "bytePattern" | "charPattern" | "signed" | "string" | "unsigned" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("ctx."))?;
parts.arg(|ctx| ctx.identifier(_node, "op", true, false))?;
parts.arg(|ctx| ctx.text("(_node, "))?;
parts.arg(|ctx| ctx.string(_node, "field"))?;
parts.arg(|ctx| ctx.text(")"))?;
Ok(())
}),
            "emitModuleAction" | "firstRule" | "kind" | "lastRule" | "nextRule" | "previousRule" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("frame.step()?; frame."))?;
parts.arg(|ctx| ctx.identifier(_node, "op", true, false))?;
parts.arg(|ctx| ctx.text("()?;\n"))?;
Ok(())
}),
            "emitRecursive" | "list" | "lower" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("ctx."))?;
parts.arg(|ctx| ctx.select(_node, "op", |ctx, tag| match tag { "list" => ctx.text("list"), "lower" => ctx.text("child"), tag => Err(lowering::missing_selection(tag)), }))?;
parts.arg(|ctx| ctx.text("(_node, "))?;
parts.arg(|ctx| ctx.string(_node, "field"))?;
parts.arg(|ctx| ctx.text(", _depth, self)"))?;
Ok(())
}),
            "emitUnary" | "group" | "ok" | "try" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("ctx."))?;
parts.arg(|ctx| ctx.select(_node, "op", |ctx, tag| match tag { "group" => ctx.text("group"), "ok" => ctx.text("ok"), "try" => ctx.text("try_value"), tag => Err(lowering::missing_selection(tag)), }))?;
parts.arg(|ctx| ctx.text("(|ctx| "))?;
parts.arg(|ctx| ctx.child(_node, "value", _depth, self))?;
parts.arg(|ctx| ctx.text(")"))?;
Ok(())
}),
            "emitValue" | "path" | "text" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("ctx."))?;
parts.arg(|ctx| ctx.identifier(_node, "op", false, false))?;
parts.arg(|ctx| ctx.text("("))?;
parts.arg(|ctx| ctx.string(_node, "value"))?;
parts.arg(|ctx| ctx.text(")"))?;
Ok(())
}),
            "expression" => ctx.text("frame.step()?; frame.expression(self)?;\n"),
            "get" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("ctx.rendered(_render_"))?;
parts.arg(|ctx| ctx.identifier(_node, "name", false, false))?;
parts.arg(|ctx| ctx.text(")"))?;
Ok(())
}),
            "identifier" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("ctx.identifier(_node, "))?;
parts.arg(|ctx| ctx.string(_node, "field"))?;
parts.arg(|ctx| ctx.text(", "))?;
parts.arg(|ctx| ctx.boolean(_node, "snake"))?;
parts.arg(|ctx| ctx.text(", "))?;
parts.arg(|ctx| ctx.boolean(_node, "upperFirst"))?;
parts.arg(|ctx| ctx.text(")"))?;
Ok(())
}),
            "ifEqual" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("ctx.if_equal(|ctx| "))?;
parts.arg(|ctx| ctx.child(_node, "left", _depth, self))?;
parts.arg(|ctx| ctx.text(", |ctx| "))?;
parts.arg(|ctx| ctx.child(_node, "right", _depth, self))?;
parts.arg(|ctx| ctx.text(", |ctx| "))?;
parts.arg(|ctx| ctx.child(_node, "then", _depth, self))?;
parts.arg(|ctx| ctx.text(", |ctx| "))?;
parts.arg(|ctx| ctx.child(_node, "else", _depth, self))?;
parts.arg(|ctx| ctx.text(")"))?;
Ok(())
}),
            "join" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("ctx.join(_node, "))?;
parts.arg(|ctx| ctx.string(_node, "field"))?;
parts.arg(|ctx| ctx.text(", "))?;
parts.arg(|ctx| ctx.string(_node, "separator"))?;
parts.arg(|ctx| ctx.text(", _depth, self)"))?;
Ok(())
}),
            "moduleText" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("frame.step()?; frame.text("))?;
parts.arg(|ctx| ctx.string(_node, "value"))?;
parts.arg(|ctx| ctx.text(")?;\n"))?;
Ok(())
}),
            "select" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("ctx.select(_node, "))?;
parts.arg(|ctx| ctx.string(_node, "field"))?;
parts.arg(|ctx| ctx.text(", |ctx, tag| match tag { "))?;
parts.arg(|ctx| ctx.list(_node, "cases", _depth, self))?;
parts.arg(|ctx| ctx.text("tag => Err(lowering::missing_selection(tag)), })"))?;
Ok(())
}),
            "selectCase" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.string(_node, "tag"))?;
parts.arg(|ctx| ctx.text(" => "))?;
parts.arg(|ctx| ctx.child(_node, "value", _depth, self))?;
parts.arg(|ctx| ctx.text(", "))?;
Ok(())
}),
            "whileRules" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("while { frame.step()?; frame.has_rule() } {\n"))?;
parts.arg(|ctx| ctx.list(_node, "body", _depth, self))?;
parts.arg(|ctx| ctx.text("}\n"))?;
Ok(())
}),
            op => Err(lowering::missing_case(op)),
        }
    }
}

pub fn lower(node: &Value) -> Result<String, Error> {
    Context::run(node, &Compiled)
}
