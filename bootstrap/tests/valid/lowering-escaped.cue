package bootstrap

#LoweringPolicy & {
	cases: {
		"text\"\\\nλ": {op: "string", field: "payload\"\\\nλ"}
		empty: {op: "call", path: "module::empty", args: []}
		outer: {op: "call", path: "module::record", args: [
			{op: "lower", field: "child\"\\\nλ"},
			{op: "boolean", field: "flag\"\\\nλ"},
		]}
	}
}
