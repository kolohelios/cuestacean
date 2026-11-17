package bootstrap

#PrattProgram & {
	body: [{op: "whitespace"}, {op: "scanGroup"}, {
		op: "if", condition: {op: "groupOpen"}
		then: [{op: "advanceGroup"}, {op: "parseGroup", minimum: {op: "integer", value: 0}}, {op: "return"}]
		else: [{op: "scanNot"}, {op: "scanNegation"}, {op: "scanBound"}, {op: "scanString"}, {op: "scanIdentifier"}, {op: "parseInteger"}, {op: "return"}]
	}]
}
