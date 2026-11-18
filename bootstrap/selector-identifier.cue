package bootstrap

import "list"

// Regular ASCII labels only: a leading underscore would select a hidden field.
let start = list.Concat([list.Range(65, 91, 1), list.Range(97, 123, 1)])
let continuation = list.Concat([start, list.Range(48, 58, 1), [95]])
#ScanProgram & {body: [{op: "if", condition: {op: "byteIn", bytes: start}, then: [
	{op: "advance"}, {op: "setFlag", value: true},
	{op: "while", condition: {op: "byteIn", bytes: continuation}, body: [{op: "advance"}]},
], else: [{op: "reject", reason: "expected selector label"}]}]}
