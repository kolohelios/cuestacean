package bootstrap

#ScanProgram & {body: [{op: "if", condition: {op: "byteIn", bytes: [123]}, then: [{op: "advance"}], else: [{op: "reject", reason: "expected opening brace"}]}]}
