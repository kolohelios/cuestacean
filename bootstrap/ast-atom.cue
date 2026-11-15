package bootstrap

#BuildProgram & {
	parameters: [{name: "kind", type: "string"}]
	result: {
		op: "object"
		fields: [{key: {op: "string", value: "kind"}, value: {op: "argument", name: "kind"}}]
	}
}
