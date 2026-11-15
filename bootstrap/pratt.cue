package bootstrap

#PrattProgram & {
	body: [{op: "primary"}, {op: "afterOperand"}, {op: "lookupOperator"}, {
		op: "while"
		condition: {
			op: "and"
			left: {op: "hasOperator"}
			right: {op: "atLeast", left: {op: "precedence"}, right: {op: "minimum"}}
		}
		body: [{
			op: "setBindingPower"
			value: {op: "precedence"}
		}, {
			op: "if"
			condition: {op: "leftAssociative"}
			then: [{op: "setBindingPower", value: {
				op: "add", left: {op: "precedence"}, right: {op: "integer", value: 1}
			}}]
		}, {op: "advanceOperator"}, {
			op: "if"
			condition: {op: "atLeast", left: {op: "tokenLength"}, right: {op: "integer", value: 3}}
			then: [{op: "afterWordOperator"}]
		}, {op: "parseRight"}, {op: "buildBinary"},
			{op: "afterOperand"}, {op: "lookupOperator"}]
	}]
}
