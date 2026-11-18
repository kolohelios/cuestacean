package bootstrap

#BinaryControlProgram & {
	function: "recordIndex"
	result: {
		op: "if", condition: {op: "and", left: {op: "kindIs", input: "left", kind: "record"}, right: {op: "kindIs", input: "right", kind: "text"}}
		then: {op: "if", condition: {op: "recordHas"}, then: {op: "recordGet"}, else: {op: "fail", reason: "recordMissing"}}
		else: {op: "fail", reason: "recordIndexOperands"}
	}
}
