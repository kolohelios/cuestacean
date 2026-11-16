package bootstrap

// Experimental syntax and rule data; these definitions do not evaluate trees.
#Expression: #IntegerLiteral | #IntegerType | #BooleanAtom | #StringLiteral | #StringType | #LowerBound | #UpperBound | #Addition | #Subtraction | #Multiplication | #Division | #Modulo | #Quotient | #Remainder | #Unification | #Negation

#IntegerLiteral: {
	kind:  "integer"
	value: int
}

#IntegerType: {kind: "intType"}
#BooleanAtom: {kind: "true" | "false" | "boolType"}
#StringLiteral: {kind: "string", value: string}
#StringType: {kind: "stringType"}

#LowerBound: {
	kind:  "lowerBound"
	value: int
}

#UpperBound: {
	kind:  "upperBound"
	value: int
}

#Negation: {
	kind:    "negate"
	operand: #Expression
}

#Addition: {
	kind:  "add"
	left:  #Expression
	right: #Expression
}

#Subtraction: {
	kind:  "subtract"
	left:  #Expression
	right: #Expression
}

#Multiplication: {
	kind:  "multiply"
	left:  #Expression
	right: #Expression
}

#Division: {
	kind:  "divide"
	left:  #Expression
	right: #Expression
}

#Modulo: {
	kind:  "modulo"
	left:  #Expression
	right: #Expression
}

#Quotient: {
	kind:  "quotient"
	left:  #Expression
	right: #Expression
}

#Remainder: {
	kind:  "remainder"
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

// Instruction-to-Rust lowering expressions. Paths come only from trusted policy;
// instruction string fields are quoted as data, never inserted as Rust syntax.
#LoweringPath: string & =~"^[A-Za-z_][A-Za-z0-9_]*(::[A-Za-z_][A-Za-z0-9_]*)*$"
#LoweringExpression: {op: "path", value: #LoweringPath} | {
	op: "call", path: #LoweringPath, args: [...#LoweringExpression]
} | {op: "ok" | "group" | "try", value: #LoweringExpression} | {
	op: "lower" | "string" | "boolean" | "unsigned" | "signed" | "list" | "bytePattern" | "charPattern", field: string & !=""
} | {op: "text", value: string} | {op: "concat", parts: [...#LoweringExpression]} | {
	op: "ifEmpty" | "ifZero" | "ifTrue", field: string & !="", then: #LoweringExpression, else: #LoweringExpression
} | {op: "select", field: string & !="", cases: [string & !=""]: #LoweringExpression} | {
	op: "join", field: string & !="", separator: string
} | {
	op: "identifier", field: string & !="", snake?: bool
} | {
	op: "bind", name: string & !="", value: #LoweringExpression, body: #LoweringExpression
} | {
	op: "get", name: string & !=""
} | {
	op: "ifEqual", left: #LoweringExpression, right: #LoweringExpression, then: #LoweringExpression, else: #LoweringExpression
}
#ModuleStatement: {op: "text", value: string} | {
	op: "firstRule" | "lastRule" | "nextRule" | "previousRule" | "kind" | "expression"
} | {op: "whileRules", body: [...#ModuleStatement]}
#LoweringPolicy: {
	formatVersion: 1
	cases: [string & !=""]:    #LoweringExpression
	aliases?: [string & !=""]: string & !=""
	module?: [...#ModuleStatement]
}

// Pure finite unification control. Local endpoints are scoped by letBounds.
#UnifySide:         "left" | "right"
#UnifyEndpointName: "lower" | "upper"
#UnifyEndpoint: {op: "input", input: #UnifySide, endpoint: #UnifyEndpointName} | {op: "local", endpoint: #UnifyEndpointName} | {op: "empty"} | {
	op: "choose", condition: #UnifyCondition, then: #UnifyEndpoint, else: #UnifyEndpoint
}
#UnifyCondition: {op: "kindIs", input: #UnifySide, kind: "integer" | "int" | "bounds" | "boolean" | "bool" | "text" | "string"} | {op: "same"} | {op: "hasEndpoint", value: #UnifyEndpoint} | {
	op: "compare", left: #UnifyEndpoint, right: #UnifyEndpoint, relation: "equal" | "less" | "greater"
} | {op: "and" | "or", left: #UnifyCondition, right: #UnifyCondition}
#UnifyExpression: {op: "input", input: #UnifySide} | {
	op: "if", condition: #UnifyCondition, then: #UnifyExpression, else: #UnifyExpression
} | {
	op: "letBounds", lower: #UnifyEndpoint, upper: #UnifyEndpoint, result: #UnifyExpression
} | {op: "bounds", lower: #UnifyEndpoint, upper: #UnifyEndpoint} | {op: "fail", reason: "integers" | "bounds" | "booleans" | "types" | "strings"}
#UnifyProgram: {formatVersion: 1, result: #UnifyExpression}

// Expression-parser control flow; native frame operations are explicit intrinsics.
#PrattNumber: {op: "minimum"} | {op: "precedence"} | {op: "tokenLength"} | {op: "integer", value: uint & <=255} | {
	op:    "add"
	left:  #PrattNumber
	right: #PrattNumber
}
#PrattCondition: {op: "hasOperator" | "leftAssociative" | "groupOpen" | "hasBound" | "hasIdentifier" | "hasNegation" | "hasString"} | {op: "boolean", value: bool} | {
	op:    "atLeast"
	left:  #PrattNumber
	right: #PrattNumber
} | {
	op:    "and"
	left:  #PrattCondition
	right: #PrattCondition
}
#PrattStatement: {op: "primary" | "afterOperand" | "afterWordOperator" | "lookupOperator" | "advanceOperator" | "parseRight" | "buildBinary" | "whitespace" | "scanGroup" | "advanceGroup" | "closeGroup" | "scanBound" | "advanceBound" | "parseBound" | "scanIdentifier" | "parseAtom" | "parseInteger" | "scanString" | "parseString" | "scanNegation" | "advanceNegation" | "buildNegation"} | {
	op:      "parseGroup" | "parseNegation"
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
#ScanPredicate: #ScanByteIn | #ScanCharIn | #ScanFlagIs | #ScanHasByte
#ScanByteIn: {
	ahead?: uint & <=64
	op:     "byteIn"
	bytes: [uint & <=127, ...uint & <=127]
}
#ScanCharIn: {
	op: "charIn"
	characters: [string & =~"(?s)^.$", ...string & =~"(?s)^.$"]
}
#ScanFlagIs: {op: "flagIs", value: bool}
#ScanHasByte: {op: "hasByte"}

