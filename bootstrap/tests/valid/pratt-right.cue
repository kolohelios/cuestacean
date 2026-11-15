package bootstrap

// Alternate algorithm: omit the left-associative increment, making all operators right-associative.
#PrattProgram & {
	body: [{op: "primary"}, {op: "afterOperand"}, {op: "lookupOperator"}, {
		op: "while"
		condition: {op: "and", left: {op: "hasOperator"}, right: {
			op: "atLeast", left: {op: "precedence"}, right: {op: "minimum"}
		}}
		body: [{op: "setBindingPower", value: {op: "precedence"}},
			{op: "advanceOperator"}, {op: "parseRight"}, {op: "buildBinary"},
			{op: "afterOperand"}, {op: "lookupOperator"}]
	}]
}
