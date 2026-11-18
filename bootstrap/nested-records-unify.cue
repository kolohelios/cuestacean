package bootstrap

let Value = {kind: "recordExpression", fields: [{name: "x", value: {kind: "recordExpression", fields: [{name: "a", value: {kind: "integer", value: 7}}]}}]}
#Program & {entry: {kind: "unify", left: Value, right: Value}}
