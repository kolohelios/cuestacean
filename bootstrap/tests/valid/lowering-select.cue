package bootstrap

#LoweringPolicy & {
	cases: {
		"root\"\\\nλ": {op: "select", field: "kind\"\\\nλ", cases: {
			"go\"\\\nλ": {op: "lower", field: "child\"\\\nλ"}
			fallback: {op: "text", value: "fallback"}
		}}
		leaf: {op: "select", field: "mode", cases: {
			yes: {op: "text", value: "chosen"}
			no: {op: "text", value: "other"}
		}}
	}
}
