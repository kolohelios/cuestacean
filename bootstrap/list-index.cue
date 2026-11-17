package bootstrap

#BinaryControlProgram & {
	function: "listIndex"
	result: {
		op: "if"
		condition: {op: "and", left: {op: "kindIs", input: "left", kind: "list"}, right: {op: "kindIs", input: "right", kind: "integer"}}
		then: {
			op: "if"
			condition: {op: "and", left: {
				op: "compare", left: {op: "input", input: "right", endpoint: "lower"}, right: {op: "integer", value: 0}, relation: "greaterEqual"
			}, right: {
				op: "compare", left: {op: "input", input: "right", endpoint: "lower"}, right: {op: "listCount"}, relation: "less"
			}}
			then: {op: "listElement"}
			else: {op: "fail", reason: "indexRange"}
		}
		else: {op: "fail", reason: "indexOperands"}
	}
}
