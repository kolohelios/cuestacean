package bootstrap

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
