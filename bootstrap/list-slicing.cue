package bootstrap

#Program & {
	let Values = {kind: "list", value: [-9223372036854775808, 0, 9223372036854775807]}
	entry: {
		kind: "listExpression"
		elements: [
			{kind: "listSlice", operand: Values, lower: {kind: "integer", value: 0}, upper: {kind: "integer", value: 3}},
			{kind: "listSlice", operand: Values, lower: {kind: "integer", value: 1}, upper: {kind: "integer", value: 2}},
			{kind: "listSlice", operand: Values, lower: {kind: "integer", value: 3}, upper: {kind: "integer", value: 3}},
			{kind: "listSlice", operand: {
				kind: "listExpression", elements: [
					{kind: "integer", value: 99},
					{kind: "recordExpression", fields: [{name: "é🙂", value: {kind: "list", value: [true, null, "尾", 9223372036854775807]}}]},
					{kind: "list", value: ["é", -9223372036854775808]},
				]
			}, lower: {kind: "integer", value: 1}, upper: {kind: "integer", value: 3}},
		]
	}
}
