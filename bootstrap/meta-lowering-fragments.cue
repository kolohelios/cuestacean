package bootstrap

// Go-only validation bridge; represented instructions live in the raw providers.
#MetaLowering: #LoweringPolicy & {
	aliases: {
		text:     string
		nextRule: string
	}
	cases: {
		driver:           _
		dispatchCase:     _
		dispatchKey:      _
		moduleText:       _
		whileRules:       _
		expression:       _
		emitModuleAction: _
		emitField:        _
		emitValue:        _
		emitRecursive:    _
		emitUnary:        _
		emitConditional:  _
		join:             _
		bind:             _
		get:              _
		ifEqual:          _
		identifier:       _
		select:           _
		selectCase:       _
		concat:           _
		concatArgument:   _
		call:             _
		callArgument:     _
	}
}
