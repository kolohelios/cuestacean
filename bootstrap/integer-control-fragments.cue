package bootstrap

#IntegerBinaryControl: #BinaryControlProgram & {
	_operation: "difference" | "product"
	result: {
		op: "if", condition: {op: "and", left: {op: "kindIs", input: "left", kind: "integer"}, right: {op: "kindIs", input: "right", kind: "integer"}}
		then: {op: "integerPrimitive", operation: _operation}
		else: {op: "fail", reason: "arithmetic"}
	}
}
