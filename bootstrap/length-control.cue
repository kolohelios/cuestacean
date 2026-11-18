package bootstrap

#UnaryControlProgram & {
	function: "length"
	result: {
		op: "if", condition: {op: "kindIs", input: "left", kind: "text"}
		then: {op: "textLength"}
		else: {
			op: "if", condition: {op: "kindIs", input: "left", kind: "list"}
			then: {op: "listCount"}
			else: {op: "if", condition: {op: "kindIs", input: "left", kind: "record"}, then: {op: "recordCount"}, else: {op: "fail", reason: "lengthOperand"}}
		}
	}
}
