package demandpolicy

// Intentional conservative alternate: executable policy may refuse all demands.
#Policy & {body: {op: "return", action: "defer"}}
