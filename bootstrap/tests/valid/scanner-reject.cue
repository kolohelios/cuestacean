package bootstrap

// Exercise escaped diagnostic text and stop before the following unsafe advance.
#ScanProgram & {
	body: [{op: "advance"}, {
		op:     "reject"
		reason: "bad \"token\"\\\nλ {value}"
	}, {op: "advance"}]
}
