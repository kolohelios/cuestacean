package bootstrap

// Experimental syntax and rule data; these definitions do not evaluate trees.
#Expression: #IntegerLiteral | #IntegerType | #LowerBound | #UpperBound | #Addition | #Multiplication | #Unification

#IntegerLiteral: {
	kind:  "integer"
	value: int
}

#IntegerType: {kind: "intType"}

#LowerBound: {
	kind:  "lowerBound"
	value: int
}

#UpperBound: {
	kind:  "upperBound"
	value: int
}

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

// Executable scanner statements, separate from expression evaluation rules.
// Control flow lives in these programs; cursor safety remains native.
#ScanPredicate: {
	op: "byteIn"
	bytes: [uint & <=127, ...uint & <=127]
}

#ScanStatement: #ScanWhile | #ScanIf | #ScanAdvance | #ScanSetFlag | #ScanReject
#ScanWhile: {
	op:        "while"
	condition: #ScanPredicate
	body: [...#ScanStatement]
}
#ScanIf: {
	op:        "if"
	condition: #ScanPredicate
	then: [...#ScanStatement]
	else: *[] | [...#ScanStatement]
}
#ScanAdvance: {op: "advance"}
#ScanSetFlag: {op: "setFlag", value: bool}
#ScanReject: {op: "reject", reason: string & !=""}
#ScanProgram: {
	formatVersion: 1
	body: [...#ScanStatement]
}

// Pratt configuration: tokens and AST constructors come from data. Pratt
// control flow remains native; whitespace/decimal/identifier scans use #ScanProgram.
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

// Prefix bounds currently accept only signed decimal literal operands.
#BoundPrefix: {
	token:      ">=" | "<="
	kind:       string & !=""
	valueField: string & !="" & !="kind"
}

#Grammar: {
	formatVersion: 1
	integer: {
		kind:       string & !=""
		valueField: string & !="" & !="kind"
	}
	atoms: *[] | [...#Atom]
	bounds: *[] | [...#BoundPrefix]
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
		lowerBound: result: {op: "lowerBound", value: {op: "field", field: "value"}}
		upperBound: result: {op: "upperBound", value: {op: "field", field: "value"}}
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
