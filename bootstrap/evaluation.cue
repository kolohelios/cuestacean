package bootstrap

#EvaluationRules & {
	rules: {
		integer: result: {op: "field", field: "value"}
		add: result: {
			op: "sum"
			left: {op: "evaluate", field: "left"}
			right: {op: "evaluate", field: "right"}
		}
	}
}
