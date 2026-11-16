package bootstrap

#LoweringPolicy & {
	cases: {
		root: {op: "join", field: "items\"\\\nλ", separator: "|\"\\\nλ|"}
		item: {op: "ifZero", field: "count", then: {op: "signed", field: "n"}, else: {op: "text", value: "more"}}
	}
}
