package bootstrap

// Structurally valid but non-progressing; execution must exhaust its budget.
#ScanProgram & {
	body: [{op: "while", condition: {op: "byteIn", bytes: [32]}, body: []}]
}
