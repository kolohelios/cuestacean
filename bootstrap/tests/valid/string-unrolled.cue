package bootstrap

// Preserve the preceding composed-but-unrolled Unicode digit control.
#StringLiteralScanner & {_wide: [{op: "reject", reason: "unsupported string escape"}], _unicode: #ComposedUnicodeEscape}
