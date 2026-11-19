package bootstrap

// Go validation bridge: every original case requires its raw DATA provider.
#ScanLowering: #LoweringPolicy & {
	cases: {
		program: _
		repeat: _
		while: _
		"if": _
		advance: _
		advanceChar: _
		break: _
		setFlag: _
		reject: _
		and: _
		or: _
		byteIn: _
		charIn: _
		flagIs: _
		hasByte: _
	}
}
