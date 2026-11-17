package bootstrap

#ExclusiveBoundControl & {
	function:  "lowerExclusiveBound"
	_limit:    9223372036854775807
	_relation: "less"
	_lower: {op: "offset", value: {op: "input", input: "left", endpoint: "lower"}, delta: 1}
	_upper: {op: "empty"}
}
