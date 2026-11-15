package bootstrap

// Experimental syntax and rule data; these definitions do not evaluate trees.
#Expression: #IntegerLiteral | #IntegerType | #Addition | #Multiplication | #Unification

#IntegerLiteral: {
	kind:  "integer"
	value: int
}

#IntegerType: {kind: "intType"}

#Addition: {
	kind:  "add"
	left:  #Expression
	right: #Expression
}

#Multiplication: {
	kind:  "multiply"
	left:  #Expression
	right: #Expression
}

#Unification: {
	kind:  "unify"
	left:  #Expression
	right: #Expression
}

#Program: {
	formatVersion: 1
	entry:         #Expression
}

// Configuration for a bootstrap Pratt parser. The parsing algorithm itself
// remains handwritten initially; tokens and AST construction come from data.
#InfixOperator: {
	token:         string & =~"^[+*/&]$"
	kind:          string & !=""
	leftField:     string & !="" & !="kind"
	rightField:    string & !="" & !="kind" & !=leftField
	precedence:    int & >=1 & <=100
	associativity: "left" | "right"
}

#Atom: {
	token: string & =~"^[a-zA-Z_][a-zA-Z0-9_]*$"
	kind:  string & !=""
}

#Grammar: {
	formatVersion: 1
	integer: {
		kind:       string & !=""
		valueField: string & !="" & !="kind"
	}
	atoms: *[] | [...#Atom]
	group: {open: "(", close: ")"}
	operators: [#InfixOperator, ...#InfixOperator]
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
		intType: result: {op: "int"}
		integer: result: {
			op:    "field"
			field: "value"
		}
		add: result: {
			op:    "sum"
			left:  #EvaluateChild
			right: #EvaluateChild
		}
		multiply: result: {
			op:    "product"
			left:  #EvaluateChild
			right: #EvaluateChild
		}
		unify: result: {
			op:    "unify"
			left:  #EvaluateChild
			right: #EvaluateChild
		}
	}
}
