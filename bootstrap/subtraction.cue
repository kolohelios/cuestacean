package bootstrap

// Preserve the tree for 50 - 8, rather than computing its result.
#Program & {
	entry: {
		kind: "subtract"
		left: {kind: "integer", value: 50}
		right: {kind: "integer", value: 8}
	}
}
