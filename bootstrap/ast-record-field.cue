package bootstrap

#BuildProgram & {
	parameters: [{name: "name", type: "string"}, {name: "value", type: "json"}]
	result: {op: "object", fields: [
		{key: {op: "string", value: "name"}, value: {op: "argument", name: "name"}},
		{key: {op: "string", value: "value"}, value: {op: "argument", name: "value"}},
	]}
}
