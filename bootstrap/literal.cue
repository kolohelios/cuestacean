package bootstrap

// A serialization fixture, not yet the compiler's intermediate representation.
#IntegerLiteral: {
	kind:  "integer"
	value: int
}

formatVersion: 1
entry: #IntegerLiteral & {value: 42}
