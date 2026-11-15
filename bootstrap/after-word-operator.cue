package bootstrap

import "list"

// Pinned CUE inserts a comma after a word operator before LF, unlike symbols.
#ScanProgram & {
	body: list.Concat([#WhitespaceBody, [{
		op: "if"
		condition: {op: "flagIs", value: true}
		then: [{
			op: "if"
			condition: {op: "hasByte"}
			then: [{op: "reject", reason: "line break after word operator is unsupported"}]
		}]
	}]])
}
