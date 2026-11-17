package bootstrap

// Experimental syntax and rule data; these definitions do not evaluate trees.
#Expression: #NullLiteral | #TopType | #BinaryExpression | #IntegerLiteral | #IntegerType | #BooleanAtom | #StringLiteral | #StringType | #LowerBound | #UpperBound | #UnaryExpression

// Factor the identical recursive shape; separate recursive disjuncts expand
// combinatorially in the upstream exporter as binary node kinds accumulate.
#BinaryExpression: {
	kind:  "add" | "subtract" | "multiply" | "divide" | "modulo" | "quotient" | "remainder" | "unify" | "equal" | "notEqual" | "less" | "lessEqual" | "greater" | "greaterEqual" | "logicalAnd" | "logicalOr"
	left:  #Expression
	right: #Expression
}

#IntegerLiteral: {
	kind:  "integer"
	value: int
}

#NullLiteral: {kind: "null"}
#TopType: {kind: "top"}
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

#UnaryExpression: {kind: "negate" | "logicalNot" | "lowerBoundExpression" | "upperBoundExpression", operand: #Expression}
#LowerBoundExpression: #UnaryExpression & {kind: "lowerBoundExpression"}
#UpperBoundExpression: #UnaryExpression & {kind: "upperBoundExpression"}
#Negation: #UnaryExpression & {kind: "negate"}
#LogicalNot: #UnaryExpression & {kind: "logicalNot"}

#Logical: #BinaryExpression & {kind: "logicalAnd" | "logicalOr"}
#Ordering: #BinaryExpression & {kind: "less" | "lessEqual" | "greater" | "greaterEqual"}
#Equality: #BinaryExpression & {kind: "equal" | "notEqual"}
#Addition: #BinaryExpression & {kind: "add"}
#Subtraction: #BinaryExpression & {kind: "subtract"}
#Multiplication: #BinaryExpression & {kind: "multiply"}
#Division: #BinaryExpression & {kind: "divide"}
#Modulo: #BinaryExpression & {kind: "modulo"}
#Quotient: #BinaryExpression & {kind: "quotient"}
#Remainder: #BinaryExpression & {kind: "remainder"}
#Unification: #BinaryExpression & {kind: "unify"}

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
	op: "identifier", field: string & !="", snake?: bool, upperFirst?: bool
	if snake != _|_ {
		if snake {upperFirst?: false}
	}
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
#ControlInteger: {op: "integer", value: int & >=-9223372036854775808 & <=9223372036854775807}
#UnifyEndpoint: #ControlInteger | {op: "input", input: #UnifySide, endpoint: #UnifyEndpointName} | {op: "local", endpoint: #UnifyEndpointName} | {op: "empty"} | {
	op: "choose", condition: #UnifyCondition, then: #UnifyEndpoint, else: #UnifyEndpoint
}
#UnifyRelation: "equal" | "less" | "greater" | "lessEqual" | "greaterEqual"
#UnifyKind:     "null" | "top" | "integer" | "int" | "bounds" | "boolean" | "bool" | "text" | "string"
#UnifyCondition: {op: "booleanIs", input: #UnifySide, value: bool} | {op: "textCompare", relation: #UnifyRelation} | {op: "kindIs", input: #UnifySide, kind: #UnifyKind} | {op: "same"} | {op: "hasEndpoint", value: #UnifyEndpoint} | {
	op: "compare", left: #UnifyEndpoint, right: #UnifyEndpoint, relation: #UnifyRelation
} | {op: "and" | "or", left: #UnifyCondition, right: #UnifyCondition}
#IntegerOperation: "negate" | "difference" | "product" | "divide" | "modulo" | "quotient" | "remainder"
#OutputFailure:    "incompleteInt" | "incompleteValue" | "incompleteBool" | "incompleteString" | "incompleteBounds" | "integerOutput"
#UnifyExpression: #ControlInteger | {op: "integerPrimitive", operation: #IntegerOperation} | {op: "boolean", value: bool} | {op: "integerSum" | "textConcat"} | {op: "input", input: #UnifySide} | {
	op: "if", condition: #UnifyCondition, then: #UnifyExpression, else: #UnifyExpression
} | {
	op: "letBounds", lower: #UnifyEndpoint, upper: #UnifyEndpoint, result: #UnifyExpression
} | {op: "bounds", lower: #UnifyEndpoint, upper: #UnifyEndpoint} | {op: "fail", reason: #OutputFailure | "integers" | "bounds" | "booleans" | "types" | "strings" | "arithmetic" | "comparison" | "ordering" | "logical" | "divisionByZero" | "boundThreshold"}
#BinaryControlProgram: {formatVersion: 1, function?: "unify" | "sum" | "equal" | "notEqual" | "less" | "lessEqual" | "greater" | "greaterEqual" | "logicalAnd" | "logicalOr" | "difference" | "product" | "divide" | "modulo" | "quotient" | "remainder", result: #UnifyExpression}
#UnaryControlCondition: {op: "kindIs", input: "left", kind: #UnifyKind} | {op: "booleanIs", input: "left", value: bool} | {op: "and" | "or", left: #UnaryControlCondition, right: #UnaryControlCondition}
#UnaryControlEndpoint: #ControlInteger | {op: "input", input: "left", endpoint: #UnifyEndpointName} | {op: "empty"} | {
	op: "choose", condition: #UnaryControlCondition, then: #UnaryControlEndpoint, else: #UnaryControlEndpoint
}
#UnaryControlExpr: {op: "bounds", lower: #UnaryControlEndpoint, upper: #UnaryControlEndpoint} | #ControlInteger | {op: "integerPrimitive", operation: "negate"} | {op: "boolean", value: bool} | {op: "input", input: "left"} | {op: "fail", reason: #OutputFailure | "logical" | "arithmetic" | "boundThreshold"} | {op: "if", condition: #UnaryControlCondition, then: #UnaryControlExpr, else: #UnaryControlExpr}
#UnaryControlProgram: {formatVersion: 1, function: "logicalNot" | "negate" | "jsonOutput" | "integerOutput" | "lowerBound" | "upperBound", result: #UnaryControlExpr}
#UnifyProgram: *#BinaryControlProgram | #UnaryControlProgram

