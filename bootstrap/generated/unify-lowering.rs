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
parts.arg(|ctx| ctx.identifier(_node, "relation", false, true))?;
parts.arg(|ctx| ctx.text(")"))?;
Ok(())
}),
            "empty" => ctx.text("Ok::<_, Error>(None::<i64>)"),
            "fail" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("crate::unification::fail(crate::unification::Failure::"))?;
parts.arg(|ctx| ctx.identifier(_node, "reason", false, true))?;
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
parts.arg(|ctx| ctx.identifier(_node, "endpoint", false, true))?;
parts.arg(|ctx| ctx.text(")"))?;
Ok(())
})),
            "inputValue" => ctx.ok(|ctx| ctx.concat(|parts| {
parts.arg(|ctx| ctx.select(_node, "input", |ctx, tag| match tag { "left" => ctx.text("_left"), "right" => ctx.text("_right"), tag => Err(lowering::missing_selection(tag)), }))?;
parts.arg(|ctx| ctx.text(".clone()"))?;
Ok(())
})),
            "integerEndpoint" => ctx.ok(|ctx| ctx.call("Some", |call| { call.arg(|ctx| ctx.signed(_node, "value"))?; Ok(()) })),
            "integerPrimitive" => ctx.select(_node, "operation", |ctx, tag| match tag { "difference" => ctx.text("crate::runtime::integer_difference(&_left, &_right)"), "divide" => ctx.text("crate::runtime::integer_divide(&_left, &_right)"), "modulo" => ctx.text("crate::runtime::integer_modulo(&_left, &_right)"), "negate" => ctx.text("crate::runtime::integer_negate(&_left)"), "product" => ctx.text("crate::runtime::integer_product(&_left, &_right)"), "quotient" => ctx.text("crate::runtime::integer_quotient(&_left, &_right)"), "remainder" => ctx.text("crate::runtime::integer_remainder(&_left, &_right)"), tag => Err(lowering::missing_selection(tag)), }),
            "integerSum" => ctx.text("crate::runtime::integer_sum(&_left, &_right)"),
            "integerValue" => ctx.ok(|ctx| ctx.call("Value::Integer", |call| { call.arg(|ctx| ctx.signed(_node, "value"))?; Ok(()) })),
            "kindIs" => ctx.ok(|ctx| ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("crate::unification::kind_is(&"))?;
parts.arg(|ctx| ctx.select(_node, "input", |ctx, tag| match tag { "left" => ctx.text("_left"), "right" => ctx.text("_right"), tag => Err(lowering::missing_selection(tag)), }))?;
parts.arg(|ctx| ctx.text(", crate::unification::Kind::"))?;
parts.arg(|ctx| ctx.identifier(_node, "kind", false, true))?;
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
            "listCount" => ctx.text("crate::runtime::list_count(&_left)"),
            "listCountEndpoint" => ctx.text("crate::runtime::list_count(&_left).map(|value| crate::unification::endpoint(&value, crate::unification::Endpoint::Lower))"),
            "listElement" => ctx.text("crate::runtime::list_at(&_left, &_right)"),
            "local" => ctx.ok(|ctx| ctx.select(_node, "endpoint", |ctx, tag| match tag { "lower" => ctx.text("_lower"), "upper" => ctx.text("_upper"), tag => Err(lowering::missing_selection(tag)), })),
            "offset" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("crate::unification::offset_endpoint(("))?;
parts.arg(|ctx| ctx.child(_node, "value", _depth, self))?;
parts.arg(|ctx| ctx.text(")?, "))?;
parts.arg(|ctx| ctx.signed(_node, "delta"))?;
parts.arg(|ctx| ctx.text(")"))?;
Ok(())
}),
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
parts.arg(|ctx| ctx.identifier(_node, "function", true, false))?;
parts.arg(|ctx| ctx.if_true(_node, "unary", |ctx| ctx.text("(_left: Value) -> Result<Value, Error> {\n"), |ctx| ctx.text("(_left: Value, _right: Value) -> Result<Value, Error> {\n")))?;
parts.arg(|ctx| ctx.child(_node, "result", _depth, self))?;
parts.arg(|ctx| ctx.text("\n}\n"))?;
Ok(())
}),
            "recordCount" => ctx.text("crate::runtime::record_count(&_left)"),
            "recordGet" => ctx.text("crate::runtime::record_get(&_left, &_right)"),
            "recordHas" => ctx.text("crate::runtime::record_has(&_left, &_right)"),
            "same" => ctx.text("Ok::<_, Error>(_left == _right)"),
            "textCompare" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("crate::unification::text_compare(&_left, &_right, crate::unification::Relation::"))?;
parts.arg(|ctx| ctx.identifier(_node, "relation", false, true))?;
parts.arg(|ctx| ctx.text(")"))?;
Ok(())
}),
            "textConcat" => ctx.text("crate::runtime::text_concat(&_left, &_right)"),
            "textLength" => ctx.text("crate::runtime::text_length(&_left)"),
            "unaryFail" => ctx.select(_node, "reason", |ctx, tag| match tag { "arithmetic" => ctx.text("Err(Error::Arithmetic)"), "boundThreshold" => ctx.text("Err(Error::BoundThreshold)"), "emptyBound" => ctx.text("Err(Error::EmptyBound)"), "incompleteBool" => ctx.text("Err(Error::IncompleteBool)"), "incompleteBounds" => ctx.text("Err(Error::IncompleteBounds)"), "incompleteInt" => ctx.text("Err(Error::Incomplete)"), "incompleteString" => ctx.text("Err(Error::IncompleteString)"), "incompleteValue" => ctx.text("Err(Error::IncompleteValue)"), "integerOutput" => ctx.text("Err(Error::IntegerOutput)"), "length" => ctx.text("Err(Error::Length)"), "lengthOperand" => ctx.text("Err(Error::LengthOperand)"), "listLength" => ctx.text("Err(Error::ListLength)"), "logical" => ctx.text("Err(Error::Logical)"), "recordLength" => ctx.text("Err(Error::RecordLength)"), tag => Err(lowering::missing_selection(tag)), }),
            op => Err(lowering::missing_case(op)),
        }
    }
}

pub fn lower(node: &Value) -> Result<String, Error> {
    Context::run(node, &Compiled)
}
