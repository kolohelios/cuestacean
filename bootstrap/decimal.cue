package bootstrap

// Token boundaries only: optional ASCII minus, then a run of ASCII digits.
// The flag records that a minus was consumed; validation/conversion stay native.
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
