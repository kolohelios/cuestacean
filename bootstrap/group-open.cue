package bootstrap

// Recognition only: an unmatched start leaves the cursor and flag unchanged.
#ScanProgram & {
	body: [{
		op: "if"
		condition: {op: "byteIn", bytes: [40]}
		then: [{op: "setFlag", value: true}, {op: "advance"}]
	}]
}
