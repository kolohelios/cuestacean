package bootstrap

#UnifyProgram & {formatVersion: 1, function: "lowerBound", result: {
	op: "bounds"
	lower: {op: "choose", condition: {op: "kindIs", input: "left", kind: "integer"}, then: {op: "empty"}, else: {op: "input", input: "right", endpoint: "lower"}}
	upper: {op: "empty"}
}}
