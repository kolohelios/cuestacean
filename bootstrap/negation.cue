package bootstrap

#Program & {
	entry: {
		kind: "negate"
		operand: {
			kind: "add"
			left: {kind: "integer", value: 1}
			right: {kind: "integer", value: 2}
		}
	}
}
