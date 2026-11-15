package bootstrap

// The loop, newline branch and state updates are executable DSL statements.
#ScanProgram & {
	body: [{
		op: "while"
		condition: {op: "byteIn", bytes: [32, 9, 10, 13]}
		body: [{
			op: "if"
			condition: {op: "byteIn", bytes: [10, 13]}
			then: [{op: "setFlag", value: true}]
		}, {op: "advance"}]
	}]
}
