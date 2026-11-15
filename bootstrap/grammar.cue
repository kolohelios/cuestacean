package bootstrap

#Grammar & {
	integer: {kind: "integer", valueField: "value"}
	atoms: [{token: "int", kind: "intType"}]
	bounds: [
		{token: ">=", kind: "lowerBound", valueField: "value"},
		{token: "<=", kind: "upperBound", valueField: "value"},
	]
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
	}]
}
