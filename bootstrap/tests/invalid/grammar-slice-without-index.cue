package bootstrap

// Otherwise-complete closed grammar: the dependency, not missing data, rejects it.
#Grammar & close({
	formatVersion: 1
	integer: {kind: "integer", valueField: "value"}
	atoms: [], calls: [], bounds: []
	group: {open: "(", close: ")"}
	operators: [{token: "+", kind: "add", leftField: "left", rightField: "right", precedence: 10, associativity: "left"}]
	slice: {kind: "listSlice", operandField: "operand", lowerField: "lower", upperField: "upper"}
})
