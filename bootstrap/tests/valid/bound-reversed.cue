package bootstrap

// Change only CUE endpoint placement for the same threshold.
#BoundControl & {
	function: "lowerBound"
	_lower: {op: "empty"}
	_upper: {op: "input", input: "left", endpoint: "lower"}
}
