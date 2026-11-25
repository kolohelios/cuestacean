package bootstrap

import "list"

// Experimental syntax and rule data; these definitions do not evaluate trees.
#Expression: #OptionalSliceExpression | #ConjunctionExpression | #ListSliceExpression | #SelectorExpression | #RecordExpression | #RecordLiteral | #ListExpression | #ListLiteral | #NullLiteral | #TopType | #BinaryExpression | #IntegerLiteral | #IntegerType | #BooleanAtom | #StringLiteral | #StringType | #LowerBound | #UpperBound | #UnaryExpression

// Factor the identical recursive shape; separate recursive disjuncts expand
// combinatorially in the upstream exporter as binary node kinds accumulate.
#BinaryExpression: {
	kind:  "index" | "recordIndex" | "listIndex" | "add" | "subtract" | "multiply" | "divide" | "modulo" | "quotient" | "remainder" | "unify" | "equal" | "notEqual" | "less" | "lessEqual" | "greater" | "greaterEqual" | "logicalAnd" | "logicalOr"
	left:  #Expression
	right: #Expression
}

#OptionalSliceExpression: {kind: "slice", operand: #Expression, lower?: #Expression, upper?: #Expression}
#ConjunctionExpression: {kind: "conjunction", operands: [...#Expression] & list.MaxItems(64)}
#ListSliceExpression: {kind: "listSlice", operand: #Expression, lower: #Expression, upper: #Expression}
#SelectorExpression: {kind: "select", operand: #Expression, label: string}

#RecordEntry: {name: string & !="", value: null | bool | int | string, _bytes: len(name) & <=64}
#RecordLiteral: {
	kind: "record"
	fields: [...#RecordEntry] & list.MaxItems(64)
	_unique: true & list.UniqueItems([for f in fields {f.name}])
}

#RecordExpression: {
	kind: "recordExpression"
	fields: [...{name: string & !="", value: #Expression, _bytes: len(name) & <=64}] & list.MaxItems(64)
	_unique: true & list.UniqueItems([for f in fields {f.name}])
}

#ListLiteral: {kind: "list", value: [...(null | bool | int | string)] & list.MaxItems(64)}
#ListExpression: {kind: "listExpression", elements: [...#Expression] & list.MaxItems(64)}

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
	kind:  "lowerBound" | "lowerExclusiveBound"
	value: int
}

#UpperBound: {
	kind:  "upperBound" | "upperExclusiveBound"
	value: int
}

