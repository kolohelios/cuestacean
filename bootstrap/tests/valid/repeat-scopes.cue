package bootstrap

#ScanProgram & {body: [
	{op: "repeat", count: 2, body: [
		{op: "repeat", count: 64, body: [{op: "advanceChar"}, {op: "break"}]},
		{op: "advanceChar"},
	]},
	{op: "setFlag", value: true},
]}
