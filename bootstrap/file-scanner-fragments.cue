package bootstrap

import "list"

let Start = list.Concat([list.Range(65, 91, 1), list.Range(97, 123, 1)])
let Continue = list.Concat([Start, list.Range(48, 58, 1), [95]])
#FileLabel: [{op: "if", condition: {op: "byteIn", bytes: Start}, then: [
	{op: "advance"}, {op: "setFlag", value: true},
	{op: "while", condition: {op: "byteIn", bytes: Continue}, body: [{op: "advance"}]},
], else: []}]
#FileColon: [{op: "while", condition: {op: "byteIn", bytes: [32, 9]}, body: [{op: "advance"}]}, {
	op: "if", condition: {op: "byteIn", bytes: [58]}, then: [{op: "advance"}, {op: "setFlag", value: true}], else: []
}]

// Unlike expression trivia, only LF is a declaration boundary. CR is still
// trivia, and inside comments only LF terminates the comment.
#FileTrivia: [{
	op: "while", condition: {op: "hasByte"}, body: [{
		op: "if", condition: {op: "byteIn", bytes: [32, 9, 10, 13]}, then: [
			{op: "if", condition: {op: "byteIn", bytes: [10]}, then: [{op: "setFlag", value: true}], else: []}, {op: "advance"},
		], else: [{
			op: "if", condition: {op: "and", left: {op: "byteIn", bytes: [47]}, right: {op: "byteIn", ahead: 1, bytes: [47]}}
			then: [{op: "advance"}, {op: "advance"}, #LineCommentLoop], else: [{op: "break"}]
		}]
	}]
}]
