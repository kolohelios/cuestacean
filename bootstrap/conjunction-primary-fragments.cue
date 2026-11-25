package bootstrap

#ConjunctionPrimary: #PrattProgram & {
	_operandMinimum: #ConjunctionMinimumInput.v
	body: [{op: "advanceConjunction"}, {op: "openConjunctionCall"}, {op: "whitespace"}, {op: "openConjunctionList"},
		{op: "whitespace"}, {op: "probeListEnd"}, {
			op: "while", condition: {op: "hasListElement"}
			body: [{op: "parseListElement", minimum: {op: "integer", value: _operandMinimum}}, {op: "appendListElement"}, {op: "advanceListTail"}, {
				op: "if", condition: {op: "hasListElement"}, then: [{op: "whitespace"}, {op: "probeListEnd"}]
			}]
		}, {op: "closeConjunctionCall"}, {op: "buildConjunction"}]
}
