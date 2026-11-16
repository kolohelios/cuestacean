package bootstrap

// Test-only unary condition traversal, not logical not semantics.
#UnifyProgram & {function: "logicalNot", result: {
	op: "if", condition: {
		op: "and", left: {op: "kindIs", input: "left", kind: "boolean"}
		right: {op: "or", left: {op: "booleanIs", input: "left", value: true}, right: {op: "booleanIs", input: "left", value: false}}
	}
	then: {op: "boolean", value: false}
	else: {op: "boolean", value: true}
}}
