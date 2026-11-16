package bootstrap

#ScanProgram & {body: [{op: "if", condition: {op: "byteIn", bytes: [33]}, then: [
	{op: "if", condition: {op: "byteIn", ahead: 1, bytes: [61]}, then: [], else: [{op: "advance"}, {op: "setFlag", value: true}]},
]}]}
