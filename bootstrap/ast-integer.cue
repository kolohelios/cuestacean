package bootstrap

// Shared by integer literals and bound leaves; names come from grammar data.
#BuildProgram & {
	parameters: [
		{name: "kind", type: "string"},
		{name: "valueField", type: "string"},
		{name: "value", type: "integer"},
	]
	result: {
		op: "object"
		fields: [
			{key: {op: "string", value: "kind"}, value: {op: "argument", name: "kind"}},
			{key: {op: "argument", name: "valueField"}, value: {op: "argument", name: "value"}},
		]
	}
}
