package bootstrap

#UnifyProgram & {
	formatVersion: 1
	function: "lowerBound"
	result: {
		op: "bounds"
		lower: {
			op: "choose", condition: {op: "kindIs", input: "left", kind: "integer"}
			then: {op: "integer", value: -7}
			else: {op: "empty"}
		}
		upper: {op: "input", input: "left", endpoint: "upper"}
	}
}
