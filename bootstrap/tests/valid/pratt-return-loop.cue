package bootstrap

#PrattProgram & {
	body: [{op: "while", condition: {op: "boolean", value: true}, body: [
		{op: "primary"},
		{op: "if", condition: {op: "boolean", value: true}, then: [{op: "return"}]},
	]}]
}
