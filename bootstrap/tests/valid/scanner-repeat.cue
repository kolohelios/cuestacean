package bootstrap

#ScanProgram & {body: [
	{op: "repeat", count: 3, body: [
		{op: "if", condition: {op: "hasByte"}, then: [{op: "advanceChar"}], else: [{op: "break"}]},
	]},
	{op: "setFlag", value: true},
]}
