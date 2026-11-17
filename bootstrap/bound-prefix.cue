package bootstrap

// Longest match: consume a comparison glyph and its optional adjacent equals.
#ScanProgram & {
	body: [{op: "if", condition: {op: "byteIn", bytes: [60, 62]}, then: [
		{op: "advance"}, {op: "if", condition: {op: "byteIn", bytes: [61]}, then: [{op: "advance"}]},
		{op: "setFlag", value: true},
	]}]
}
