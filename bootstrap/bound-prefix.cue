package bootstrap

// Check the full token before consuming anything; partial matches are untouched.
#ScanProgram & {
	body: [{
		op: "if"
		condition: {op: "byteIn", bytes: [60, 62]}
		then: [{
			op: "if"
			condition: {op: "byteIn", bytes: [61], ahead: 1}
			then: [{op: "advance"}, {op: "advance"}, {op: "setFlag", value: true}]
		}]
	}]
}
