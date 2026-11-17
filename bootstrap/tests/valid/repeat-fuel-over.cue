package bootstrap

// Same boundary program with one additional action: 257 steps.
#ScanProgram & {body: [
	{op: "repeat", count: 63, body: [{op: "setFlag", value: true}]},
	{op: "repeat", count: 63, body: [{op: "setFlag", value: true}]},
	{op: "repeat", count: 0, body: [{op: "advance"}]},
	{op: "setFlag", value: false},
	{op: "setFlag", value: true},
]}
