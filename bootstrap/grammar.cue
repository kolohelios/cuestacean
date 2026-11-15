package bootstrap

#Grammar & {
	integer: {kind: "integer", valueField: "value"}
	atoms: [{token: "int", kind: "intType"}]
	bounds: [
		{token: ">=", kind: "lowerBound", valueField: "value"},
		{token: "<=", kind: "upperBound", valueField: "value"},
	]
	negation: {token: "-", kind: "negate", operandField: "operand"}
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
	}]
}
