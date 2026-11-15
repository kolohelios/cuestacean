package bootstrap

#PrattProgram & {
	body: [{op: "whitespace"}, {op: "scanGroup"}, {
		op: "if"
		condition: {op: "groupOpen"}
		then: [{op: "advanceGroup"}, {op: "parseGroup", minimum: {op: "integer", value: 100}}, {op: "closeGroup"}]
		else: [{op: "scanNegation"}, {
			op: "if"
			condition: {op: "hasNegation"}
			then: [{op: "advanceNegation"}, {op: "parseNegation", minimum: {op: "integer", value: 101}}, {op: "buildNegation"}]
			else: [{op: "scanBound"}, {
				op: "if"
				condition: {op: "hasBound"}
				then: [{op: "advanceBound"}, {op: "whitespace"}, {op: "parseBound"}]
				else: [{op: "scanIdentifier"}, {
					op: "if"
					condition: {op: "hasIdentifier"}
					then: [{op: "parseAtom"}]
					else: [{op: "parseInteger"}]
				}]
			}]
		}]
	}]
}
