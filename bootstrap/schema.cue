package bootstrap

// Experimental syntax and rule data; these definitions do not evaluate trees.
#Expression: #IntegerLiteral | #Addition

#IntegerLiteral: {
	kind:  "integer"
	value: int
}

#Addition: {
	kind:  "add"
	left:  #Expression
	right: #Expression
}

#Program: {
	formatVersion: 1
	entry:         #Expression
}

// An instruction to recursively evaluate a child of the current input node.
#EvaluateChild: {
	op:    "evaluate"
	field: "left" | "right"
}

// Rules are selected by the input node's kind. Their result instructions are
// data for the bootstrap executor, not CUE expressions that compute an answer here.
#EvaluationRules: {
	formatVersion: 1
	rules: {
		integer: result: {
			op:    "field"
			field: "value"
		}
		add: result: {
			op:    "sum"
			left:  #EvaluateChild
			right: #EvaluateChild
		}
	}
}
