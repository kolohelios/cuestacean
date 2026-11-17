package bootstrap

#ExclusiveBoundControl: #UnaryControlProgram & {
	_limit:    int
	_relation: "less" | "greater"
	_lower:    #UnaryControlEndpoint
	_upper:    #UnaryControlEndpoint
	result: {
		op: "if", condition: {op: "kindIs", input: "left", kind: "integer"}
		then: {
			op: "if", condition: {op: "compare", left: {op: "input", input: "left", endpoint: "lower"}, right: {op: "integer", value: _limit}, relation: _relation}
			then: {op: "bounds", lower: _lower, upper: _upper}
			else: {op: "fail", reason: "emptyBound"}
		}
		else: {op: "fail", reason: "boundThreshold"}
	}
}
