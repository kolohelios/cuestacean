package bootstrap

#InstructionModuleText: close({header: string, footer: string})
#InstructionModuleFirst: close({op: string})
#InstructionModuleStep: close({op: string})

#InstructionModule: close({
	let Text = close({op: string, value: string})
	let Op = close({op: string})
	let Loop = close({op: string, body: [Op, Text, Op, Text, Op]})
	module: [Text, Op, Loop, Text]
})
