package bootstrap

#EvaluationRules & {
	rules: add: result: {
		op: "sum"
		left: {op: "evaluate", field: "left"}
	}
}
