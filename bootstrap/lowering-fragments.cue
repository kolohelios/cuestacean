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
#InstructionRightOperand: close({op: string, value: close({op: string, value: close({op: string, field: string})})})

// Executable recursion/Result layout, not Rust's instruction enum dispatch.
#InstructionLowering: #LoweringPolicy & {
	let L = {op: "try", value: {op: "group", value: {op: "lower", field: "left"}}}

	// The alternate policy overrides only sum's default right-child selection.
	let R = #InstructionRightOperand
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
		"null": _
		top: _
		int: _
		bool: _
		stringType: _
		text: _
		list: _
		record: _
		recordExpression: _
		listExpression: _
		boolean: _
		field: _
		evaluate: _
		unaryRuntime: _
		binaryRuntime: {op: "concat", parts: [
			#InstructionBinaryTables.select,
			{op: "text", value: "("}, L, {op: "text", value: ", "}, R, {op: "text", value: ")"},
		]}
		sum: {op: "call", path: "runtime::sum", args: [L, R]}
		conjunction: _
		slice: _
		less: {op: "call", path: "runtime::less", args: [L, R]}
		lessEqual: {op: "call", path: "runtime::less_equal", args: [L, R]}
		greater: {op: "call", path: "runtime::greater", args: [L, R]}
		greaterEqual: {op: "call", path: "runtime::greater_equal", args: [L, R]}
		notEqual: {op: "call", path: "runtime::not_equal", args: [L, R]}
		remainder: {op: "call", path: "runtime::remainder", args: [L, R]}
		unify: {op: "concat", parts: [{op: "text", value: "(|_context: &mut runtime::EvaluationContext| "},
			{op: "call", path: "runtime::unify", args: [L, R]}, {op: "text", value: ")(_context)"},
		]}
		listSlice: _
	}
}
