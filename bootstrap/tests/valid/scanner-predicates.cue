package bootstrap

#ScanProgram & {body: [
	{op: "if", condition: {
		op: "and", left: {op: "hasByte"}
		right: {op: "or", left: {op: "byteIn", bytes: [97]}, right: {op: "charIn", characters: ["λ"]}}
	}, then: [{op: "setFlag", value: true}, {op: "advanceChar"}]},
	{op: "if", condition: {
		op: "or", left: {op: "flagIs", value: true}
		right: {op: "and", left: {op: "byteIn", bytes: [120]}, right: {op: "byteIn", ahead: 1, bytes: [121]}}
	}, then: [], else: [{op: "setFlag", value: true}]},
]}
