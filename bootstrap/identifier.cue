package bootstrap

import "list"

// ASCII letters/underscore at the start; digits are allowed only afterwards.
let startBytes = list.Concat([list.Range(65, 91, 1), [95], list.Range(97, 123, 1)])
let continuationBytes = list.Concat([list.Range(48, 58, 1), startBytes])

#ScanProgram & {
	body: [{
		op: "if"
		condition: {op: "byteIn", bytes: startBytes}
		then: [{op: "setFlag", value: true}, {op: "advance"}, {
			op: "while"
			condition: {op: "byteIn", bytes: continuationBytes}
			body: [{op: "advance"}]
		}]
	}]
}
