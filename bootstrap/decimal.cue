package bootstrap

// Decimal spelling and diagnostics live here; checked i64 conversion stays native.
let digits = [48, 49, 50, 51, 52, 53, 54, 55, 56, 57]
let digitRun = {
	op: "while"
	condition: {op: "byteIn", bytes: digits}
	body: [{op: "advance"}]
}

#ScanProgram & {
	body: [{
		op: "if"
		condition: {op: "byteIn", bytes: [45]}
		then: [{op: "setFlag", value: true}, {op: "advance"}]
	}, {
		op: "if"
		condition: {op: "byteIn", bytes: [48]}
		then: [{op: "advance"}, {
			op: "if"
			condition: {op: "byteIn", bytes: digits}
			// Consume the full run before rejecting, preserving diagnostic offsets.
			then: [digitRun, {op: "reject", reason: "leading zeroes are unsupported"}]
		}]
		else: [{
			op: "if"
			condition: {op: "byteIn", bytes: [49, 50, 51, 52, 53, 54, 55, 56, 57]}
			then: [digitRun]
			else: [{op: "reject", reason: "expected signed decimal integer"}]
		}]
	}]
}
