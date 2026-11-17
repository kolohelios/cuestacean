package bootstrap

#Grammar & {
	integer: {kind: "integer", valueField: "value"}
	bounds: [{token: ">=", kind: "lowerBound", valueField: "value", expression: {kind: "lowerBoundExpression", operandField: "kind"}}]
	operators: [{token: "&", kind: "unify", leftField: "left", rightField: "right", precedence: 5, associativity: "left"}]
}
