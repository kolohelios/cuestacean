package bootstrap

// Same unary name/arity; CUE instead chooses guarded upper endpoint minus one.
#ExclusiveBoundControl & {
	function:  "lowerExclusiveBound"
	_limit:    -9223372036854775808
	_relation: "greater"
	_lower: {op: "empty"}
	_upper: {op: "offset", value: {op: "input", input: "left", endpoint: "upper"}, delta: -1}
}
