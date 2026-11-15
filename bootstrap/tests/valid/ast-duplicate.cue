package bootstrap

#BuildProgram & {
	parameters: [{name: "value", type: "json"}]
	result: {
		op: "object"
		fields: [
			{key: {op: "string", value: "first"}, value: {op: "argument", name: "value"}},
			{key: {op: "string", value: "second"}, value: {op: "argument", name: "value"}},
		]
	}
}
