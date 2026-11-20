package bootstrap

// Go-only schema bridge: every policy slot needs an independent provider.
#PrattLowering: #LoweringPolicy & {
	aliases: {
		parseSliceUpper: _, parseRecordValue: _, parseIndex:    _, parseListElement: _
		parseGroup:      _, parseNot:         _, parseNegation: _
	}
	cases: {
		action:         _, program:      _, "return":          _, minimum:        _, tokenLength: _, precedence:    _, integer: _
		add:            _, hasOperator:  _, leftAssociative:   _, groupOpen:      _, hasBound:    _, hasIdentifier: _
		hasCall:        _, hasRecord:    _, hasConjunction:    _, hasRecordField: _, hasList:     _, hasSlice:      _
		hasIndex:       _, hasPostfix:   _, hasSelectorString: _, hasListElement: _, hasString:   _
		hasNegation:    _, hasNot:       _, hasLower:          _, hasUpper:       _, sliceKindIs: _, defaultSliceLower: _
		hasItem:        _, itemKindIs:   _, boolean:           _, atLeast:        _, and:         _, setBindingPower:   _
		selectorAction: _, parseMinimum: _, while:             _, "if":           _
	}
}
