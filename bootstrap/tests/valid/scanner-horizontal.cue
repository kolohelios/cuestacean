package bootstrap

// Different algorithm: skip spaces/tabs; report whether the last was a space.
#ScanProgram & {
	body: [{
		op: "while"
		condition: {op: "byteIn", bytes: [32, 9]}
		body: [{
			op: "if"
			condition: {op: "byteIn", bytes: [32]}
			then: [{op: "setFlag", value: true}]
			else: [{op: "setFlag", value: false}]
		}, {op: "advance"}]
	}]
}
