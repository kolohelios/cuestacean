package bootstrap

// Deliberately invalid execution: try expression operations in primary scope,
// both with a parsed-but-unclosed group and with a completed atom.
#PrattProgram & {body: [
	{op: "whitespace"},
	{op: "scanGroup"},
	{op: "if", condition: {op: "groupOpen"}, then: [
		{op: "advanceGroup"},
		{op: "parseGroup", minimum: {op: "integer", value: 0}},
		{op: "lookupOperator"},
		{op: "closeGroup"},
	], else: [
		{op: "scanBound"},
		{op: "if", condition: {op: "hasBound"}, then: [
			{op: "advanceBound"}, {op: "whitespace"}, {op: "parseBound"},
		], else: [
			{op: "scanIdentifier"},
			{op: "if", condition: {op: "hasIdentifier"}, then: [{op: "parseAtom"}, {op: "lookupOperator"}], else: [{op: "parseInteger"}]},
		]},
	]},
]}
