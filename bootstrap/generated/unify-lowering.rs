// Generated lowering driver. Do not edit.
use bootstrap_executor::lowering::{self, Context, Driver, Error};
use serde_json::Value;

pub struct Compiled;

impl Driver for Compiled {
    fn render(&self, ctx: &mut Context, _node: &Value, _depth: usize) -> Result<String, Error> {
        match lowering::instruction_op(_node)? {
            "and" => ctx.ok(|ctx| ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("("))?;
parts.arg(|ctx| ctx.child(_node, "left", _depth, self))?;
parts.arg(|ctx| ctx.text(")? && ("))?;
parts.arg(|ctx| ctx.child(_node, "right", _depth, self))?;
parts.arg(|ctx| ctx.text(")?"))?;
Ok(())
})),
            "boolean" => ctx.ok(|ctx| ctx.call("Value::Boolean", |call| { call.arg(|ctx| ctx.boolean(_node, "value"))?; Ok(()) })),
            "booleanIs" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("crate::unification::boolean_is(&"))?;
parts.arg(|ctx| ctx.select(_node, "input", |ctx, tag| match tag { "left" => ctx.text("_left"), "right" => ctx.text("_right"), tag => Err(lowering::missing_selection(tag)), }))?;
parts.arg(|ctx| ctx.text(", "))?;
parts.arg(|ctx| ctx.boolean(_node, "value"))?;
parts.arg(|ctx| ctx.text(")"))?;
Ok(())
}),
            "bounds" => ctx.ok(|ctx| ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("Value::Bounds { lower: ("))?;
parts.arg(|ctx| ctx.child(_node, "lower", _depth, self))?;
parts.arg(|ctx| ctx.text(")?, upper: ("))?;
parts.arg(|ctx| ctx.child(_node, "upper", _depth, self))?;
parts.arg(|ctx| ctx.text(")? }"))?;
Ok(())
})),
            "choose" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("if ("))?;
parts.arg(|ctx| ctx.child(_node, "condition", _depth, self))?;
parts.arg(|ctx| ctx.text(")? { "))?;
parts.arg(|ctx| ctx.child(_node, "then", _depth, self))?;
parts.arg(|ctx| ctx.text(" } else { "))?;
parts.arg(|ctx| ctx.child(_node, "else", _depth, self))?;
parts.arg(|ctx| ctx.text(" }"))?;
Ok(())
}),
            "compare" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("crate::unification::compare(("))?;
parts.arg(|ctx| ctx.child(_node, "left", _depth, self))?;
parts.arg(|ctx| ctx.text(")?, ("))?;
parts.arg(|ctx| ctx.child(_node, "right", _depth, self))?;
parts.arg(|ctx| ctx.text(")?, crate::unification::Relation::"))?;
parts.arg(|ctx| ctx.select(_node, "relation", |ctx, tag| match tag { "equal" => ctx.text("Equal"), "greater" => ctx.text("Greater"), "greaterEqual" => ctx.text("GreaterEqual"), "less" => ctx.text("Less"), "lessEqual" => ctx.text("LessEqual"), tag => Err(lowering::missing_selection(tag)), }))?;
parts.arg(|ctx| ctx.text(")"))?;
Ok(())
}),
            "empty" => ctx.text("Ok::<_, Error>(None::<i64>)"),
            "fail" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("crate::unification::fail(crate::unification::Failure::"))?;
parts.arg(|ctx| ctx.select(_node, "reason", |ctx, tag| match tag { "arithmetic" => ctx.text("Arithmetic"), "booleans" => ctx.text("Booleans"), "bounds" => ctx.text("Bounds"), "comparison" => ctx.text("Comparison"), "integers" => ctx.text("Integers"), "logical" => ctx.text("Logical"), "ordering" => ctx.text("Ordering"), "strings" => ctx.text("Strings"), "types" => ctx.text("Types"), tag => Err(lowering::missing_selection(tag)), }))?;
parts.arg(|ctx| ctx.text(", &_left, &_right)"))?;
Ok(())
}),
            "hasEndpoint" => ctx.ok(|ctx| ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("("))?;
parts.arg(|ctx| ctx.child(_node, "value", _depth, self))?;
parts.arg(|ctx| ctx.text(")?.is_some()"))?;
Ok(())
})),
            "if" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("if ("))?;
parts.arg(|ctx| ctx.child(_node, "condition", _depth, self))?;
parts.arg(|ctx| ctx.text(")? { "))?;
parts.arg(|ctx| ctx.child(_node, "then", _depth, self))?;
parts.arg(|ctx| ctx.text(" } else { "))?;
parts.arg(|ctx| ctx.child(_node, "else", _depth, self))?;
parts.arg(|ctx| ctx.text(" }"))?;
Ok(())
}),
            "inputEndpoint" => ctx.ok(|ctx| ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("crate::unification::endpoint(&"))?;
