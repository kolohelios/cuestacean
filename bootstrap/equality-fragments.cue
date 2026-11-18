package bootstrap

#ConcreteSide: {
	_side: #UnifySide
	condition: {
		op: "or", left: {op: "kindIs", input: _side, kind: "integer"}
		right: {op: "or", left: {op: "kindIs", input: _side, kind: "boolean"}, right: {op: "or", left: {op: "kindIs", input: _side, kind: "text"}, right: {op: "or", left: {op: "kindIs", input: _side, kind: "null"}, right: {op: "or", left: {op: "kindIs", input: _side, kind: "list"}, right: {op: "kindIs", input: _side, kind: "record"}}}}}
	}
}

#EqualityProgram: #BinaryControlProgram & {
	_same: bool
	result: {
		op: "if"
		condition: {op: "and", left: (#ConcreteSide & {_side: "left"}).condition, right: (#ConcreteSide & {_side: "right"}).condition}
		then: {
			op: "if", condition: {op: "same"}
			then: {op: "boolean", value: _same}
			else: {op: "boolean", value: !_same}
		}
		else: {op: "fail", reason: "comparison"}
	}
}
