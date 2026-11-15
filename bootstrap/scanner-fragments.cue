package bootstrap

// Shared executable statements, not a native whitespace primitive.
#WhitespaceBody: [{
	op: "while"
	condition: {op: "byteIn", bytes: [32, 9, 10, 13]}
	body: [{
		op: "if"
		condition: {op: "byteIn", bytes: [10, 13]}
		then: [{op: "setFlag", value: true}]
		else: []
	}, {op: "advance"}]
}]