#UnaryExpression: {kind: "recordLength" | "length" | "listLength" | "stringLength" | "negate" | "logicalNot" | "lowerBoundExpression" | "upperBoundExpression" | "lowerExclusiveBoundExpression" | "upperExclusiveBoundExpression", operand: #Expression}
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
#UnifyEndpoint: {op: "offset", value: #UnifyEndpoint, delta: -1 | 1} | {op: "listCount"} | #ControlInteger | {op: "input", input: #UnifySide, endpoint: #UnifyEndpointName} | {op: "local", endpoint: #UnifyEndpointName} | {op: "empty"} | {
	op: "choose", condition: #UnifyCondition, then: #UnifyEndpoint, else: #UnifyEndpoint
}
#UnifyRelation: "equal" | "less" | "greater" | "lessEqual" | "greaterEqual"
#UnifyKind:     "null" | "top" | "integer" | "int" | "bounds" | "boolean" | "bool" | "text" | "string" | "list" | "record"
#UnifyCondition: {op: "recordHas"} | {op: "booleanIs", input: #UnifySide, value: bool} | {op: "textCompare", relation: #UnifyRelation} | {op: "kindIs", input: #UnifySide, kind: #UnifyKind} | {op: "same"} | {op: "hasEndpoint", value: #UnifyEndpoint} | {
	op: "compare", left: #UnifyEndpoint, right: #UnifyEndpoint, relation: #UnifyRelation
} | {op: "and" | "or", left: #UnifyCondition, right: #UnifyCondition}
#IntegerOperation: "negate" | "difference" | "product" | "divide" | "modulo" | "quotient" | "remainder"
#OutputFailure:    "incompleteInt" | "incompleteValue" | "incompleteBool" | "incompleteString" | "incompleteBounds" | "integerOutput"
#UnifyExpression: #ControlInteger | {op: "integerPrimitive", operation: #IntegerOperation} | {op: "boolean", value: bool} | {op: "integerSum" | "textConcat" | "textLength" | "listCount" | "listElement" | "recordGet" | "recordCount"} | {op: "input", input: #UnifySide} | {
	op: "if", condition: #UnifyCondition, then: #UnifyExpression, else: #UnifyExpression
} | {
	op: "letBounds", lower: #UnifyEndpoint, upper: #UnifyEndpoint, result: #UnifyExpression
} | {op: "bounds", lower: #UnifyEndpoint, upper: #UnifyEndpoint} | {op: "fail", reason: #OutputFailure | "integers" | "bounds" | "booleans" | "types" | "strings" | "arithmetic" | "comparison" | "ordering" | "logical" | "divisionByZero" | "boundThreshold" | "length" | "lists" | "listLength" | "lengthOperand" | "indexOperands" | "indexRange" | "emptyBound" | "records" | "recordLength" | "recordIndexOperands" | "recordMissing" | "collectionIndexOperands"}
#BinaryControlProgram: {formatVersion: 1, function?: "index" | "recordIndex" | "listIndex" | "unify" | "sum" | "equal" | "notEqual" | "less" | "lessEqual" | "greater" | "greaterEqual" | "logicalAnd" | "logicalOr" | "difference" | "product" | "divide" | "modulo" | "quotient" | "remainder", result: #UnifyExpression}
#UnaryControlCondition: {op: "compare", left: #UnaryControlEndpoint, right: #UnaryControlEndpoint, relation: #UnifyRelation} | {op: "kindIs", input: "left", kind: #UnifyKind} | {op: "booleanIs", input: "left", value: bool} | {op: "and" | "or", left: #UnaryControlCondition, right: #UnaryControlCondition}
#UnaryControlEndpoint: {op: "offset", value: #UnaryControlEndpoint, delta: -1 | 1} | #ControlInteger | {op: "input", input: "left", endpoint: #UnifyEndpointName} | {op: "empty"} | {
	op: "choose", condition: #UnaryControlCondition, then: #UnaryControlEndpoint, else: #UnaryControlEndpoint
}
#UnaryControlExpr: {op: "textLength" | "listCount" | "recordCount"} | {op: "bounds", lower: #UnaryControlEndpoint, upper: #UnaryControlEndpoint} | #ControlInteger | {op: "integerPrimitive", operation: "negate"} | {op: "boolean", value: bool} | {op: "input", input: "left"} | {op: "fail", reason: #OutputFailure | "logical" | "arithmetic" | "boundThreshold" | "length" | "listLength" | "lengthOperand" | "emptyBound" | "recordLength"} | {op: "if", condition: #UnaryControlCondition, then: #UnaryControlExpr, else: #UnaryControlExpr}
#UnaryControlProgram: {formatVersion: 1, function: "recordLength" | "length" | "listLength" | "stringLength" | "logicalNot" | "negate" | "jsonOutput" | "integerOutput" | "lowerBound" | "upperBound" | "lowerExclusiveBound" | "upperExclusiveBound", result: #UnaryControlExpr}

