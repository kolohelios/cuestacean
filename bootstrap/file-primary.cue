package bootstrap

#PrattProgram & {body: [{op: "probeFile"}, {
	op: "if", condition: {op: "hasFileField"}, then: [
		{op: "startFile"}, {op: "while", condition: {op: "hasFileField"}, body: [
			{op: "parseFileLabel"}, {op: "parseFileColon"}, {op: "parseFileValue"},
			{op: "appendFileField"}, {op: "advanceFileTail"},
		]}, {op: "buildFile"},
	], else: [{op: "parseFileExpression"}]
}]}
