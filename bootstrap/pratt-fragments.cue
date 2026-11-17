package bootstrap

#PrattControl: #PrattProgram & {
	_postfix: *true | false
	_associativity: *true | false
	_indexMinimum: *0 | 100
	body: [{op: "primary"}, {op: "afterOperand"},
		if _postfix {op: "scanIndex"}, if _postfix {
			op: "while", condition: {op: "hasIndex"}
			body: [{op: "advanceIndex"}, {op: "parseIndex", minimum: {op: "integer", value: _indexMinimum}},
				{op: "closeIndex"}, {op: "afterOperand"}, {op: "scanIndex"}]
		}, {op: "lookupOperator"}, {
		op: "while"
		condition: {
			op: "and"
			left: {op: "hasOperator"}
			right: {op: "atLeast", left: {op: "precedence"}, right: {op: "minimum"}}
		}
		body: [{
			op: "setBindingPower"
			value: {op: "precedence"}
		}, if _associativity {
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
