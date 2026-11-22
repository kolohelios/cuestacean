package bootstrap

#InstructionBinaryRuntime: {
	let L = #InstructionOperandCases.leftOperand

	// The alternate policy overrides only sum's default right-child selection.
	let R = #InstructionRightOperand
	op: "concat", parts: [
		#InstructionBinaryTables.select,
		{op: "text", value: "("}, L, {op: "text", value: ", "}, R, {op: "text", value: ")"},
	]
}
