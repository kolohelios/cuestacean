package bootstrap

#PrattControl: #PrattProgram & {
	_file:          *false | true
	_postfix:       *true | false
	_selectors:     *true | false
	_slices:        *true | false
	_sliceMinimum:  #PrattSliceMinimumInput.v
	_associativity: *true | false
	_indexMinimum:  #PrattIndexMinimumInput.v
	let Index = [{op: "advanceIndex"}, {op: "parseIndex", minimum: {op: "integer", value: _indexMinimum}},
		if _slices {op: "scanSlice"}, if _slices {
			op: "if", condition: {op: "hasSlice"}
			then: [{op: "advanceSlice"}, {op: "parseSliceUpper", minimum: {op: "integer", value: _sliceMinimum}}, {op: "closeSlice"}]
			else: [{op: "closeIndex"}]
		}, if !_slices {op: "closeIndex"},
	]
	let ProbeSelector = {op: "if", condition: {op: "hasIndex"}, then: [], else: [{op: "scanSelector"}]}
	let Selector = [{op: "advanceSelector"}, {op: "selectorWhitespace"}, {op: "scanSelectorString"}, {
		op: "if", condition: {op: "hasSelectorString"}, then: [{op: "parseSelectorString"}], else: [{op: "parseSelectorIdentifier"}]
	}, {op: "buildSelector"}]
	let After = [if !_file {op: "afterOperand"}, if _file {op: "fileAfterOperand"}, if _file {
		op: "if", condition: {op: "fileBoundary"}, then: [{op: "return"}]
	}]
	body: [{op: "primary"}, for action in After {action},
		if _postfix {op: "scanIndex"}, if _postfix && _selectors {ProbeSelector}, if _postfix {
			op: "while", condition: {if _selectors {op: "hasPostfix"}, if !_selectors {op: "hasIndex"}}
			body: [
				if _selectors {op: "if", condition: {op: "hasIndex"}, then: Index, else: Selector},
				if !_selectors for action in Index {action},
				for action in After {action}, {op: "scanIndex"}, if _selectors {ProbeSelector},
			]
		}, {op: "lookupOperator"}, {
			op: "while"
			condition: {
				op: "and"
				left: {op: "hasOperator"}
				right: {op: "atLeast", left: {op: "precedence"}, right: {op: "minimum"}}
			}
			body: [{
				op: "setBindingPower"
				value: {op: "precedence"}
			}, if _associativity {
				op: "if"
				condition: {op: "leftAssociative"}
				then: [{op: "setBindingPower", value: {
					op: "add", left: {op: "precedence"}, right: {op: "integer", value: 1}
				}}]
			}, {op: "advanceOperator"}, {
				op: "if"
				condition: {op: "atLeast", left: {op: "tokenLength"}, right: {op: "integer", value: 3}}
				then: [{op: "afterWordOperator"}]
			}, {op: "parseRight"}, {op: "buildBinary"},
				for action in After {action}, {op: "lookupOperator"}]
		}]
}
