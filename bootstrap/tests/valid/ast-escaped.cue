package bootstrap

// Parameter names are metadata strings, never Rust source identifiers.
#BuildProgram & {
	parameters: [{name: "param\"\\\nλ", type: "string"}]
	result: {
		op: "object"
		fields: [
			{key: {op: "string", value: "payload\"\\\nλ"}, value: {op: "argument", name: "param\"\\\nλ"}},
			{key: {op: "string", value: "second"}, value: {op: "argument", name: "param\"\\\nλ"}},
		]
	}
}
