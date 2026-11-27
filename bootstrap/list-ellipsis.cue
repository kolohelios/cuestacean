package bootstrap

// Probe exactly one list-slot marker, with no partial consumption.
#ScanProgram & {body: [{op: "if", condition: {
	op: "and", left: {op: "byteIn", bytes: [46]}, right: {
		op: "and", left: {op: "byteIn", bytes: [46], ahead: 1}
		right: {op: "byteIn", bytes: [46], ahead: 2}
	}
}, then: [{op: "repeat", count: 3, body: [{op: "advance"}]}, {op: "setFlag", value: true}]}]}
