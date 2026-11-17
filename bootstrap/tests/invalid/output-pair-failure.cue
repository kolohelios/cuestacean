package bootstrap

#UnifyProgram & {formatVersion: 1, function: "integerOutput", result: {
	op: "if", condition: {op: "kindIs", input: "left", kind: "integer"}
	then: {op: "input", input: "left"}
	else: {op: "fail", reason: "integers"}
}}
