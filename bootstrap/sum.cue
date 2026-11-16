package bootstrap

#UnifyProgram & {
	function: "sum"
	result: {
		op: "if", condition: {op: "kindIs", input: "left", kind: "integer"}
		then: {op: "if", condition: {op: "kindIs", input: "right", kind: "integer"}, then: {op: "integerSum"}, else: {op: "fail", reason: "arithmetic"}}
		else: {
			op: "if", condition: {op: "kindIs", input: "left", kind: "text"}
			then: {op: "if", condition: {op: "kindIs", input: "right", kind: "text"}, then: {op: "textConcat"}, else: {op: "fail", reason: "arithmetic"}}
			else: {op: "fail", reason: "arithmetic"}
		}
	}
}
