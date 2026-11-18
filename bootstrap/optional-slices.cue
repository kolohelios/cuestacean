package bootstrap

#Program & {
	let Values = {kind: "list", value: [-9223372036854775808, 0, 9223372036854775807, "é🙂"]}
	entry: {kind: "listExpression", elements: [
		{kind: "slice", operand: Values},
		{kind: "slice", operand: Values, lower: {kind: "integer", value: 1}},
		{kind: "slice", operand: Values, upper: {kind: "integer", value: 2}},
		{kind: "slice", operand: Values, lower: {kind: "integer", value: 1}, upper: {kind: "integer", value: 3}},
		{kind: "slice", operand: {kind: "list", value: []}},
		{kind: "slice", operand: {kind: "listExpression", elements: [
			{kind: "integer", value: 99},
			{kind: "recordExpression", fields: [{name: "x", value: {kind: "list", value: [-9223372036854775808, "尾"]}}]},
		]}, lower: {kind: "integer", value: 1}},
	]}
}
