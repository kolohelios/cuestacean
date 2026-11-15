package bootstrap

// Leave digit-adjacent minus to signed-decimal conversion, including i64::MIN.
#ScanProgram & {body: [{
	op: "if"
	condition: {op: "byteIn", bytes: [45]}
	then: [{
		op: "if"
		condition: {op: "byteIn", ahead: 1, bytes: [48, 49, 50, 51, 52, 53, 54, 55, 56, 57]}
		then: []
		else: [{op: "advance"}, {op: "setFlag", value: true}]
	}]
}]}
