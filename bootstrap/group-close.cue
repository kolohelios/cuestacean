package bootstrap

import "list"

// Expression recursion stays native; whitespace, closing-token consumption,
// and the diagnostic belong to this program.
#ScanProgram & {
	body: list.Concat([#WhitespaceBody, [{
		op: "if"
		condition: {op: "byteIn", bytes: [41]}
		then: [{op: "advance"}]
		else: [{op: "reject", reason: "expected closing parenthesis"}]
	}]])
}
