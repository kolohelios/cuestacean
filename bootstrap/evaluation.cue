package bootstrap

#EvaluationRules & {
	rules: {
		intType: result: {op: "int"}
		lowerBound: result: {op: "lowerBound", value: {op: "field", field: "value"}}
		upperBound: result: {op: "upperBound", value: {op: "field", field: "value"}}
		integer: result: {op: "field", field: "value"}
		negate: result: {op: "negate", value: {op: "evaluate", field: "operand"}}
		add: result: {
			op: "sum"
			left: {op: "evaluate", field: "left"}
			right: {op: "evaluate", field: "right"}
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
		divide: result: {
			op: "divide"
			left: {op: "evaluate", field: "left"}
			right: {op: "evaluate", field: "right"}
		}
		modulo: result: {
			op: "modulo"
			left: {op: "evaluate", field: "left"}
			right: {op: "evaluate", field: "right"}
		}
		quotient: result: {
			op: "quotient"
			left: {op: "evaluate", field: "left"}
			right: {op: "evaluate", field: "right"}
		}
		remainder: result: {
			op: "remainder"
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
