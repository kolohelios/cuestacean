package bootstrap

// Exercise signed extrema in both typed literal roles, including an unused arm.
#UnifyProgram & {
	formatVersion: 1
	result: {
		op: "if", condition: {
			op: "compare", relation: "less"
			left: {op: "integer", value: -9223372036854775808}
			right: {op: "integer", value: 9223372036854775807}
		}
		then: {op: "integer", value: -9223372036854775808}
		else: {op: "integer", value: 9223372036854775807}
	}
}
