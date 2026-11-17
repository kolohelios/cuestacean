package bootstrap

#ListControl: #PrattProgram & {
	target: "list"
	_first: *"firstItem" | "lastItem"
	_next:  *"nextItem" | "previousItem"
	let Append = {op: "appendItem"}
	let Admit = {
		op: "if", condition: {op: "itemKindIs", kind: "null"}, then: [Append]
		else: [{op: "if", condition: {op: "itemKindIs", kind: "boolean"}, then: [Append]
			else: [{op: "if", condition: {op: "itemKindIs", kind: "integer"}, then: [Append]
				else: [{op: "if", condition: {op: "itemKindIs", kind: "text"}, then: [Append], else: [{op: "rejectItem"}]}]
			}]
		}]
	}
	body: [{op: _first}, {op: "while", condition: {op: "hasItem"}, body: [
		{op: "evaluateItem"}, Admit, {op: _next},
	]}, {op: "finishList"}]
}
