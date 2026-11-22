package bootstrap

#InstructionModuleText: close({header: string, footer: string})

#InstructionModuleFirst: close({op: string})
#InstructionModuleStep: close({op: string})

#InstructionBinaryTables: close({
	aliases: close({
		index:       string, difference: string, product:    string, divide:    string, modulo: string, quotient: string
		recordIndex: string, listIndex:  string, logicalAnd: string, logicalOr: string, equal:  string
	})
	select: close({
		op: string, field: string
		cases: close({
			index: close({op: string, value: string}), difference: close({op: string, value: string})
			product: close({op: string, value: string}), divide: close({op: string, value: string})
			modulo: close({op: string, value: string}), quotient: close({op: string, value: string})
			recordIndex: close({op: string, value: string}), listIndex: close({op: string, value: string})
			logicalAnd: close({op: string, value: string}), logicalOr: close({op: string, value: string})
			equal: close({op: string, value: string})
		})
	})
})

#InstructionRightField: close({field: string})
#InstructionRightOperand: close({op: string, value: close({op: string, value: close({op: string, field: string})})})

#InstructionOperandCases: close({
	let Wrapped = close({op: string, value: close({op: string, value: close({op: string, field: string})})})
	let Call = close({op: string, path: string, args: [Wrapped, Wrapped]})
	let Text = close({op: string, value: string})
	leftOperand: Wrapped
	cases: close({
		sum: Call, less: Call, lessEqual: Call, greater: Call, greaterEqual: Call, notEqual: Call, remainder: Call
		unify: close({op: string, parts: [Text, Call, Text]})
	})
})

#InstructionBinaryRuntimeShape: close({
	let Text = close({op: string, value: string})
	let Wrapped = close({op: string, value: close({op: string, value: close({op: string, field: string})})})
	op: string
	parts: [close({
		op: string, field: string
		cases: close({
			index:       Text, difference: Text, product:    Text, divide:    Text, modulo: Text, quotient: Text
			recordIndex: Text, listIndex:  Text, logicalAnd: Text, logicalOr: Text, equal:  Text
		})
	}), Text, Wrapped, Text, Wrapped, Text]
})
#InstructionBinaryRuntime: #InstructionBinaryRuntimeShape

// Executable recursion/Result layout, not Rust's instruction enum dispatch.
#InstructionLowering: #LoweringPolicy & {
	aliases: {
		lowerExclusiveBound: _, upperExclusiveBound: _, lowerBound:   _, upperBound: _, negate:       _
		logicalNot:          _, length:              _, stringLength: _, listLength: _, recordLength: _
		#InstructionBinaryTables.aliases
	}
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
	cases: {
		#InstructionOperandCases.cases
		"null":           _
		top:              _
		int:              _
		bool:             _
		stringType:       _
		text:             _
		list:             _
		record:           _
		recordExpression: _
		listExpression:   _
		boolean:          _
		field:            _
		evaluate:         _
		unaryRuntime:     _
		binaryRuntime:    #InstructionBinaryRuntime
		sum:              _
		conjunction:      _
		slice:            _
		less:             _
		lessEqual:        _
		greater:          _
		greaterEqual:     _
		notEqual:         _
		remainder:        _
		unify:            _
		listSlice:        _
	}
}
