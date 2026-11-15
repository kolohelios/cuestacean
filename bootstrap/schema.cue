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

// Expression-parser control flow; native frame operations are explicit intrinsics.
#PrattNumber: {op: "minimum"} | {op: "precedence"} | {op: "integer", value: uint & <=255} | {
	op:    "add"
	left:  #PrattNumber
	right: #PrattNumber
}
#PrattCondition: {op: "hasOperator" | "leftAssociative" | "groupOpen" | "hasBound" | "hasIdentifier"} | {op: "boolean", value: bool} | {
	op:    "atLeast"
	left:  #PrattNumber
	right: #PrattNumber
} | {
	op:    "and"
	left:  #PrattCondition
	right: #PrattCondition
}
#PrattStatement: {op: "primary" | "afterOperand" | "lookupOperator" | "advanceOperator" | "parseRight" | "buildBinary" | "whitespace" | "scanGroup" | "advanceGroup" | "closeGroup" | "scanBound" | "advanceBound" | "parseBound" | "scanIdentifier" | "parseAtom" | "parseInteger"} | {
	op:      "parseGroup"
	minimum: #PrattNumber
} | {
	op:    "setBindingPower"
	value: #PrattNumber
} | {
	op:        "while"
	condition: #PrattCondition
	body: [...#PrattStatement]
} | {
	op:        "if"
	condition: #PrattCondition
	then: [...#PrattStatement]
	else: *[] | [...#PrattStatement]
}
#PrattProgram: {formatVersion: 1, body: [...#PrattStatement]}

// Typed JSON construction expressions used by the parser's AST builders.
#BuildParameter: {
	name: string & !=""
	type: "string" | "integer" | "json"
}
#BuildArgument: {op: "argument", name: string & !=""}
#BuildString: {op: "string", value: string}
#BuildInteger: {op: "integer", value: int}
#BuildExpression: #BuildArgument | #BuildString | #BuildInteger | #BuildObject
#BuildObject: {
	op: "object"
	fields: [...{
		key:   #BuildString | #BuildArgument
		value: #BuildExpression
	}]
}
#BuildProgram: {
	formatVersion: 1
	parameters: [...#BuildParameter]
	result: #BuildExpression
}

// Executable scanner statements, separate from expression evaluation rules.
// Control flow lives in these programs; cursor safety remains native.
#ScanPredicate: #ScanByteIn | #ScanFlagIs | #ScanHasByte
#ScanByteIn: {
	ahead?: uint & <=64
	op:     "byteIn"
	bytes: [uint & <=127, ...uint & <=127]
}
#ScanFlagIs: {op: "flagIs", value: bool}
#ScanHasByte: {op: "hasByte"}

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

// Pratt configuration: tokens and constructors come from data. #PrattProgram
// owns expression and primary control flow; grammar lookup stays native.
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
