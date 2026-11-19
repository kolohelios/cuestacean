package bootstrap

// Go-only validation bridge. Native construction materializes the two ordinary
// data providers independently; neither this definition nor its schema is loaded.
#AstLowering: #LoweringPolicy & {
	cases: {
		program: _
		parameter: _
		argument: _
		string: _
		integer: _
		object: _
		field: _
	}
}
