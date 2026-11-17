package bootstrap

#HexDigitStep: {
	op: "if", condition: {op: "byteIn", bytes: [48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70, 97, 98, 99, 100, 101, 102]}
	then: [{op: "advance"}]
	else: [{op: "reject", reason: "invalid Unicode escape"}]
}
#FourHex: [#HexDigitStep, #HexDigitStep, #HexDigitStep, #HexDigitStep]
#SurrogateLead: {op: "byteIn", bytes: [68, 100]}
#HighTail: {op: "byteIn", ahead: 1, bytes: [56, 57, 65, 66, 97, 98]}
#LowTail: {op: "byteIn", ahead: 1, bytes: [67, 68, 69, 70, 99, 100, 101, 102]}
#UnicodeEscape: [
	{op: "advance"}, // u; the backslash has already been consumed.
	{op: "if", condition: #SurrogateLead, then: [{op: "if", condition: #HighTail, then: [{op: "setFlag", value: true}]}]},
	{op: "if", condition: #SurrogateLead, then: [{op: "if", condition: #LowTail, then: [{op: "reject", reason: "unexpected low surrogate escape"}]}]},
	for step in #FourHex {step},
	{op: "if", condition: {op: "flagIs", value: true}, then: [
		{op: "if", condition: {op: "byteIn", bytes: [92]}, then: [{
			op: "if", condition: {op: "byteIn", ahead: 1, bytes: [117]}
			then: [{op: "advance"}, {op: "advance"}]
			else: [{op: "reject", reason: "missing low surrogate escape"}]
		}], else: [{op: "reject", reason: "missing low surrogate escape"}]},
		{op: "if", condition: #SurrogateLead, then: [{
			op: "if", condition: #LowTail, then: [for step in #FourHex {step}]
			else: [{op: "reject", reason: "expected low surrogate escape"}]
		}], else: [{op: "reject", reason: "expected low surrogate escape"}]},
		{op: "setFlag", value: false},
	]},
]

// Compose pure prefix reads instead of consuming statements for nested tests.
#ComposedUnicodeEscape: [
	{op: "advance"},
	{op: "if", condition: {op: "and", left: #SurrogateLead, right: #HighTail}, then: [{op: "setFlag", value: true}]},
	{op: "if", condition: {op: "and", left: #SurrogateLead, right: #LowTail}, then: [{op: "reject", reason: "unexpected low surrogate escape"}]},
	for step in #FourHex {step},
	{op: "if", condition: {op: "flagIs", value: true}, then: [
		{op: "if", condition: {op: "and", left: {op: "byteIn", bytes: [92]}, right: {op: "byteIn", ahead: 1, bytes: [117]}}
			then: [{op: "advance"}, {op: "advance"}]
			else: [{op: "reject", reason: "missing low surrogate escape"}]},
		{op: "if", condition: {op: "and", left: #SurrogateLead, right: #LowTail}, then: [for step in #FourHex {step}]
			else: [{op: "reject", reason: "expected low surrogate escape"}]},
		{op: "setFlag", value: false},
	]},
]

#FourHexRepeat: [{op: "repeat", count: 4, body: [#HexDigitStep]}]
#RepeatedUnicodeEscape: [
	{op: "advance"},
	{op: "if", condition: {op: "and", left: #SurrogateLead, right: #HighTail}, then: [{op: "setFlag", value: true}]},
	{op: "if", condition: {op: "and", left: #SurrogateLead, right: #LowTail}, then: [{op: "reject", reason: "unexpected low surrogate escape"}]},
	for step in #FourHexRepeat {step},
	{op: "if", condition: {op: "flagIs", value: true}, then: [
		{op: "if", condition: {op: "and", left: {op: "byteIn", bytes: [92]}, right: {op: "byteIn", ahead: 1, bytes: [117]}}
			then: [{op: "advance"}, {op: "advance"}]
			else: [{op: "reject", reason: "missing low surrogate escape"}]},
		{op: "if", condition: {op: "and", left: #SurrogateLead, right: #LowTail}, then: [for step in #FourHexRepeat {step}]
			else: [{op: "reject", reason: "expected low surrogate escape"}]},
		{op: "setFlag", value: false},
	]},
]

#WideZeroPair: {op: "and", left: {op: "byteIn", bytes: [48]}, right: {op: "byteIn", ahead: 1, bytes: [48]}}
#WideRangePrefix: {
	op: "and", left: #WideZeroPair
	right: {op: "or", left: {op: "byteIn", ahead: 2, bytes: [48]}, right: {
		op: "and", left: {op: "byteIn", ahead: 2, bytes: [49]}, right: {op: "byteIn", ahead: 3, bytes: [48]}
	}}
}
#WideSurrogate: {
	op: "and"
	left: {op: "and", left: #WideZeroPair, right: {
		op: "and", left: {op: "byteIn", ahead: 2, bytes: [48]}, right: {op: "byteIn", ahead: 3, bytes: [48]}
	}}
	right: {op: "and", left: {op: "byteIn", ahead: 4, bytes: [68, 100]}, right: {
		op: "byteIn", ahead: 5, bytes: [56, 57, 65, 66, 67, 68, 69, 70, 97, 98, 99, 100, 101, 102]
	}}
}
#WideUnicodeEscape: [
	{op: "advance"},
	{op: "if", condition: #WideRangePrefix, then: [
		{op: "if", condition: #WideSurrogate, then: [{op: "setFlag", value: true}]},
	], else: [{op: "setFlag", value: true}]},
	{op: "repeat", count: 8, body: [#HexDigitStep]},
	{op: "if", condition: {op: "flagIs", value: true}, then: [{op: "reject", reason: "invalid Unicode scalar escape"}]},
]

#StringLiteralScanner: #ScanProgram & {
	_wide: *[{op: "if", condition: {op: "byteIn", bytes: [85]}, then: [for step in #WideUnicodeEscape {step}], else: [{op: "reject", reason: "unsupported string escape"}]}] | [{op: "reject", reason: "unsupported string escape"}]
	_unicode: *#RepeatedUnicodeEscape | #ComposedUnicodeEscape | #UnicodeEscape | [{op: "reject", reason: "unsupported string escape"}]
	body: [{
		op: "if", condition: {op: "byteIn", bytes: [34]}, then: [
			{op: "advance"},
			{op: "while", condition: {op: "hasByte"}, body: [{
				op: "if", condition: {op: "byteIn", bytes: [34]}
				then: [{op: "advance"}, {op: "setFlag", value: true}, {op: "break"}]
				else: [{
					op: "if", condition: {op: "byteIn", bytes: [92]}
					then: [{op: "advance"}, {
						op: "if", condition: {op: "byteIn", bytes: [34, 92, 98, 102, 110, 114, 116]}
						then: [{op: "advance"}], else: [{
							op: "if", condition: {op: "byteIn", bytes: [117]}
							then: [for step in _unicode {step}]
							else: [for step in _wide {step}]
						}]
					}]
					else: [{
						op: "if", condition: {op: "byteIn", bytes: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31]}
						then: [{op: "reject", reason: "unsupported character in string"}]
						else: [{op: "if", condition: {op: "charIn", characters: ["\ufeff"]}
							then: [{op: "reject", reason: "unsupported character in string"}], else: [{op: "advanceChar"}]}]
					}]
				}]
			}]},
			{op: "if", condition: {op: "flagIs", value: false}, then: [{op: "reject", reason: "unterminated string literal"}]},
		]
	}]
}
