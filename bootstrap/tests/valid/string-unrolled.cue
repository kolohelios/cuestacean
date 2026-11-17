package bootstrap

// Preserve the preceding composed-but-unrolled Unicode digit control.
#StringLiteralScanner & {_unicode: #ComposedUnicodeEscape}
