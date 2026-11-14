package bootstrap

// Preserve the tree for 20 + (10 + 12), rather than computing its result.
#Program & {
	entry: {
		kind: "add"
		left: {kind: "integer", value: 20}
		right: {
			kind: "add"
			left: {kind: "integer", value: 10}
			right: {kind: "integer", value: 12}
		}
	}
}
