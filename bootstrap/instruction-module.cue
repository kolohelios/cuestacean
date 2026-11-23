package bootstrap

#InstructionModule: {
	module: [{
		op:    "text"
		value: #InstructionModuleText.header + "\n"
	}, #InstructionModuleFirst, {
		op: "whileRules"
		body: [{op: "kind"}, {op: "text", value: " => "}, {op: "expression"},
			{op: "text", value: ",\n"}, #InstructionModuleStep]
	}, {
		op:    "text"
		value: #InstructionModuleText.footer
	}]
}
