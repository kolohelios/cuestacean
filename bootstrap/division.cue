package bootstrap

// 39 + (9 quo 4) + (9 rem 4) + (-9 div 4) + (-9 mod 4) = 42.
// Exercise every new AST shape through the recursive CUE expression schema.
#Program & {entry: {
	kind: "add"
	left: {kind: "integer", value: 39}
	right: {
		kind: "add"
		left: {kind: "quotient", left: {kind: "integer", value: 9}, right: {kind: "integer", value: 4}}
		right: {
			kind: "add"
			left: {kind: "remainder", left: {kind: "integer", value: 9}, right: {kind: "integer", value: 4}}
			right: {
				kind: "add"
				left: {kind: "divide", left: {kind: "integer", value: -9}, right: {kind: "integer", value: 4}}
				right: {kind: "modulo", left: {kind: "integer", value: -9}, right: {kind: "integer", value: 4}}
			}
		}
	}
}}