#ScanStatement: #ScanWhile | #ScanIf | #ScanAdvance | #ScanAdvanceChar | #ScanBreak | #ScanSetFlag | #ScanReject
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
#ScanAdvanceChar: {op: "advanceChar"}
#ScanBreak: {op: "break"}
#ScanSetFlag: {op: "setFlag", value: bool}
#ScanReject: {op: "reject", reason: string & !=""}
#ScanProgram: {
	formatVersion: 1
	body: [...#ScanStatement]
}

// Pratt configuration: tokens and constructors come from data. #PrattProgram
// owns expression and primary control flow; grammar lookup stays native.
#InfixOperator: {
	token:         string & =~"^([-+*/&]|div|mod|quo|rem)$"
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
	negation?: {
		token:        "-"
		kind:         string & !=""
		operandField: string & !="" & !="kind"
	}
	text?: {kind: string & !="", valueField: string & !="" & !="kind"}
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
		boolType: result: {op: "bool"}
		stringType: result: {op: "stringType"}
		string: result: {op: "text", field: "value"}
		"true": result: {op: "boolean", value: bool}
		"false": result: {op: "boolean", value: bool}
		lowerBound: result: {op: "lowerBound", value: {op: "field", field: "value"}}
		upperBound: result: {op: "upperBound", value: {op: "field", field: "value"}}
		negate: result: {op: "negate", value: {op: "evaluate", field: "operand"}}
		integer: result: {
			op:    "field"
			field: "value"
		}
		add: result: {
			op:    "sum"
			left:  #EvaluateChild
			right: #EvaluateChild
		}
		subtract: result: {
			op:    "difference"
			left:  #EvaluateChild
			right: #EvaluateChild
		}
		multiply: result: {
			op:    "product"
			left:  #EvaluateChild
			right: #EvaluateChild
		}
		divide: result: {
			op:    "divide"
			left:  #EvaluateChild
			right: #EvaluateChild
		}
		modulo: result: {
			op:    "modulo"
			left:  #EvaluateChild
			right: #EvaluateChild
		}
		quotient: result: {
			op:    "quotient"
			left:  #EvaluateChild
			right: #EvaluateChild
		}
		remainder: result: {
			op:    "remainder"
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
