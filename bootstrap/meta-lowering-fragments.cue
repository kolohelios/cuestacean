package bootstrap

// Expression emission only: the native driver shell and module emitter remain.
#MetaLowering: #LoweringPolicy & {
	aliases: {
		string:      "emitField", boolean:      "emitField", unsigned: "emitField", signed: "emitField"
		bytePattern: "emitField", charPattern:  "emitField"
		path:        "emitValue", text:         *"emitValue" | _
		lower:       "emitRecursive", list:     "emitRecursive"
		ok:          "emitUnary", group:        "emitUnary", "try":        "emitUnary"
		ifEmpty:     "emitConditional", ifZero: "emitConditional", ifTrue: "emitConditional"
	}
	cases: {
		emitField: {op: "concat", parts: [{op: "text", value: "ctx."}, {op: "identifier", field: "op", snake: true},
			{op: "text", value: "(_node, "}, {op: "string", field: "field"}, {op: "text", value: ")"}]}
		emitValue: {op: "concat", parts: [{op: "text", value: "ctx."}, {op: "identifier", field: "op"},
			{op: "text", value: "("}, {op: "string", field: "value"}, {op: "text", value: ")"}]}
		emitRecursive: {op: "concat", parts: [{op: "text", value: "ctx."}, {op: "select", field: "op", cases: {
			lower: {op: "text", value: "child"}, list: {op: "text", value: "list"}
		}}, {op: "text", value: "(_node, "}, {op: "string", field: "field"}, {op: "text", value: ", _depth, self)"}]}
		emitUnary: {op: "concat", parts: [{op: "text", value: "ctx."}, {op: "select", field: "op", cases: {
			ok: {op: "text", value: "ok"}, group: {op: "text", value: "group"}, "try": {op: "text", value: "try_value"}
		}}, {op: "text", value: "(|ctx| "}, {op: "lower", field: "value"}, {op: "text", value: ")"}]}
		emitConditional: {op: "concat", parts: [
			{op: "text", value: "ctx."}, {op: "identifier", field: "op", snake: true}, {op: "text", value: "(_node, "},
			{op: "string", field: "field"}, {op: "text", value: ", |ctx| "}, {op: "lower", field: "then"},
			{op: "text", value: ", |ctx| "}, {op: "lower", field: "else"}, {op: "text", value: ")"},
		]}
		join: {op: "concat", parts: [{op: "text", value: "ctx.join(_node, "}, {op: "string", field: "field"},
			{op: "text", value: ", "}, {op: "string", field: "separator"}, {op: "text", value: ", _depth, self)"}]}
		bind: {op: "concat", parts: [{op: "text", value: "ctx.bind(|ctx| "}, {op: "lower", field: "value"},
			{op: "text", value: ", |ctx, _render_"}, {op: "identifier", field: "name"}, {op: "text", value: "| "},
			{op: "lower", field: "body"}, {op: "text", value: ")"}]}
		get: {op: "concat", parts: [{op: "text", value: "ctx.rendered(_render_"}, {op: "identifier", field: "name"}, {op: "text", value: ")"}]}
		ifEqual: {op: "concat", parts: [{op: "text", value: "ctx.if_equal(|ctx| "}, {op: "lower", field: "left"},
			{op: "text", value: ", |ctx| "}, {op: "lower", field: "right"}, {op: "text", value: ", |ctx| "},
			{op: "lower", field: "then"}, {op: "text", value: ", |ctx| "}, {op: "lower", field: "else"}, {op: "text", value: ")"}]}
		identifier: {op: "concat", parts: [{op: "text", value: "ctx.identifier(_node, "}, {op: "string", field: "field"},
			{op: "text", value: ", "}, {op: "boolean", field: "snake"}, {op: "text", value: ")"}]}
		select: {op: "concat", parts: [{op: "text", value: "ctx.select(_node, "}, {op: "string", field: "field"},
			{op: "text", value: ", |ctx, tag| match tag { "}, {op: "list", field: "cases"},
			{op: "text", value: "tag => Err(lowering::missing_selection(tag)), })"}]}
		selectCase: {op: "concat", parts: [{op: "string", field: "tag"}, {op: "text", value: " => "},
			{op: "lower", field: "value"}, {op: "text", value: ", "}]}
		concat: {op: "concat", parts: [{op: "text", value: "ctx.concat(|"},
			{op: "ifEmpty", field: "parts", then: {op: "text", value: "_parts"}, else: {op: "text", value: "parts"}},
			{op: "text", value: "| {\n"}, {op: "list", field: "parts"}, {op: "text", value: "Ok(())\n})"}]}
		concatArgument: {op: "concat", parts: [{op: "text", value: "parts.arg(|ctx| "}, {op: "lower", field: "value"}, {op: "text", value: ")?;\n"}]}
		call: {op: "concat", parts: [{op: "text", value: "ctx.call("}, {op: "string", field: "path"}, {op: "text", value: ", |"},
			{op: "ifEmpty", field: "args", then: {op: "text", value: "_call"}, else: {op: "text", value: "call"}},
			{op: "text", value: "| { "}, {op: "list", field: "args"}, {op: "text", value: "Ok(()) })"}]}
		callArgument: {op: "concat", parts: [{op: "text", value: "call.arg(|ctx| "}, {op: "lower", field: "value"}, {op: "text", value: ")?; "}]}
	}
}
