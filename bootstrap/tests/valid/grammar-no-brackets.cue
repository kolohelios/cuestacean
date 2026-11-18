package bootstrap

// Complete counterpart to grammar-slice-without-index: no brackets is valid.
#Grammar & close({
	formatVersion: 1
	integer: {kind: "integer", valueField: "value"}
	atoms: [], calls: [], bounds: []
	group: {open: "(", close: ")"}
	operators: [{token: "+", kind: "add", leftField: "left", rightField: "right", precedence: 10, associativity: "left"}]
})
