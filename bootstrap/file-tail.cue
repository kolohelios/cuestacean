package bootstrap

import "list"

#ScanProgram & {body: list.Concat([#FileTrivia, [{
	op: "if", condition: {op: "and", left: {op: "hasByte"}, right: {op: "flagIs", value: false}}
	then: [{op: "reject", reason: "expected LF between file fields"}]
}, {op: "setFlag", value: false}, {
	op: "if", condition: {op: "hasByte"}, then: [{op: "setFlag", value: true}]
}]])
}
