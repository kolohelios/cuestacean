package bootstrap

#Grammar & {
	integer: {kind: "integer", valueField: "value"}
	atoms: [{token: "int", kind: "intType"}]
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
	}]
}