parts.arg(|ctx| ctx.select(_node, "input", |ctx, tag| match tag { "left" => ctx.text("_left"), "right" => ctx.text("_right"), tag => Err(lowering::missing_selection(tag)), }))?;
parts.arg(|ctx| ctx.text(", crate::unification::Endpoint::"))?;
parts.arg(|ctx| ctx.select(_node, "endpoint", |ctx, tag| match tag { "lower" => ctx.text("Lower"), "upper" => ctx.text("Upper"), tag => Err(lowering::missing_selection(tag)), }))?;
parts.arg(|ctx| ctx.text(")"))?;
Ok(())
})),
            "inputValue" => ctx.ok(|ctx| ctx.concat(|parts| {
parts.arg(|ctx| ctx.select(_node, "input", |ctx, tag| match tag { "left" => ctx.text("_left"), "right" => ctx.text("_right"), tag => Err(lowering::missing_selection(tag)), }))?;
parts.arg(|ctx| ctx.text(".clone()"))?;
Ok(())
})),
            "integerSum" => ctx.text("crate::runtime::integer_sum(&_left, &_right)"),
            "kindIs" => ctx.ok(|ctx| ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("crate::unification::kind_is(&"))?;
parts.arg(|ctx| ctx.select(_node, "input", |ctx, tag| match tag { "left" => ctx.text("_left"), "right" => ctx.text("_right"), tag => Err(lowering::missing_selection(tag)), }))?;
parts.arg(|ctx| ctx.text(", crate::unification::Kind::"))?;
parts.arg(|ctx| ctx.select(_node, "kind", |ctx, tag| match tag { "bool" => ctx.text("Bool"), "boolean" => ctx.text("Boolean"), "bounds" => ctx.text("Bounds"), "int" => ctx.text("Int"), "integer" => ctx.text("Integer"), "string" => ctx.text("String"), "text" => ctx.text("Text"), tag => Err(lowering::missing_selection(tag)), }))?;
parts.arg(|ctx| ctx.text(")"))?;
Ok(())
})),
            "letBounds" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("{ let (_lower, _upper) = (("))?;
parts.arg(|ctx| ctx.child(_node, "lower", _depth, self))?;
parts.arg(|ctx| ctx.text(")?, ("))?;
parts.arg(|ctx| ctx.child(_node, "upper", _depth, self))?;
parts.arg(|ctx| ctx.text(")?); "))?;
parts.arg(|ctx| ctx.child(_node, "result", _depth, self))?;
parts.arg(|ctx| ctx.text(" }"))?;
Ok(())
}),
            "local" => ctx.ok(|ctx| ctx.select(_node, "endpoint", |ctx, tag| match tag { "lower" => ctx.text("_lower"), "upper" => ctx.text("_upper"), tag => Err(lowering::missing_selection(tag)), })),
            "or" => ctx.ok(|ctx| ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("("))?;
parts.arg(|ctx| ctx.child(_node, "left", _depth, self))?;
parts.arg(|ctx| ctx.text(")? || ("))?;
parts.arg(|ctx| ctx.child(_node, "right", _depth, self))?;
parts.arg(|ctx| ctx.text(")?"))?;
Ok(())
})),
            "program" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("// Generated by bootstrap-unify-codegen. Do not edit by hand.\nuse crate::{Error, runtime::Value};\npub fn "))?;
parts.arg(|ctx| ctx.select(_node, "function", |ctx, tag| match tag { "equal" => ctx.text("equal"), "greater" => ctx.text("greater"), "greaterEqual" => ctx.text("greater_equal"), "less" => ctx.text("less"), "lessEqual" => ctx.text("less_equal"), "logicalAnd" => ctx.text("logical_and"), "logicalOr" => ctx.text("logical_or"), "notEqual" => ctx.text("not_equal"), "sum" => ctx.text("sum"), "unify" => ctx.text("unify"), tag => Err(lowering::missing_selection(tag)), }))?;
parts.arg(|ctx| ctx.text("(_left: Value, _right: Value) -> Result<Value, Error> {\n"))?;
parts.arg(|ctx| ctx.child(_node, "result", _depth, self))?;
parts.arg(|ctx| ctx.text("\n}\n"))?;
Ok(())
}),
            "same" => ctx.text("Ok::<_, Error>(_left == _right)"),
            "textCompare" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("crate::unification::text_compare(&_left, &_right, crate::unification::Relation::"))?;
parts.arg(|ctx| ctx.select(_node, "relation", |ctx, tag| match tag { "equal" => ctx.text("Equal"), "greater" => ctx.text("Greater"), "greaterEqual" => ctx.text("GreaterEqual"), "less" => ctx.text("Less"), "lessEqual" => ctx.text("LessEqual"), tag => Err(lowering::missing_selection(tag)), }))?;
parts.arg(|ctx| ctx.text(")"))?;
Ok(())
}),
            "textConcat" => ctx.text("crate::runtime::text_concat(&_left, &_right)"),
            op => Err(lowering::missing_case(op)),
        }
    }
}

pub fn lower(node: &Value) -> Result<String, Error> {
    Context::run(node, &Compiled)
}
