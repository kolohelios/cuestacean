package bootstrap

#LoweringPolicy & {cases: root: {
	op: "bind", name: "saved", value: {op: "string", field: "text\"\\\nλ"}, body: {
		op: "bind", name: "saved", value: {op: "get", name: "saved"}, body: {
			op: "ifEqual", left: {op: "get", name: "saved"}, right: {op: "text", value: "\"same\""}
			then: {op: "concat", parts: [{op: "get", name: "saved"}, {op: "text", value: "!"}]}
			else: {op: "ifTrue", field: "flag", then: {op: "get", name: "saved"}, else: {op: "text", value: "no"}}
		}
	}
}}
