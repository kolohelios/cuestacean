package bootstrap

#LogicProgram: #BinaryControlProgram & {
	_short: bool
	let Fail = {op: "fail", reason: "logical"}
	result: {
		op: "if", condition: {op: "and", left: (#ConcreteSide & {_side: "left"}).condition, right: (#ConcreteSide & {_side: "right"}).condition}
		then: {
			op: "if", condition: {op: "kindIs", input: "left", kind: "boolean"}
			then: {
				op: "if", condition: {op: "booleanIs", input: "left", value: _short}
				then: {op: "boolean", value: _short}
				else: {op: "if", condition: {op: "kindIs", input: "right", kind: "boolean"}, then: {op: "input", input: "right"}, else: Fail}
			}
			else: Fail
		}
		else: Fail
	}
}
