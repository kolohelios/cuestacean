package bootstrap

#EvaluationRules & {
	rules: multiply: result: {
		op: "product"
		left: {op: "evaluate", field: "left"}
		right: {op: "evaluate", field: "right"}
	}
	rules: add: result: {
		op: "multiply"
		left: {op: "evaluate", field: "left"}
		right: {op: "evaluate", field: "right"}
	}
}
