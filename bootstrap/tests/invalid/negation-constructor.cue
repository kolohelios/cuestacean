package bootstrap

#Grammar & {
	integer: {kind: "integer", valueField: "value"}
	negation: {token: "-", kind: "negate", operandField: "kind"}
	operators: [{token: "+", kind: "add", leftField: "left", rightField: "right", precedence: 10, associativity: "left"}]
}
