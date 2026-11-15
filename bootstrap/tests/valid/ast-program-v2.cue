package bootstrap

// Valid builder, intentionally incompatible program version: policy is data.
#BuildProgram & {
	parameters: [{name: "entry", type: "json"}]
	result: {
		op: "object"
		fields: [
			{key: {op: "string", value: "formatVersion"}, value: {op: "integer", value: 2}},
			{key: {op: "string", value: "entry"}, value: {op: "argument", name: "entry"}},
		]
	}
}
