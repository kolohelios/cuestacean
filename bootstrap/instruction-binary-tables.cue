package bootstrap

{
	let Binary = {
		index:       "index", difference:       "difference", product:    "product", divide:        "divide", modulo:    "modulo", quotient: "quotient"
		recordIndex: "record_index", listIndex: "list_index", logicalAnd: "logical_and", logicalOr: "logical_or", equal: "equal"
	}
	aliases: {
		for name, _ in Binary {"\(name)": "binaryRuntime"}
	}
	select: {op: "select", field: "op", cases: {for name, method in Binary {"\(name)": {op: "text", value: "runtime::\(method)"}}}}
}
