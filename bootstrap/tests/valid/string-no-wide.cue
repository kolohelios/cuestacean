package bootstrap

// Preserve the complete four-digit-only policy, including counted digit steps.
#StringLiteralScanner & {_wide: [{op: "reject", reason: "unsupported string escape"}]}
