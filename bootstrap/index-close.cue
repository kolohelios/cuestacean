package bootstrap

import "list"

#ScanProgram & {
	body: list.Concat([#WhitespaceBody, [{
		op: "if", condition: {op: "byteIn", bytes: [93]}
		then: [{op: "advance"}]
		else: [{op: "reject", reason: "expected closing bracket"}]
	}]])
}
