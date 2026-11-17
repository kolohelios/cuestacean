package bootstrap

#UnifyProgram & {
	formatVersion: 1
	function: "negate"
	result: {op: "if", condition: {op: "kindIs", input: "left", kind: "integer"}, then: {op: "integer", value: 0}, else: {op: "integerPrimitive", operation: "divide"}}
}
