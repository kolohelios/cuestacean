package bootstrap

#Program & {entry: {
	kind: "listExpression"
	elements: [
		{kind: "conjunction", operands: [{kind: "intType"}, {kind: "lowerBound", value: 0}, {kind: "upperBound", value: 100}, {kind: "integer", value: 42}]},
		{kind: "conjunction", operands: [{kind: "top"}, {kind: "stringType"}, {kind: "string", value: "é🙂"}]},
		{kind: "conjunction", operands: [{kind: "boolType"}, {kind: "true"}]},
		{kind: "unify", left: {kind: "conjunction", operands: []}, right: {kind: "integer", value: -9223372036854775808}},
		{kind: "conjunction", operands: [{kind: "list", value: [null, true, 9223372036854775807]}, {kind: "list", value: [null, true, 9223372036854775807]}]},
		{kind: "conjunction", operands: [{kind: "recordExpression", fields: [{name: "x", value: {kind: "list", value: [9223372036854775807, "尾"]}}]}, {kind: "recordExpression", fields: [{name: "x", value: {kind: "list", value: [9223372036854775807, "尾"]}}]}]},
	]
}}