// Expression-parser control flow; native frame operations are explicit intrinsics.
#PrattNumber: {op: "minimum"} | {op: "precedence"} | {op: "tokenLength"} | {op: "integer", value: uint & <=255} | {
	op:    "add"
	left:  #PrattNumber
	right: #PrattNumber
}
#PrattCondition: {op: "hasOperator" | "leftAssociative" | "groupOpen" | "hasBound" | "hasIdentifier" | "hasNegation" | "hasNot" | "hasString"} | {op: "boolean", value: bool} | {
	op:    "atLeast"
	left:  #PrattNumber
	right: #PrattNumber
} | {
	op:    "and"
	left:  #PrattCondition
	right: #PrattCondition
}
#PrattStatement: {op: "return" | "primary" | "afterOperand" | "afterWordOperator" | "lookupOperator" | "advanceOperator" | "parseRight" | "buildBinary" | "whitespace" | "scanGroup" | "advanceGroup" | "closeGroup" | "scanBound" | "advanceBound" | "parseBound" | "scanIdentifier" | "parseAtom" | "parseInteger" | "scanString" | "parseString" | "scanNegation" | "advanceNegation" | "buildNegation" | "scanNot" | "advanceNot" | "buildNot"} | {
	op:      "parseGroup" | "parseNegation" | "parseNot"
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
#ScanPredicate: #ScanByteIn | #ScanCharIn | #ScanFlagIs | #ScanHasByte | {
	op: "and" | "or", left: #ScanPredicate, right: #ScanPredicate
}
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

#ScanRepeat: {op: "repeat", count: uint & <=64, body: [...#ScanStatement]}
#ScanStatement: #ScanRepeat | #ScanWhile | #ScanIf | #ScanAdvance | #ScanAdvanceChar | #ScanBreak | #ScanSetFlag | #ScanReject
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
	token:         string & =~"^([-+*/&<>]|&&|\\|\\||<=|>=|==|!=|div|mod|quo|rem)$"
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

// Literal bounds accept signed decimal/radix i64s; optional metadata enables
// parenthesized expression thresholds through the existing unary builder.
#BoundPrefix: {
	token:      ">=" | "<="
	kind:       string & !=""
	valueField: string & !="" & !="kind"
	expression?: {kind: string & !="", operandField: string & !="" & !="kind"}
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
	logicalNot?: {token: "!", kind: string & !="", operandField: string & !="" & !="kind"}
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
		"null": result: {op: "null"}
		top: result: {op: "top"}
		intType: result: {op: "int"}
		boolType: result: {op: "bool"}
		stringType: result: {op: "stringType"}
		string: result: {op: "text", field: "value"}
		"true": result: {op: "boolean", value: bool}
		"false": result: {op: "boolean", value: bool}
		lowerBound: result: {op: "lowerBound", value: {op: "field", field: "value"}}
		upperBound: result: {op: "upperBound", value: {op: "field", field: "value"}}
		lowerBoundExpression: result: {op: "lowerBound", value: {op: "evaluate", field: "operand"}}
		upperBoundExpression: result: {op: "upperBound", value: {op: "evaluate", field: "operand"}}
		negate: result: {op: "negate", value: {op: "evaluate", field: "operand"}}
		logicalNot: result: {op: "logicalNot", value: {op: "evaluate", field: "operand"}}
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
		logicalAnd: result: {op: "logicalAnd", left: #EvaluateChild, right: #EvaluateChild}
		logicalOr: result: {op: "logicalOr", left: #EvaluateChild, right: #EvaluateChild}
		less: result: {op: "less", left: #EvaluateChild, right: #EvaluateChild}
		lessEqual: result: {op: "lessEqual", left: #EvaluateChild, right: #EvaluateChild}
		greater: result: {op: "greater", left: #EvaluateChild, right: #EvaluateChild}
		greaterEqual: result: {op: "greaterEqual", left: #EvaluateChild, right: #EvaluateChild}
		equal: result: {op: "equal", left: #EvaluateChild, right: #EvaluateChild}
		notEqual: result: {op: "notEqual", left: #EvaluateChild, right: #EvaluateChild}
		unify: result: {
			op:    "unify"
			left:  #EvaluateChild
			right: #EvaluateChild
		}
	}
}
