package bootstrap

#UnaryControlProgram & {
	function: "recordLength"
	result: {op: "if", condition: {op: "kindIs", input: "left", kind: "record"}, then: {op: "recordCount"}, else: {op: "fail", reason: "recordLength"}}
}
