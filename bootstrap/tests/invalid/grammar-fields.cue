package bootstrap

#Grammar & {
	integer: {kind: "integer", valueField: "value"}
	operators: [{token: "+", kind: "add", leftField: "left", rightField: "left", precedence: 10, associativity: "left"}]
}
