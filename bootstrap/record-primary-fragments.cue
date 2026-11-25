package bootstrap

#RecordPrimaryControl: #PrattProgram & {
	_minimum: #RecordMinimumInput.v
	body: [{op: "advanceRecord"}, {op: "openRecordCall"}, {op: "recordWhitespace"}, {op: "openRecordBody"},
		{op: "recordWhitespace"}, {op: "probeRecordEnd"}, {
			op: "while", condition: {op: "hasRecordField"}, body: [
				{op: "requireRecordLabel"}, {op: "parseRecordLabel"}, {op: "recordLabelTrivia"}, {op: "recordColon"},
				{op: "parseRecordValue", minimum: {op: "integer", value: _minimum}}, {op: "appendRecordField"}, {op: "advanceRecordTail"},
				{op: "if", condition: {op: "hasRecordField"}, then: [{op: "recordWhitespace"}, {op: "probeRecordEnd"}]},
			]
		}, {op: "closeRecordCall"}, {op: "buildRecord"}]
}
