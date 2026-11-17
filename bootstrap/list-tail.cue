package bootstrap

// Required separator; false means comma, true means completed closing bracket.
#ScanProgram & {body: [{op: "if", condition: {op: "byteIn", bytes: [44]}, then: [
	{op: "advance"},
], else: [{op: "if", condition: {op: "byteIn", bytes: [93]}, then: [
	{op: "advance"}, {op: "setFlag", value: true},
], else: [{op: "reject", reason: "expected comma or closing bracket"}]}]}]}
