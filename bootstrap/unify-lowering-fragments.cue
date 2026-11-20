package bootstrap

// Go validation bridge: every original slot requires its raw DATA provider.
#UnifyLowering: #LoweringPolicy & {
	aliases: choose: _
	cases: {
		program:   _
		unaryFail: _
		integerPrimitive: cases: {
			negate:     _
			difference: _
			product:    _
			divide:     _
			modulo:     _
			quotient:   _
			remainder:  _
		}
		integerValue:      _
		integerEndpoint:   _
		boolean:           _
		integerSum:        _
		textConcat:        _
		textLength:        _
		listCount:         _
		recordCount:       _
		recordGet:         _
		recordHas:         _
		listSlice:         _
		listElement:       _
		offset:            _
		listCountEndpoint: _
		inputValue:        _
		inputEndpoint:     _
		local:             _
		empty:             _
		"if":              _
		letBounds:         _
		bounds:            _
		fail:              _
		same:              _
		or:                _
		and:               _
		kindIs:            _
		hasEndpoint:       _
		booleanIs:         _
		textCompare:       _
		compare:           _
	}
}
