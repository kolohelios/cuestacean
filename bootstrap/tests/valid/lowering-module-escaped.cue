package bootstrap

#LoweringPolicy & {
	cases: x: {op: "path", value: "X"}
	module: [{op: "text", value: "// quote\" slash\\\n// 雪\r\n"},
		{op: "firstRule"}, {op: "whileRules", body: [{op: "kind"}, {op: "expression"}, {op: "nextRule"}]}]
}
