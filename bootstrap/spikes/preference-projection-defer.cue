package projectionpolicy

// Deliberate alternate: observe flat preferences but do not publish a value.
#Policy & {body: {op: "return", action: "defer"}}
