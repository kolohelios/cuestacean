package bootstrap

#UnaryControlProgram & {
	function: "listLength"
	result: {
		op: "if", condition: {op: "kindIs", input: "left", kind: "list"}
		then: {op: "listCount"}
		else: {op: "fail", reason: "listLength"}
	}
}
