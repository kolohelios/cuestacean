package bootstrap

let Record = {kind: "recordExpression", fields: [{name: "items", value: {kind: "listExpression", elements: [
	{kind: "recordExpression", fields: [{name: "v", value: {kind: "integer", value: 9223372036854775807}}]},
]}}]}
#Program & {entry: {kind: "index", left: {kind: "index", left: {kind: "index", left: Record, right: {kind: "string", value: "items"}}, right: {kind: "integer", value: 0}}, right: {kind: "string", value: "v"}}}
