package bootstrap

#UnaryControlProgram & {function: "negate", result: {
	op: "if", condition: {op: "kindIs", input: "left", kind: "integer"}
	then: {op: "integerPrimitive", operation: "negate"}
	else: {op: "fail", reason: "arithmetic"}
}}
