package bootstrap

// Exercise Unicode literals, nearest-loop break and unreachable suffix omission.
#ScanProgram & {body: [{
	op: "while"
	condition: {op: "hasByte"}
	body: [{
		op: "if"
		condition: {op: "charIn", characters: ["λ", "😀", "'", "\\", "\n"]}
		then: [
			{op: "advanceChar"},
			{op: "while", condition: {op: "hasByte"}, body: [
				{op: "if", condition: {op: "byteIn", bytes: [33]}, then: [{op: "break"}], else: [{op: "break"}]},
				{op: "reject", reason: "unreachable after both branches break"},
			]},
			{op: "setFlag", value: true},
			{op: "break"},
			{op: "reject", reason: "unreachable after direct break"},
		]
		else: [{op: "advanceChar"}]
	}, {op: "setFlag", value: false}]
}]}