// Keep binary/unary wire sides closed; only the ternary grammar admits third.
#TernarySide: "left" | "right" | "third"
#TernaryEndpoint: #ControlInteger | {op: "listCount"} | {op: "input", input: #TernarySide, endpoint: #UnifyEndpointName} | {op: "local", endpoint: #UnifyEndpointName} | {op: "empty"} | {
	op: "offset", value: #TernaryEndpoint, delta: -1 | 1
} | {op: "choose", condition: #TernaryCondition, then: #TernaryEndpoint, else: #TernaryEndpoint}
#TernaryCondition: {op: "same" | "recordHas"} | {op: "textCompare", relation: #UnifyRelation} | {op: "kindIs", input: #TernarySide, kind: #UnifyKind} | {op: "booleanIs", input: #TernarySide, value: bool} | {
	op: "compare", left: #TernaryEndpoint, right: #TernaryEndpoint, relation: #UnifyRelation
} | {op: "hasEndpoint", value: #TernaryEndpoint} | {op: "and" | "or", left: #TernaryCondition, right: #TernaryCondition}
#TernaryControlExpr: #ControlInteger | {op: "boolean", value: bool} | {op: "integerPrimitive", operation: #IntegerOperation} | {op: "listSlice" | "integerSum" | "textConcat" | "textLength" | "listCount" | "listElement" | "recordGet" | "recordCount"} | {op: "input", input: #TernarySide} | {
	op: "fail", reason: "listSliceOperands" | "listSliceRange"
} | {op: "if", condition: #TernaryCondition, then: #TernaryControlExpr, else: #TernaryControlExpr} | {
	op: "letBounds", lower: #TernaryEndpoint, upper: #TernaryEndpoint, result: #TernaryControlExpr
} | {op: "bounds", lower: #TernaryEndpoint, upper: #TernaryEndpoint}
#TernaryControlProgram: {formatVersion: 1, function: "listSlice", result: #TernaryControlExpr}
#UnifyProgram: *#BinaryControlProgram | #UnaryControlProgram | #TernaryControlProgram

// Expression-parser control flow; native frame operations are explicit intrinsics.
#PrattNumber: {op: "minimum"} | {op: "precedence"} | {op: "tokenLength"} | {op: "integer", value: uint & <=255} | {
	op:    "add"
	left:  #PrattNumber
	right: #PrattNumber
}
#PrattCondition: {op: "fileBoundary" | "hasFileField" | "hasConjunction" | "hasSlice" | "hasPostfix" | "hasSelectorString" | "hasRecord" | "hasRecordField" | "hasIndex" | "hasList" | "hasListElement" | "hasCall" | "hasOperator" | "leftAssociative" | "groupOpen" | "hasBound" | "hasIdentifier" | "hasNegation" | "hasNot" | "hasString"} | {op: "boolean", value: bool} | {
	op:    "atLeast"
	left:  #PrattNumber
	right: #PrattNumber
} | {
	op:    "and"
	left:  #PrattCondition
	right: #PrattCondition
}
#PrattStatement: {op: "probeFile" | "startFile" | "parseFileLabel" | "parseFileColon" | "parseFileValue" | "appendFileField" | "advanceFileTail" | "buildFile" | "parseFileExpression" | "fileAfterOperand" | "conjunction" | "advanceConjunction" | "openConjunctionCall" | "openConjunctionList" | "closeConjunctionCall" | "buildConjunction" | "scanSlice" | "advanceSlice" | "closeSlice" | "scanSelector" | "advanceSelector" | "selectorWhitespace" | "scanSelectorString" | "parseSelectorString" | "parseSelectorIdentifier" | "buildSelector" | "record" | "advanceRecord" | "openRecordCall" | "recordWhitespace" | "openRecordBody" | "probeRecordEnd" | "requireRecordLabel" | "parseRecordLabel" | "recordLabelTrivia" | "recordColon" | "appendRecordField" | "advanceRecordTail" | "closeRecordCall" | "buildRecord" | "scanIndex" | "advanceIndex" | "closeIndex" | "scanList" | "advanceList" | "probeListEnd" | "appendListElement" | "advanceListTail" | "buildList" | "advanceCall" | "return" | "primary" | "afterOperand" | "afterWordOperator" | "lookupOperator" | "advanceOperator" | "parseRight" | "buildBinary" | "whitespace" | "scanGroup" | "advanceGroup" | "closeGroup" | "scanBound" | "advanceBound" | "parseBound" | "scanIdentifier" | "parseAtom" | "parseInteger" | "scanString" | "parseString" | "scanNegation" | "advanceNegation" | "buildNegation" | "scanNot" | "advanceNot" | "buildNot"} | {
	op:      "parseSliceUpper" | "parseRecordValue" | "parseIndex" | "parseListElement" | "parseGroup" | "parseNegation" | "parseNot"
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
#ListControlCondition: {op: "hasItem"} | {op: "itemKindIs", kind: #UnifyKind} | {op: "boolean", value: bool} | {op: "and", left: #ListControlCondition, right: #ListControlCondition}
#ListControlStatement: {op: "firstItem" | "lastItem" | "evaluateItem" | "appendItem" | "nextItem" | "previousItem" | "rejectItem" | "finishList" | "return"} | {
	op: "while", condition: #ListControlCondition, body: [...#ListControlStatement]
} | {
	op: "if", condition: #ListControlCondition, then: [...#ListControlStatement], else: *[] | [...#ListControlStatement]
}
#RecordControlStatement: {op: "firstItem" | "lastItem" | "evaluateItem" | "appendItem" | "nextItem" | "previousItem" | "rejectItem" | "finishRecord" | "return"} | {
	op: "while", condition: #ListControlCondition, body: [...#RecordControlStatement]
} | {
	op: "if", condition: #ListControlCondition, then: [...#RecordControlStatement], else: *[] | [...#RecordControlStatement]
}
#ConjunctionControlStatement: {op: "seedTop" | "firstItem" | "lastItem" | "evaluateItem" | "unifyItem" | "nextItem" | "previousItem" | "finishConjunction" | "return"} | {
	op: "while", condition: #ListControlCondition, body: [...#ConjunctionControlStatement]
} | {op: "if", condition: #ListControlCondition, then: [...#ConjunctionControlStatement], else: *[] | [...#ConjunctionControlStatement]}
#SliceControlCondition: {op: "hasLower" | "hasUpper"} | {op: "sliceKindIs", kind: #UnifyKind} | {op: "boolean", value: bool} | {op: "and", left: #SliceControlCondition, right: #SliceControlCondition}
#SliceControlStatement: {op: "evaluateSliceOperand" | "evaluateSliceLower" | "evaluateSliceUpper" | "defaultSliceUpper" | "rejectSliceOperand" | "finishSlice" | "return"} | {
	op: "defaultSliceLower", value: int & >=-9223372036854775808 & <=9223372036854775807
} | {op: "while", condition: #SliceControlCondition, body: [...#SliceControlStatement]} | {
	op: "if", condition: #SliceControlCondition, then: [...#SliceControlStatement], else: *[] | [...#SliceControlStatement]
}
#PrattProgram: {
	formatVersion: 1
	target:        *"parser" | "list" | "record" | "conjunction" | "slice"
	if target == "parser" {body: [...#PrattStatement]}
	if target == "list" {body: [...#ListControlStatement]}
	if target == "record" {body: [...#RecordControlStatement]}
	if target == "conjunction" {body: [...#ConjunctionControlStatement]}
	if target == "slice" {body: [...#SliceControlStatement]}
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

#UnaryCall: {
	token:        string & =~"^[a-zA-Z_][a-zA-Z0-9_]*$"
	kind:         string & !=""
	operandField: string & !="" & !="kind"
}

// Literal bounds accept signed decimal/radix i64s; optional metadata enables
// parenthesized expression thresholds through the existing unary builder.
#BoundPrefix: {
	token:      ">=" | "<=" | ">" | "<"
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
	calls: *[] | [...#UnaryCall]
	bounds: *[] | [...#BoundPrefix]
	negation?: {
		token:        "-"
		kind:         string & !=""
		operandField: string & !="" & !="kind"
	}
	logicalNot?: {token: "!", kind: string & !="", operandField: string & !="" & !="kind"}
	text?: {kind: string & !="", valueField: string & !="" & !="kind"}
	list?: {kind: string & !="", elementsField: string & !="" & !="kind"}
	conjunction?: {token: string & =~"^[a-zA-Z_][a-zA-Z0-9_]*$", kind: string & !="", operandsField: string & !="" & !="kind"}
	record?: {token: string & =~"^[a-zA-Z_][a-zA-Z0-9_]*$", kind: string & !="", fieldsField: string & !="" & !="kind"}
	slice?: {kind: string & !="", operandField: string & !="" & !="kind", lowerField: string & !="" & !="kind" & !=operandField, upperField: string & !="" & !="kind" & !=operandField & !=lowerField}
	if slice != _|_ {index: {}}
	selector?: {kind: string & !="", operandField: string & !="" & !="kind", labelField: string & !="" & !="kind" & !=operandField}
	index?: {kind: string & !="", leftField: string & !="" & !="kind", rightField: string & !="" & !="kind" & !=leftField}
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
		lowerExclusiveBound: result: {op: "lowerExclusiveBound", value: {op: "field", field: "value"}}
		upperExclusiveBound: result: {op: "upperExclusiveBound", value: {op: "field", field: "value"}}
		lowerExclusiveBoundExpression: result: {op: "lowerExclusiveBound", value: {op: "evaluate", field: "operand"}}
		upperExclusiveBoundExpression: result: {op: "upperExclusiveBound", value: {op: "evaluate", field: "operand"}}
		lowerBound: result: {op: "lowerBound", value: {op: "field", field: "value"}}
		upperBound: result: {op: "upperBound", value: {op: "field", field: "value"}}
		lowerBoundExpression: result: {op: "lowerBound", value: {op: "evaluate", field: "operand"}}
		upperBoundExpression: result: {op: "upperBound", value: {op: "evaluate", field: "operand"}}
		negate: result: {op: "negate", value: {op: "evaluate", field: "operand"}}
		logicalNot: result: {op: "logicalNot", value: {op: "evaluate", field: "operand"}}
		length: result: {op: "length", value: {op: "evaluate", field: "operand"}}
		stringLength: result: {op: "stringLength", value: {op: "evaluate", field: "operand"}}
		list: result: {op: "list", field: "value"}
		slice: result: {op: "slice", operandField: "operand", lowerField: "lower", upperField: "upper"}
		conjunction: result: {op: "conjunction", field: "operands"}
		listExpression: result: {op: "listExpression", field: "elements"}
		listLength: result: {op: "listLength", value: {op: "evaluate", field: "operand"}}
		listSlice: result: {op: "listSlice", value: {op: "evaluate", field: "operand"}, lower: {op: "evaluate", field: "lower"}, upper: {op: "evaluate", field: "upper"}}
		listIndex: result: {op: "listIndex", left: #EvaluateChild, right: #EvaluateChild}
		record: result: {op: "record", field: "fields"}
		recordExpression: result: {op: "recordExpression", field: "fields"}
		recordIndex: result: {op: "recordIndex", left: #EvaluateChild, right: #EvaluateChild}
		select: result: {op: "recordIndex", left: {op: "evaluate", field: "operand"}, right: {op: "text", field: "label"}}
		index: result: {op: "index", left: #EvaluateChild, right: #EvaluateChild}
		recordLength: result: {op: "recordLength", value: {op: "evaluate", field: "operand"}}
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
