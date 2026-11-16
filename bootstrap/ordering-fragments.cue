package bootstrap

#OrderingProgram: #BinaryControlProgram & {
	_relation: "less" | "lessEqual" | "greater" | "greaterEqual"
	let Result = {op: "boolean", value: true}
	let Other = {op: "boolean", value: false}
	result: {
		op: "if", condition: {op: "and", left: {op: "kindIs", input: "left", kind: "integer"}, right: {op: "kindIs", input: "right", kind: "integer"}}
		then: {op: "if", condition: {op: "compare", left: {op: "input", input: "left", endpoint: "lower"}, right: {op: "input", input: "right", endpoint: "lower"}, relation: _relation}, then: Result, else: Other}
		else: {
			op: "if", condition: {op: "and", left: {op: "kindIs", input: "left", kind: "text"}, right: {op: "kindIs", input: "right", kind: "text"}}
			then: {op: "if", condition: {op: "textCompare", relation: _relation}, then: Result, else: Other}
			else: {op: "fail", reason: "ordering"}
		}
	}
}
