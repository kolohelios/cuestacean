package bootstrap

// A well-typed but different rule: add evaluates its left child twice.
#EvaluationRules & {
	rules: {
		intType: result: {op: "int"}
		lowerBound: result: {op: "lowerBound", value: {op: "field", field: "value"}}
		upperBound: result: {op: "upperBound", value: {op: "field", field: "value"}}
		integer: result: {op: "field", field: "value"}
		add: result: {
			op: "sum"
			left: {op: "evaluate", field: "left"}
			right: {op: "evaluate", field: "left"}
		}
		subtract: result: {
			op: "difference"
			left: {op: "evaluate", field: "left"}
			right: {op: "evaluate", field: "right"}
		}
		unify: result: {
			op: "unify"
			left: {op: "evaluate", field: "left"}
			right: {op: "evaluate", field: "right"}
		}
		multiply: result: {
			op: "product"
			left: {op: "evaluate", field: "left"}
			right: {op: "evaluate", field: "right"}
		}
	}
}
