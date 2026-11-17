package bootstrap

#OutputControl: #UnaryControlProgram & {
	_concrete: #UnaryControlExpr
	result: {
		op: "if", condition: {op: "kindIs", input: "left", kind: "top"}
		then: {op: "fail", reason: "incompleteValue"}
		else: {
			op: "if", condition: {op: "kindIs", input: "left", kind: "int"}
			then: {op: "fail", reason: "incompleteInt"}
			else: {
				op: "if", condition: {op: "kindIs", input: "left", kind: "bool"}
				then: {op: "fail", reason: "incompleteBool"}
				else: {
					op: "if", condition: {op: "kindIs", input: "left", kind: "string"}
					then: {op: "fail", reason: "incompleteString"}
					else: {
						op: "if", condition: {op: "kindIs", input: "left", kind: "bounds"}
						then: {op: "fail", reason: "incompleteBounds"}
						else: _concrete
					}
				}
			}
		}
	}
}
