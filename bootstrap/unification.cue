package bootstrap

// Concrete integer equality is the first unification slice, not a value lattice.
#Program & {
	entry: {
		kind: "unify"
		left: {
			kind: "add"
			left: {kind: "integer", value: 1}
			right: {kind: "integer", value: 2}
		}
		right: {kind: "integer", value: 3}
	}
}
