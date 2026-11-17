package bootstrap

#ExclusiveBoundControl & {
	function:  "upperExclusiveBound"
	_limit:    -9223372036854775808
	_relation: "greater"
	_lower: {op: "empty"}
	_upper: {op: "offset", value: {op: "input", input: "left", endpoint: "upper"}, delta: -1}
}
