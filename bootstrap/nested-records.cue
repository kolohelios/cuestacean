package bootstrap

#Program & {entry: {kind: "recordExpression", fields: [
	{name: "limits", value: {kind: "recordExpression", fields: [
		{name: "min", value: {kind: "integer", value: -9223372036854775808}},
		{name: "max", value: {kind: "integer", value: 9223372036854775807}},
	]}},
	{name: "selected", value: {kind: "recordIndex", left: {kind: "recordExpression", fields: [
		{name: "child", value: {kind: "recordExpression", fields: [{name: "x", value: {kind: "add", left: {kind: "integer", value: 2}, right: {kind: "integer", value: 3}}}]}},
	]}, right: {kind: "string", value: "child"}}},
	{name: "text", value: {kind: "string", value: "owned 🙂"}},
	{name: "empty", value: {kind: "recordExpression", fields: []}},
]}}
