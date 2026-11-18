package bootstrap

#Grammar & {
	integer: {kind: "integer", valueField: "value"}
	text: {kind: "string", valueField: "value"}
	calls: [{token: "len", kind: "length", operandField: "operand"}]
	list: {kind: "listExpression", elementsField: "elements"}
	conjunction: {token: "and", kind: "conjunction", operandsField: "operands"}
	record: {token: "close", kind: "recordExpression", fieldsField: "fields"}
	selector: {kind: "select", operandField: "operand", labelField: "label"}
	index: {kind: "index", leftField: "left", rightField: "right"}
	slice: {kind: "listSlice", operandField: "operand", lowerField: "lower", upperField: "upper"}
	atoms: [{token: "int", kind: "intType"}, {token: "bool", kind: "boolType"},
		{token: "true", kind: "true"}, {token: "false", kind: "false"}, {token: "string", kind: "stringType"}, {token: "null", kind: "null"}, {token: "_", kind: "top"}]
	bounds: [
		{token: ">=", kind: "lowerBound", valueField: "value", expression: {kind: "lowerBoundExpression", operandField: "operand"}},
		{token: "<=", kind: "upperBound", valueField: "value", expression: {kind: "upperBoundExpression", operandField: "operand"}},
		{token: ">", kind: "lowerExclusiveBound", valueField: "value", expression: {kind: "lowerExclusiveBoundExpression", operandField: "operand"}},
		{token: "<", kind: "upperExclusiveBound", valueField: "value", expression: {kind: "upperExclusiveBoundExpression", operandField: "operand"}},
	]
	negation: {token: "-", kind: "negate", operandField: "operand"}
	logicalNot: {token: "!", kind: "logicalNot", operandField: "operand"}
	group: {open: "(", close: ")"}
	operators: [{
		token:         "+"
		kind:          "add"
		leftField:     "left"
		rightField:    "right"
		precedence:    10
		associativity: "left"
	}, {
		token:         "*"
		kind:          "multiply"
		leftField:     "left"
		rightField:    "right"
		precedence:    20
		associativity: "left"
	}, {
		token:         "&"
		kind:          "unify"
		leftField:     "left"
		rightField:    "right"
		precedence:    5
		associativity: "left"
	}, {
		token:         "-"
		kind:          "subtract"
		leftField:     "left"
		rightField:    "right"
		precedence:    10
		associativity: "left"
	}, {
		token:         "div"
		kind:          "divide"
		leftField:     "left"
		rightField:    "right"
		precedence:    20
		associativity: "left"
	}, {
		token:         "mod"
		kind:          "modulo"
		leftField:     "left"
		rightField:    "right"
		precedence:    20
		associativity: "left"
	}, {
		token:         "quo"
		kind:          "quotient"
		leftField:     "left"
		rightField:    "right"
		precedence:    20
		associativity: "left"
	}, {
		token:         "rem"
		kind:          "remainder"
		leftField:     "left"
		rightField:    "right"
		precedence:    20
		associativity: "left"
	}, {
		token:         "=="
		kind:          "equal"
		leftField:     "left"
		rightField:    "right"
		precedence:    8
		associativity: "left"
	}, {
		token:         "!="
		kind:          "notEqual"
		leftField:     "left"
		rightField:    "right"
		precedence:    8
		associativity: "left"
	}, {
		token: "<", kind: "less", leftField: "left", rightField: "right", precedence: 8, associativity: "left"
	}, {
		token: "<=", kind: "lessEqual", leftField: "left", rightField: "right", precedence: 8, associativity: "left"
	}, {
		token: ">", kind: "greater", leftField: "left", rightField: "right", precedence: 8, associativity: "left"
	}, {
		token: ">=", kind: "greaterEqual", leftField: "left", rightField: "right", precedence: 8, associativity: "left"
	}, {
		token: "&&", kind: "logicalAnd", leftField: "left", rightField: "right", precedence: 7, associativity: "left"
	}, {
		token: "||", kind: "logicalOr", leftField: "left", rightField: "right", precedence: 6, associativity: "left"
	}]
}
