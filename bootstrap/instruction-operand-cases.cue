package bootstrap

#InstructionOperandCases: {
	let L = {op: "try", value: {op: "group", value: {op: "lower", field: "left"}}}
	let R = #InstructionRightOperand
	leftOperand: L
	cases: {
		sum: {op: "call", path: "runtime::sum", args: [L, R]}
		less: {op: "call", path: "runtime::less", args: [L, R]}
		lessEqual: {op: "call", path: "runtime::less_equal", args: [L, R]}
		greater: {op: "call", path: "runtime::greater", args: [L, R]}
		greaterEqual: {op: "call", path: "runtime::greater_equal", args: [L, R]}
		notEqual: {op: "call", path: "runtime::not_equal", args: [L, R]}
		remainder: {op: "call", path: "runtime::remainder", args: [L, R]}
		unify: {op: "concat", parts: [{op: "text", value: "(|_context: &mut runtime::EvaluationContext| "},
			{op: "call", path: "runtime::unify", args: [L, R]}, {op: "text", value: ")(_context)"},
		]}
	}
}
