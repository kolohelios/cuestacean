package bootstrap

#UnaryControlProgram & {function: "length", result: {op: "if", condition: {op: "kindIs", input: "left", kind: "list"}, then: {op: "listCount"}, else: {op: "fail", reason: "collectionIndexOperands"}}}
