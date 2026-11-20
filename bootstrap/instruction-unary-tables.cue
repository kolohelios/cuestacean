package bootstrap

// Fixed unary DATA; the full instruction policy and schema remain Go-built.
{
	let Unary = {
		lowerExclusiveBound: "lower_exclusive_bound", upperExclusiveBound: "upper_exclusive_bound"
		lowerBound:          "lower_bound", upperBound:                    "upper_bound", negate:  "negate"
		logicalNot:          "logical_not", length:                        "length", stringLength: "string_length", listLength: "list_length", recordLength: "record_length"
	}
	aliases: {for name, _ in Unary {"\(name)": "unaryRuntime"}}
	cases: {unaryRuntime: {op: "concat", parts: [
		{op: "select", field: "op", cases: {for name, method in Unary {"\(name)": {op: "text", value: "runtime::\(method)"}}}},
		{op: "text", value: "("}, {op: "try", value: {op: "group", value: {op: "lower", field: "value"}}}, {op: "text", value: ")"},
	]}}
}
