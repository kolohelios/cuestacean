package bootstrap

// Alternate policy: token boundaries only, allowing missing digits/leading zeroes.
#ScanProgram & {
	body: [{
		op: "if"
		condition: {op: "byteIn", bytes: [45]}
		then: [{op: "setFlag", value: true}, {op: "advance"}]
	}, {
		op: "while"
		condition: {op: "byteIn", bytes: [48, 49, 50, 51, 52, 53, 54, 55, 56, 57]}
		body: [{op: "advance"}]
	}]
}
