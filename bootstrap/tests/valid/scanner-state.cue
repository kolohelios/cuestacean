package bootstrap

// Read both flag values, terminate a loop through state rather than advancement,
// and test EOF without restricting non-ASCII source bytes.
#ScanProgram & {
	body: [{
		op: "if"
		condition: {op: "flagIs", value: false}
		then: [{op: "setFlag", value: true}]
		else: [{op: "reject", reason: "unexpected initial flag"}]
	}, {
		op: "while"
		condition: {op: "flagIs", value: true}
		body: [{op: "setFlag", value: false}]
	}, {
		op: "if"
		condition: {op: "flagIs", value: false}
		then: []
		else: [{op: "reject", reason: "flag was not cleared"}]
	}, {
		op: "if"
		condition: {op: "hasByte"}
		then: []
		else: [{op: "reject", reason: "expected byte"}]
	}]
}
