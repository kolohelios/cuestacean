package bootstrap

#OutputControl & {
	function: "integerOutput"
	_concrete: {
		op: "if", condition: {op: "kindIs", input: "left", kind: "integer"}
		then: {op: "input", input: "left"}
		else: {op: "fail", reason: "integerOutput"}
	}
}
