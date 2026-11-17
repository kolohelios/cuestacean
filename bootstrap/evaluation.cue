package bootstrap

#EvaluationRules & {
	rules: {
		"null": result: {op: "null"}
		top: result: {op: "top"}
		intType: result: {op: "int"}
		boolType: result: {op: "bool"}
		"true": result: {op: "boolean", value: true}
		"false": result: {op: "boolean", value: false}
		lowerBound: result: {op: "lowerBound", value: {op: "field", field: "value"}}
		upperBound: result: {op: "upperBound", value: {op: "field", field: "value"}}
		lowerBoundExpression: result: {op: "lowerBound", value: {op: "evaluate", field: "operand"}}
		upperBoundExpression: result: {op: "upperBound", value: {op: "evaluate", field: "operand"}}
		integer: result: {op: "field", field: "value"}
		logicalNot: result: {op: "logicalNot", value: {op: "evaluate", field: "operand"}}
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
		logicalAnd: result: {op: "logicalAnd", left: {op: "evaluate", field: "left"}, right: {op: "evaluate", field: "right"}}
		logicalOr: result: {op: "logicalOr", left: {op: "evaluate", field: "left"}, right: {op: "evaluate", field: "right"}}
		less: result: {op: "less", left: {op: "evaluate", field: "left"}, right: {op: "evaluate", field: "right"}}
		lessEqual: result: {op: "lessEqual", left: {op: "evaluate", field: "left"}, right: {op: "evaluate", field: "right"}}
		greater: result: {op: "greater", left: {op: "evaluate", field: "left"}, right: {op: "evaluate", field: "right"}}
		greaterEqual: result: {op: "greaterEqual", left: {op: "evaluate", field: "left"}, right: {op: "evaluate", field: "right"}}
		equal: result: {op: "equal", left: {op: "evaluate", field: "left"}, right: {op: "evaluate", field: "right"}}
		notEqual: result: {op: "notEqual", left: {op: "evaluate", field: "left"}, right: {op: "evaluate", field: "right"}}
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
