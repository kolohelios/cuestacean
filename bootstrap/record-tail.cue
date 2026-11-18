package bootstrap

#ScanProgram & {body: [{op: "if", condition: {op: "byteIn", bytes: [44]}, then: [{op: "advance"}], else: [{op: "if", condition: {op: "byteIn", bytes: [125]}, then: [{op: "advance"}, {op: "setFlag", value: true}], else: [{op: "reject", reason: "expected comma or closing brace"}]}]}]}
