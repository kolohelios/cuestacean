package bootstrap

#ScanProgram & {body: [{
	op: "if", condition: {op: "byteIn", bytes: [34]}, then: [
		{op: "advance"},
		{op: "while", condition: {op: "hasByte"}, body: [{
			op: "if", condition: {op: "byteIn", bytes: [34]}
			then: [{op: "advance"}, {op: "setFlag", value: true}, {op: "break"}]
			else: [{
				op: "if", condition: {op: "byteIn", bytes: [92]}
				then: [{op: "advance"}, {
					op: "if", condition: {op: "byteIn", bytes: [34, 92, 98, 102, 110, 114, 116]}
					then: [{op: "advance"}], else: [{op: "reject", reason: "unsupported string escape"}]
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
}]}
