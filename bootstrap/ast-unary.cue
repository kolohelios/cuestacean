package bootstrap

#BuildProgram & {
	parameters: [
		{name: "kind", type: "string"},
		{name: "operandField", type: "string"},
		{name: "operand", type: "json"},
	]
	result: {
		op: "object"
		fields: [
			{key: {op: "string", value: "kind"}, value: {op: "argument", name: "kind"}},
			{key: {op: "argument", name: "operandField"}, value: {op: "argument", name: "operand"}},
		]
	}
}
