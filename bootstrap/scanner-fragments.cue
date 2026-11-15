package bootstrap

// Only LF terminates a CUE line comment. CR stays comment content.
#LineCommentLoop: {
	op: "while"
	condition: {op: "hasByte"}
	body: [{
		op: "if"
		condition: {op: "byteIn", bytes: [10]}
		then: [{op: "break"}]
		else: [{
			op: "if"
			condition: {op: "charIn", characters: ["\u0000", "\ufeff"]}
			then: [{op: "reject", reason: "unsupported character in line comment"}]
			else: [{op: "advanceChar"}]
		}]
	}]
}

// Shared executable trivia statements, not a native skip-comments primitive.
#WhitespaceBody: [{
	op: "while"
	condition: {op: "hasByte"}
	body: [{
		op: "if"
		condition: {op: "byteIn", bytes: [32, 9, 10, 13]}
		then: [{
			op: "if"
			condition: {op: "byteIn", bytes: [10, 13]}
			then: [{op: "setFlag", value: true}]
			else: []
		}, {op: "advance"}]
		else: [{
			op: "if"
			condition: {op: "byteIn", bytes: [47]}
			then: [{
				op: "if"
				condition: {op: "byteIn", ahead: 1, bytes: [47]}
				then: [{op: "advance"}, {op: "advance"}, #LineCommentLoop]
				else: [{op: "break"}]
			}]
			else: [{op: "break"}]
		}]
	}]
}]
