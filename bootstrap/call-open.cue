package bootstrap

import "list"

// Required opener after a registered name; trivia and diagnostics stay CUE-owned.
#ScanProgram & {
	body: list.Concat([#WhitespaceBody, [{
		op: "if", condition: {op: "flagIs", value: true}
		then: [{op: "reject", reason: "newline after call name"}]
	}, {
		op: "if", condition: {op: "byteIn", bytes: [40]}
		then: [{op: "advance"}, {op: "setFlag", value: true}]
		else: [{op: "reject", reason: "expected opening parenthesis"}]
	}]])
}
