package bootstrap

#BuildProgram & {
	parameters: [{name: "entry", type: "json"}]
	result: {
		op: "object"
		fields: [
			{key: {op: "string", value: "formatVersion"}, value: {op: "integer", value: 1}},
			{key: {op: "string", value: "entry"}, value: {op: "argument", name: "entry"}},
		]
	}
}
