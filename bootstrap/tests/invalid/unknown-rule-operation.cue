package bootstrap

#EvaluationRules & {
	rules: add: result: {
		op: "multiply"
		left: {op: "evaluate", field: "left"}
		right: {op: "evaluate", field: "right"}
	}
}
