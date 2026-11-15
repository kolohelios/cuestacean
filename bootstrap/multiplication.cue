package bootstrap

// The tree for 1 + 2 * 3 preserves multiplication's higher precedence.
#Program & {
	entry: {
		kind: "add"
		left: {kind: "integer", value: 1}
		right: {
			kind: "multiply"
			left: {kind: "integer", value: 2}
			right: {kind: "integer", value: 3}
		}
	}
}
