package bootstrap

#LoweringPolicy & {
	cases: root: {op: "text", value: "unused"}
	module: [{op: "firstRule"}, {op: "whileRules", body: [{op: "kind"}, {op: "nextRule"}]}]
}
