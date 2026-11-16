package bootstrap

// Executable recursion/Result layout, not Rust's instruction enum dispatch.
#InstructionLowering: #LoweringPolicy & {
	let Node = {op: "path", value: "node"}
	let Field = {op: "string", field: "field"}
	let L = {op: "try", value: {op: "group", value: {op: "lower", field: "left"}}}

	// The alternate policy overrides only sum's default right-child selection.
	let R = {op: "try", value: {op: "group", value: {op: "lower", field: *"right" | "left"}}}
	let V = {op: "try", value: {op: "group", value: {op: "lower", field: "value"}}}
	cases: {
		int: {op: "ok", value: {op: "path", value: "runtime::Value::Int"}}
		bool: {op: "ok", value: {op: "path", value: "runtime::Value::Bool"}}
		boolean: {op: "ok", value: {op: "call", path: "runtime::Value::Boolean", args: [{op: "boolean", field: "value"}]}}
		field: {op: "call", path: "runtime::integer", args: [Node, Field]}
		evaluate: {op: "call", path: "evaluate", args: [{op: "try", value: {op: "call", path: "runtime::field", args: [Node, Field]}}]}
		lowerBound: {op: "call", path: "runtime::lower_bound", args: [V]}
		upperBound: {op: "call", path: "runtime::upper_bound", args: [V]}
		negate: {op: "call", path: "runtime::negate", args: [V]}
		sum: {op: "call", path: "runtime::sum", args: [L, R]}
		difference: {op: "call", path: "runtime::difference", args: [L, R]}
		product: {op: "call", path: "runtime::product", args: [L, R]}
		divide: {op: "call", path: "runtime::divide", args: [L, R]}
		modulo: {op: "call", path: "runtime::modulo", args: [L, R]}
		quotient: {op: "call", path: "runtime::quotient", args: [L, R]}
		remainder: {op: "call", path: "runtime::remainder", args: [L, R]}
		unify: {op: "call", path: "runtime::unify", args: [L, R]}
	}
}
