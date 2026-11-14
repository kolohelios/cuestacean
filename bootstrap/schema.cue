package bootstrap

// Experimental syntax only; these definitions do not evaluate expressions.
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
