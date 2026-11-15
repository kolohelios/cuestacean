package bootstrap

#EvaluationRules & {
	rules: multiply: result: {
		op: "product"
		left: {op: "evaluate", field: "left"}
		right: {op: "evaluate", field: "right"}
	}
	rules: add: result: {
		op: "sum"
		left: {op: "field", field: "left"}
		right: {op: "evaluate", field: "right"}
	}
}
