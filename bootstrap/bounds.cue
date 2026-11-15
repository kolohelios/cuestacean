package bootstrap

// int & >=0 & <=10 & 3, preserving abstract intermediates.
#Program & {
	entry: {
		kind: "unify"
		left: {
			kind: "unify"
			left: {
				kind: "unify"
				left: {kind: "intType"}
				right: {kind: "lowerBound", value: 0}
			}
			right: {kind: "upperBound", value: 10}
		}
		right: {kind: "integer", value: 3}
	}
}
