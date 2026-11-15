package bootstrap

// The bootstrap alphabet includes / for runtime grammar configuration.
// Recognition alone does not enable an operator or assign its meaning.
#ScanProgram & {
	body: [{
		op: "if"
		condition: {op: "byteIn", bytes: [38, 42, 43, 47]}
		then: [{op: "advance"}, {op: "setFlag", value: true}]
	}]
}
