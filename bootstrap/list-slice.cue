package bootstrap

#TernaryControlProgram & {
	function: "listSlice"
	let Low = {op: "input", input: "right", endpoint: "lower"}
	let High = {op: "input", input: "third", endpoint: "lower"}
	result: {
		op: "if"
		condition: {op: "and", left: {op: "kindIs", input: "left", kind: "list"}, right: {
			op: "and", left: {op: "kindIs", input: "right", kind: "integer"}, right: {op: "kindIs", input: "third", kind: "integer"}
		}}
		then: {
			op: "if"
			condition: {op: "and", left: {op: "compare", left: Low, right: {op: "integer", value: 0}, relation: "greaterEqual"}, right: {
				op: "and", left: {op: "compare", left: Low, right: High, relation: "lessEqual"}, right: {op: "compare", left: High, right: {op: "listCount"}, relation: "lessEqual"}
			}}
			then: {op: "listSlice"}
			else: {op: "fail", reason: "listSliceRange"}
		}
		else: {op: "fail", reason: "listSliceOperands"}
	}
}
