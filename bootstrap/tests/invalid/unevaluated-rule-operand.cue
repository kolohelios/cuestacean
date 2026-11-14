package bootstrap

#EvaluationRules & {
	rules: add: result: {
		op: "sum"
		left: {op: "field", field: "left"}
		right: {op: "evaluate", field: "right"}
	}
}
