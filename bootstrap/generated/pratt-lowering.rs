// Generated lowering driver. Do not edit.
use bootstrap_executor::lowering::{self, Context, Driver, Error};
use serde_json::Value;

pub struct Compiled;

impl Driver for Compiled {
    fn render(&self, ctx: &mut Context, _node: &Value, _depth: usize) -> Result<String, Error> {
        match lowering::instruction_op(_node)? {
            "action" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("_frame.step()?;\n_frame."))?;
parts.arg(|ctx| ctx.select(_node, "action", |ctx, tag| match tag { "advanceBound" => ctx.text("advance_bound"), "advanceCall" => ctx.text("advance_call"), "advanceGroup" => ctx.text("advance_group"), "advanceIndex" => ctx.text("advance_index"), "advanceList" => ctx.text("advance_list"), "advanceListTail" => ctx.text("advance_list_tail"), "advanceNegation" => ctx.text("advance_negation"), "advanceNot" => ctx.text("advance_not"), "advanceOperator" => ctx.text("advance_operator"), "afterOperand" => ctx.text("after_operand"), "afterWordOperator" => ctx.text("after_word_operator"), "appendItem" => ctx.text("append_item"), "appendListElement" => ctx.text("append_list_element"), "buildBinary" => ctx.text("build_binary"), "buildList" => ctx.text("build_list"), "buildNegation" => ctx.text("build_negation"), "buildNot" => ctx.text("build_not"), "closeGroup" => ctx.text("close_group"), "closeIndex" => ctx.text("close_index"), "evaluateItem" => ctx.text("evaluate_item"), "finishList" => ctx.text("finish_list"), "firstItem" => ctx.text("first_item"), "lastItem" => ctx.text("last_item"), "lookupOperator" => ctx.text("lookup_operator"), "nextItem" => ctx.text("next_item"), "parseAtom" => ctx.text("parse_atom"), "parseBound" => ctx.text("parse_bound"), "parseInteger" => ctx.text("parse_integer"), "parseRight" => ctx.text("parse_right"), "parseString" => ctx.text("parse_string"), "previousItem" => ctx.text("previous_item"), "primary" => ctx.text("primary"), "probeListEnd" => ctx.text("probe_list_end"), "rejectItem" => ctx.text("reject_item"), "scanBound" => ctx.text("scan_bound"), "scanGroup" => ctx.text("scan_group"), "scanIdentifier" => ctx.text("scan_identifier"), "scanIndex" => ctx.text("scan_index"), "scanList" => ctx.text("scan_list"), "scanNegation" => ctx.text("scan_negation"), "scanNot" => ctx.text("scan_not"), "scanString" => ctx.text("scan_string"), "whitespace" => ctx.text("whitespace"), tag => Err(lowering::missing_selection(tag)), }))?;
parts.arg(|ctx| ctx.text("()?;\n"))?;
Ok(())
}),
            "add" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("_frame.binding_add("))?;
parts.arg(|ctx| ctx.child(_node, "left", _depth, self))?;
parts.arg(|ctx| ctx.text(", "))?;
parts.arg(|ctx| ctx.child(_node, "right", _depth, self))?;
parts.arg(|ctx| ctx.text(")?"))?;
Ok(())
}),
            "and" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.child(_node, "left", _depth, self))?;
parts.arg(|ctx| ctx.text(" && "))?;
parts.arg(|ctx| ctx.child(_node, "right", _depth, self))?;
Ok(())
}),
            "atLeast" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.child(_node, "left", _depth, self))?;
parts.arg(|ctx| ctx.text(" >= "))?;
parts.arg(|ctx| ctx.child(_node, "right", _depth, self))?;
Ok(())
}),
            "boolean" => ctx.boolean(_node, "value"),
            "groupOpen" => ctx.text("_frame.group_open()?"),
            "hasBound" => ctx.text("_frame.has_bound()?"),
            "hasCall" => ctx.text("_frame.has_call()?"),
            "hasIdentifier" => ctx.text("_frame.has_identifier()?"),
            "hasIndex" => ctx.text("_frame.has_index()?"),
            "hasItem" => ctx.text("_frame.has_item()?"),
            "hasList" => ctx.text("_frame.has_list()?"),
            "hasListElement" => ctx.text("_frame.has_list_element()?"),
            "hasNegation" => ctx.text("_frame.has_negation()?"),
            "hasNot" => ctx.text("_frame.has_not()?"),
            "hasOperator" => ctx.text("_frame.has_operator()"),
            "hasString" => ctx.text("_frame.has_string()?"),
            "if" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("_frame.step()?;\nif "))?;
