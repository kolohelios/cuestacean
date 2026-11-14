package bootstrap

#Program & {
	entry: {
		kind: "add"
		left: {kind: "integer", value: 1}
		right: {
			kind: "add"
			left: {kind: "integer", value: 2}
			right: {kind: "integer", value: 3.5}
		}
	}
}
