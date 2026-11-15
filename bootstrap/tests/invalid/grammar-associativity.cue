package bootstrap

#Grammar & {
	integer: {kind: "integer", valueField: "value"}
	operators: [{token: "+", kind: "add", leftField: "left", rightField: "right", precedence: 10, associativity: "center"}]
}
