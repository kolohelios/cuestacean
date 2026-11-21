package bootstrap

{
	let Node = {op: "path", value: "node"}
	let Field = {op: "string", field: "field"}
	let Context = {op: "path", value: "_context"}
	let V = {op: "try", value: {op: "group", value: {op: "lower", field: "value"}}}
	cases: {
		"null": {op: "ok", value: {op: "path", value: "runtime::Value::Null"}}
		top: {op: "ok", value: {op: "path", value: "runtime::Value::Top"}}
		int: {op: "ok", value: {op: "path", value: "runtime::Value::Int"}}
		bool: {op: "ok", value: {op: "path", value: "runtime::Value::Bool"}}
		stringType: {op: "ok", value: {op: "path", value: "runtime::Value::String"}}
		text: {op: "call", path: "runtime::text", args: [Node, Field]}
		list: {op: "call", path: "runtime::list", args: [Node, Field]}
		record: {op: "call", path: "runtime::record", args: [Node, Field]}
		recordExpression: {op: "call", path: "runtime::record_expression", args: [Node, Field, Context, {op: "path", value: "evaluate"}]}
		listExpression: {op: "call", path: "runtime::list_expression", args: [Node, Field, Context, {op: "path", value: "evaluate"}]}
		boolean: {op: "ok", value: {op: "call", path: "runtime::Value::Boolean", args: [{op: "boolean", field: "value"}]}}
		field: {op: "call", path: "runtime::integer", args: [Node, Field]}
		evaluate: {op: "call", path: "evaluate", args: [{op: "try", value: {op: "call", path: "runtime::field", args: [Node, Field]}}, Context]}
		conjunction: {op: "call", path: "runtime::conjunction", args: [Node, Field, Context, {op: "path", value: "evaluate"}]}
		slice: {op: "call", path: "runtime::slice_fields", args: [Node,
			{op: "string", field: "operandField"}, {op: "string", field: "lowerField"}, {op: "string", field: "upperField"},
			Context, {op: "path", value: "evaluate"},
		]}
		// Isolate wide ternary temporaries from every recursive match frame.
		listSlice: {op: "concat", parts: [{op: "text", value: "(|_context: &mut runtime::EvaluationContext| "},
			{op: "call", path: "runtime::list_slice", args: [V,
				{op: "try", value: {op: "group", value: {op: "lower", field: "lower"}}},
				{op: "try", value: {op: "group", value: {op: "lower", field: "upper"}}},
			]}, {op: "text", value: ")(_context)"},
		]}
	}
}
