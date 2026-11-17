package bootstrap

// Historical primary algorithm, before early-return flattening.
#PrattProgram & {
	body: [{op: "whitespace"}, {op: "scanGroup"}, {
		op: "if"
		condition: {op: "groupOpen"}
		then: [{op: "advanceGroup"}, {op: "parseGroup", minimum: {op: "integer", value: 0}}, {op: "closeGroup"}]
		else: [{op: "scanNot"}, {
			op: "if", condition: {op: "hasNot"}
			then: [{op: "advanceNot"}, {op: "parseNot", minimum: {op: "integer", value: 101}}, {op: "buildNot"}]
			else: [{op: "scanNegation"}, {
				op: "if"
				condition: {op: "hasNegation"}
				then: [{op: "advanceNegation"}, {op: "parseNegation", minimum: {op: "integer", value: 101}}, {op: "buildNegation"}]
				else: [{op: "scanBound"}, {
					op: "if"
					condition: {op: "hasBound"}
					then: [{op: "advanceBound"}, {op: "whitespace"}, {op: "scanGroup"}, {
						op: "if", condition: {op: "groupOpen"}
						then: [{op: "advanceGroup"}, {op: "parseGroup", minimum: {op: "integer", value: 0}}, {op: "closeGroup"}]
						else: [{op: "parseBound"}]
					}]
					else: [{op: "scanString"}, {
						op: "if", condition: {op: "hasString"}
						then: [{op: "parseString"}]
						else: [{op: "scanIdentifier"}, {
							op: "if", condition: {op: "hasIdentifier"}
							then: [{op: "parseAtom"}], else: [{op: "parseInteger"}]
						}]
					}]
				}]
			}]}]
	}]
}
