package bootstrap

#UnifyProgram & {function: "logicalNot", result: {
	op: "if", condition: {op: "kindIs", input: "left", kind: "boolean"}
	then: {op: "if", condition: {op: "booleanIs", input: "left", value: true}, then: {op: "boolean", value: false}, else: {op: "boolean", value: true}}
	else: {op: "fail", reason: "logical"}
}}
