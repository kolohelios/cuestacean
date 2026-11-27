package bootstrap

// No comma or new trivia admission after the tail operand.
#ScanProgram & {body: [{op: "if", condition: {op: "byteIn", bytes: [93]}
	then: [{op: "advance"}]
	else: [{op: "reject", reason: "expected closing bracket after list tail"}]
}]}
