package bootstrap

import "list"

// Conservatively respect CUE comma insertion: newline + more input is rejected.
// A newline at EOF is trailing whitespace, not a separate declaration.
#ScanProgram & {
	body: list.Concat([#WhitespaceBody, [{
		op: "if"
		condition: {op: "flagIs", value: true}
		then: [{
			op: "if"
			condition: {op: "hasByte"}
			then: [{op: "reject", reason: "line break after operand is unsupported"}]
		}]
	}]])
}
