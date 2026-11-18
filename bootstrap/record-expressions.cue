package bootstrap

#Program & {entry: {kind: "recordExpression", fields: [
	{name: "é", value: {kind: "string", value: "🙂"}},
	{name: "max", value: {kind: "add", left: {kind: "integer", value: 9223372036854775806}, right: {kind: "integer", value: 1}}},
	{name: "min", value: {kind: "listIndex", left: {kind: "list", value: [-9223372036854775808]}, right: {kind: "integer", value: 0}}},
	{name: "count", value: {kind: "recordLength", operand: {kind: "recordExpression", fields: [{name: "x", value: {kind: "null"}}]}}},
]}}
