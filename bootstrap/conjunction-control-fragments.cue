package bootstrap

#ConjunctionControl: #PrattProgram & {
	target:   "conjunction"
	_forward: *true | false
	body: [{op: "seedTop"}, {if _forward {op: "firstItem"}, if !_forward {op: "lastItem"}}, {
		op: "while", condition: {op: "hasItem"}
		body: [{op: "evaluateItem"}, {op: "unifyItem"}, {if _forward {op: "nextItem"}, if !_forward {op: "previousItem"}}]
	}, {op: "finishConjunction"}]
}
