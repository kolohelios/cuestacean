package bootstrap

close({
	let Path = close({op: string, value: string})
	let Read = close({op: string, field: string})
	let Wrapped = close({op: string, value: close({op: string, value: Read})})
	let Constant = close({op: string, value: Path})
	let Call2 = close({op: string, path: string, args: [Path, Read]})
	let Call4 = close({op: string, path: string, args: [Path, Read, Path, Path]})
	cases: close({
		"null":           Constant
		top:              Constant
		int:              Constant
		bool:             Constant
		stringType:       Constant
		text:             Call2
		list:             Call2
		record:           Call2
		recordExpression: Call4
		listExpression:   Call4
		boolean: close({op: string, value: close({op: string, path: string, args: [Read]})})
		field: Call2
		evaluate: close({op: string, path: string, args: [close({op: string, value: Call2}), Path]})
		conjunction: Call4
		slice: close({op: string, path: string, args: [Path, Read, Read, Read, Path, Path]})
		listSlice: close({op: string, parts: [Path, close({op: string, path: string, args: [Wrapped, Wrapped, Wrapped]}), Path]})
	})
})
