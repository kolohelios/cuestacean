package bootstrap

#SliceControl: #PrattProgram & {
	target:        "slice"
	_lowerDefault: #SliceLowerInput.v
	body: [{op: "evaluateSliceOperand"}, {
		op: "if", condition: {op: "hasLower"}, then: [{op: "evaluateSliceLower"}], else: [{op: "defaultSliceLower", value: _lowerDefault}]
	}, {
		op: "if", condition: {op: "hasUpper"}, then: [{op: "evaluateSliceUpper"}], else: [{
			op: "if", condition: {op: "sliceKindIs", kind: "list"}, then: [{op: "defaultSliceUpper"}], else: [{op: "rejectSliceOperand"}]
		}]
	}, {op: "finishSlice"}]
}
