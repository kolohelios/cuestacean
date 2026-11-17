package bootstrap

#Grammar & {
	integer: {kind: "integer", valueField: "value"}
	calls: [{token: "len", kind: "stringLength", operandField: "kind"}]
	operators: [{token: "&", kind: "unify", leftField: "left", rightField: "right", precedence: 5, associativity: "left"}]
}
