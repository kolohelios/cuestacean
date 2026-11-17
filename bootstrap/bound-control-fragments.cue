package bootstrap

#BoundControl: #UnaryControlProgram & {
	_lower: #UnaryControlEndpoint
	_upper: #UnaryControlEndpoint
	result: {
		op: "if", condition: {op: "kindIs", input: "left", kind: "integer"}
		then: {op: "bounds", lower: _lower, upper: _upper}
		else: {op: "fail", reason: "boundThreshold"}
	}
}
