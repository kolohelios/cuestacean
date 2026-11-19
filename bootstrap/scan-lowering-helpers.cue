package bootstrap

#ScanStep: {op: "text", value: "cursor.step()?;\n"}
#ScanThen: {op: "get", name: "then"}
#ScanElse: {op: "get", name: "else"}
#ScanEmpty: {op: "text", value: ""}
#ScanNot: {op: "ifTrue", field: "inverse", then: {op: "text", value: "!"}, else: #ScanEmpty}
