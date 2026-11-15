package bootstrap

#BuildProgram & {
	parameters: [
		{name: "kind", type: "string"},
		{name: "leftField", type: "string"},
		{name: "rightField", type: "string"},
		{name: "left", type: "json"},
		{name: "right", type: "json"},
	]
	result: {
		op: "object"
		fields: [
			{key: {op: "string", value: "kind"}, value: {op: "argument", name: "kind"}},
			{key: {op: "argument", name: "leftField"}, value: {op: "argument", name: "left"}},
			{key: {op: "argument", name: "rightField"}, value: {op: "argument", name: "right"}},
		]
	}
}
