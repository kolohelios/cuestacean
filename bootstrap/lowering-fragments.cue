package bootstrap

#InstructionModuleText: close({header: string, footer: string})

#InstructionModuleFirst: close({op: string})
#InstructionModuleStep: close({op: string})

#InstructionBinaryTables: close({
	aliases: close({
		index:       string, difference: string, product:    string, divide:    string, modulo: string, quotient: string
		recordIndex: string, listIndex:  string, logicalAnd: string, logicalOr: string, equal:  string
	})
	select: close({
		op: string, field: string
		cases: close({
			index: close({op: string, value: string}), difference: close({op: string, value: string})
			product: close({op: string, value: string}), divide: close({op: string, value: string})
			modulo: close({op: string, value: string}), quotient: close({op: string, value: string})
			recordIndex: close({op: string, value: string}), listIndex: close({op: string, value: string})
			logicalAnd: close({op: string, value: string}), logicalOr: close({op: string, value: string})
			equal: close({op: string, value: string})
		})
	})
})

#InstructionRightField: close({field: string})

// Executable recursion/Result layout, not Rust's instruction enum dispatch.
#InstructionLowering: #LoweringPolicy & {
	let Node = {op: "path", value: "node"}
	let Field = {op: "string", field: "field"}
	let Context = {op: "path", value: "_context"}
	let L = {op: "try", value: {op: "group", value: {op: "lower", field: "left"}}}

	// The alternate policy overrides only sum's default right-child selection.
	let R = {op: "try", value: {op: "group", value: {op: "lower", field: #InstructionRightField.field}}}
	let V = {op: "try", value: {op: "group", value: {op: "lower", field: "value"}}}
	aliases: {
		lowerExclusiveBound: _, upperExclusiveBound: _, lowerBound:   _, upperBound: _, negate:       _
		logicalNot:          _, length:              _, stringLength: _, listLength: _, recordLength: _
		#InstructionBinaryTables.aliases
	}
	module: [{
		op:    "text"
		value: #InstructionModuleText.header + "\n"
	}, #InstructionModuleFirst, {
		op: "whileRules"
		body: [{op: "kind"}, {op: "text", value: " => "}, {op: "expression"},
			{op: "text", value: ",\n"}, #InstructionModuleStep]
	}, {
		op:    "text"
		value: #InstructionModuleText.footer
	}]
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
		unaryRuntime: _
		binaryRuntime: {op: "concat", parts: [
			#InstructionBinaryTables.select,
			{op: "text", value: "("}, L, {op: "text", value: ", "}, R, {op: "text", value: ")"},
		]}
		sum: {op: "call", path: "runtime::sum", args: [L, R]}
		conjunction: {op: "call", path: "runtime::conjunction", args: [Node, Field, Context, {op: "path", value: "evaluate"}]}
		slice: {op: "call", path: "runtime::slice_fields", args: [Node,
			{op: "string", field: "operandField"}, {op: "string", field: "lowerField"}, {op: "string", field: "upperField"},
			Context, {op: "path", value: "evaluate"},
		]}
		less: {op: "call", path: "runtime::less", args: [L, R]}
		lessEqual: {op: "call", path: "runtime::less_equal", args: [L, R]}
		greater: {op: "call", path: "runtime::greater", args: [L, R]}
		greaterEqual: {op: "call", path: "runtime::greater_equal", args: [L, R]}
		notEqual: {op: "call", path: "runtime::not_equal", args: [L, R]}
		remainder: {op: "call", path: "runtime::remainder", args: [L, R]}
		unify: {op: "concat", parts: [{op: "text", value: "(|_context: &mut runtime::EvaluationContext| "},
			{op: "call", path: "runtime::unify", args: [L, R]}, {op: "text", value: ")(_context)"},
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
