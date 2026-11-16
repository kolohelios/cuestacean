package bootstrap

#LoweringPolicy & {
	cases: {
		string: {op: "string", field: "value"}
		root: {
			op: "ifEmpty", field: "items\"\\\nλ"
			then: {op: "concat", parts: []}
			else: {op: "list", field: "items\"\\\nλ"}
		}
		"item\"\\\nλ": {op: "concat", parts: [
			{op: "text", value: "raw\"\\\nλ"}, {op: "unsigned", field: "number\"\\\nλ"},
		]}
	}
}
