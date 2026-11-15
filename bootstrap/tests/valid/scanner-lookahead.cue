package bootstrap

// Maximum permitted lookahead, including an else-only branch in emitted Rust.
#ScanProgram & {
	body: [{
		op: "if"
		condition: {op: "byteIn", bytes: [61], ahead: 64}
		then: []
		else: [{op: "reject", reason: "expected equals at lookahead"}]
	}, {op: "setFlag", value: true}]
}