parts.arg(|ctx| ctx.child(_node, "condition", _depth, self))?;
parts.arg(|ctx| ctx.text(" {\n"))?;
parts.arg(|ctx| ctx.list(_node, "then", _depth, self))?;
parts.arg(|ctx| ctx.if_empty(_node, "else", |ctx| ctx.text(""), |ctx| ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("} else {\n"))?;
parts.arg(|ctx| ctx.list(_node, "else", _depth, self))?;
Ok(())
})))?;
parts.arg(|ctx| ctx.text("}\n"))?;
Ok(())
}),
            "integer" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.unsigned(_node, "value"))?;
parts.arg(|ctx| ctx.text("u8"))?;
Ok(())
}),
            "itemKindIs" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("_frame.item_kind_is(crate::unification::Kind::"))?;
parts.arg(|ctx| ctx.identifier(_node, "kind", false, true))?;
parts.arg(|ctx| ctx.text(")?"))?;
Ok(())
}),
            "leftAssociative" => ctx.text("_frame.left_associative()?"),
            "minimum" => ctx.text("_frame.minimum()"),
            "parseGroup" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("_frame.step()?;\n_frame.parse_group("))?;
parts.arg(|ctx| ctx.child(_node, "minimum", _depth, self))?;
parts.arg(|ctx| ctx.text(")?;\n"))?;
Ok(())
}),
            "parseIndex" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("_frame.step()?;\n_frame.parse_index("))?;
parts.arg(|ctx| ctx.child(_node, "minimum", _depth, self))?;
parts.arg(|ctx| ctx.text(")?;\n"))?;
Ok(())
}),
            "parseListElement" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("_frame.step()?;\n_frame.parse_list_element("))?;
parts.arg(|ctx| ctx.child(_node, "minimum", _depth, self))?;
parts.arg(|ctx| ctx.text(")?;\n"))?;
Ok(())
}),
            "parseNegation" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("_frame.step()?;\n_frame.parse_negation("))?;
parts.arg(|ctx| ctx.child(_node, "minimum", _depth, self))?;
parts.arg(|ctx| ctx.text(")?;\n"))?;
Ok(())
}),
            "parseNot" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("_frame.step()?;\n_frame.parse_not("))?;
parts.arg(|ctx| ctx.child(_node, "minimum", _depth, self))?;
parts.arg(|ctx| ctx.text(")?;\n"))?;
Ok(())
}),
            "precedence" => ctx.text("_frame.precedence()?"),
            "program" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.select(_node, "target", |ctx, tag| match tag { "list" => ctx.text("// Generated by bootstrap-pratt-codegen. Do not edit by hand.\nuse crate::{Error, runtime::ListFrame};\npub fn execute(_frame: &mut ListFrame<'_>) -> Result<(), Error> {\n"), "parser" => ctx.text("// Generated by bootstrap-pratt-codegen. Do not edit by hand.\nuse crate::parser::{Error, Frame};\npub fn execute(_frame: &mut Frame<'_, '_>) -> Result<(), Error> {\n"), tag => Err(lowering::missing_selection(tag)), }))?;
parts.arg(|ctx| ctx.list(_node, "body", _depth, self))?;
parts.arg(|ctx| ctx.select(_node, "tail", |ctx, tag| match tag { "return" => ctx.text("}\n"), "value" => ctx.text("Ok(())\n}\n"), tag => Err(lowering::missing_selection(tag)), }))?;
Ok(())
}),
            "return" => ctx.select(_node, "position", |ctx, tag| match tag { "early" => ctx.text("_frame.step()?;\nreturn Ok(());\n"), "tail" => ctx.text("_frame.step()?;\nOk(())\n"), tag => Err(lowering::missing_selection(tag)), }),
            "setBindingPower" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("_frame.step()?;\n_frame.set_binding_power("))?;
parts.arg(|ctx| ctx.child(_node, "value", _depth, self))?;
parts.arg(|ctx| ctx.text(")?;\n"))?;
Ok(())
}),
            "tokenLength" => ctx.text("_frame.token_length()?"),
            "while" => ctx.concat(|parts| {
parts.arg(|ctx| ctx.text("while { _frame.step()?; "))?;
parts.arg(|ctx| ctx.child(_node, "condition", _depth, self))?;
parts.arg(|ctx| ctx.text(" } {\n"))?;
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
