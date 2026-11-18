package bootstrap

#BuildProgram & {
	parameters: [
		{name: "kind", type: "string"},
		{name: "operandField", type: "string"},
		{name: "lowerField", type: "string"},
		{name: "upperField", type: "string"},
		{name: "operand", type: "json"},
		{name: "lower", type: "json"},
		{name: "upper", type: "json"},
	]
	result: {op: "object", fields: [
		{key: {op: "string", value: "kind"}, value: {op: "argument", name: "kind"}},
		{key: {op: "argument", name: "operandField"}, value: {op: "argument", name: "operand"}},
		{key: {op: "argument", name: "lowerField"}, value: {op: "argument", name: "lower"}},
		{key: {op: "argument", name: "upperField"}, value: {op: "argument", name: "upper"}},
	]}
}
