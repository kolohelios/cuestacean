package bootstrap

// Test lexical shadowing and short-circuiting, not a valid unification policy.
#UnifyProgram & {result: {
	op: "letBounds"
	lower: {op: "input", input: "left", endpoint: "lower"}
	upper: {op: "input", input: "left", endpoint: "upper"}
	result: {
		op: "letBounds"
		lower: {op: "local", endpoint: "upper"}
		upper: {op: "local", endpoint: "lower"}
		result: {
			op: "if"
			condition: {op: "and", left: {op: "hasEndpoint", value: {op: "empty"}}, right: {op: "compare", left: {op: "empty"}, right: {op: "empty"}, relation: "equal"}}
			then: {op: "fail", reason: "bounds"}
			else: {
				op: "if"
				condition: {
					op: "or"
					left: {op: "or", left: {op: "kindIs", input: "left", kind: "int"}, right: {op: "kindIs", input: "left", kind: "bounds"}}
					right: {op: "compare", left: {op: "empty"}, right: {op: "empty"}, relation: "equal"}
				}
				then: {op: "bounds", lower: {op: "local", endpoint: "lower"}, upper: {op: "local", endpoint: "upper"}}
				else: {op: "fail", reason: "bounds"}
			}
		}
	}
}}
