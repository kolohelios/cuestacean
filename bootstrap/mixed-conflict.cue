package bootstrap

let Left = {kind: "recordExpression", fields: [{name: "x", value: {kind: "listExpression", elements: [{kind: "recordExpression", fields: [{name: "a", value: {kind: "integer", value: 1}}]}]}}]}
let Right = {kind: "recordExpression", fields: [{name: "x", value: {kind: "listExpression", elements: [{kind: "recordExpression", fields: [{name: "a", value: {kind: "integer", value: 2}}]}]}}]}
#Program & {entry: {kind: "unify", left: Left, right: Right}}
