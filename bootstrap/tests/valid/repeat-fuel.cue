package bootstrap

// Exactly 256 steps: 127 + 127 + 1 + 1. One remaining byte supplies 256 fuel.
#ScanProgram & {body: [
	{op: "repeat", count: 63, body: [{op: "setFlag", value: true}]},
	{op: "repeat", count: 63, body: [{op: "setFlag", value: true}]},
	{op: "repeat", count: 0, body: [{op: "advance"}]},
	{op: "setFlag", value: false},
]}
