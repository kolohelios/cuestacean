package bootstrap

#IntegerDivisionControl: #BinaryControlProgram & {
	_calculation: #UnifyExpression
	result: {
		op: "if", condition: {op: "and", left: {op: "kindIs", input: "left", kind: "integer"}, right: {op: "kindIs", input: "right", kind: "integer"}}
		then: {
			op: "if", condition: #RightIntegerEquals & {_integer: 0}
			then: {op: "fail", reason: "divisionByZero"}
			else: _calculation
		}
		else: {op: "fail", reason: "arithmetic"}
	}
}

#RightIntegerEquals: {
	_integer: int
	op: "compare", relation: "equal"
	left: {op: "input", input: "right", endpoint: "lower"}
	right: {op: "integer", value: _integer}
}

#UnitRemainder: {
	_operation: "modulo" | "remainder"
	op: "if", condition: #RightIntegerEquals & {_integer: -1}
	then: {op: "integer", value: 0}
	else: {op: "integerPrimitive", operation: _operation}
}
