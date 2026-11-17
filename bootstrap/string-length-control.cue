package bootstrap

#UnaryControlProgram & {
	function: "stringLength"
	result: {
		op: "if", condition: {op: "kindIs", input: "left", kind: "text"}
		then: {op: "textLength"}
		else: {op: "fail", reason: "length"}
	}
}
