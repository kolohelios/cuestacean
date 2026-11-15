package bootstrap

#Grammar & {
	integer: {kind: "integer", valueField: "value"}
	atoms: [{token: "1int", kind: "intType"}]
	operators: [{token: "+", kind: "add", leftField: "left", rightField: "right", precedence: 10, associativity: "left"}]
}
