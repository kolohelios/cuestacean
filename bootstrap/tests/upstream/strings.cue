// Go CUE scalar probes, not admitted Cuestacean source-parser fixtures.
package scalarstrings

concrete: string & "hello, 世界 🦀"
empty:    "" & string
equal:    "λ" & "λ"
abstract: string
