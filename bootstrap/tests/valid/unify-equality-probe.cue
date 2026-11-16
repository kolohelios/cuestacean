package bootstrap

// Small compiler-policy probe, not the production scalar solver.
#UnifyProgram & {result: {
	op: "if", condition: {op: "and", left: {op: "kindIs", input: "left", kind: "boolean"}, right: {op: "kindIs", input: "right", kind: "boolean"}}
	then: {op: "if", condition: {op: "same"}, then: {op: "input", input: "left"}, else: {op: "fail", reason: "booleans"}}
	else: {
		op: "if", condition: {op: "and", left: {op: "kindIs", input: "left", kind: "integer"}, right: {op: "kindIs", input: "right", kind: "integer"}}
		then: {
			op: "if", condition: {op: "compare", left: {op: "input", input: "left", endpoint: "lower"}, right: {op: "input", input: "right", endpoint: "lower"}, relation: "equal"}
			then: {op: "input", input: "left"}, else: {op: "fail", reason: "integers"}
		}
		else: {op: "fail", reason: "types"}
	}
}}
