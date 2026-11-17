# Prefixed checked integer literals

Source primaries and inclusive bound thresholds add binary `0b`, octal `0o`,
and hexadecimal `0x`/`0X`. Hex digits are ASCII and case insensitive. Optional
adjacent minus is part of the literal, including MIN in every radix; spaced or
expression negation keeps existing semantics. Uppercase B/O prefixes, legacy
leading-zero octal, underscores, plus signs and other numeric forms stay outside
the profile. Pinned Go CUE probes establish prefix case and bound support.

A new CUE integer-literal scanner selects sign, prefix, digit class/run and
missing-digit diagnostics. It returns the selected span and a sign flag, not a
converted value. No digit after a prefix rejects there with expected binary,
octal or hexadecimal integer digit. Digit runs stop at the first nonmember,
as the existing decimal scanner does; the parser still rejects trailing input.
Operator maximal munch stays literal/class dependent (hex consumes a-f), not a
new word-token rule. Decimal leading-zero and missing-digit diagnostics remain.

The decimal program is factored into shared CUE fragments and retained byte-exact
as its own compiled/reference baseline. Production integer parsing and bounds
use the new scanner. Native checked conversion recognizes the admitted prefix
and delegates signed digits to standard i64 radix parsing; it does not choose
lexical acceptance. Error offsets and signed64 overflow diagnostics remain.
No positive magnitude is forced through i64 before interpreting an adjacent
minus. AST/rule wire, Value, arithmetic, source node/depth and all scanner/compiler
limits remain unchanged; there is no arbitrary precision or widened solver.

Independent span/AST/value/extrema/error oracles, old decimal comparisons,
compiled/reference scanner and parser paths, fixed bound/arithmetic results and
pinned source comparisons establish the boundary. Same six artifact-free seeds,
build-only JSON and committed Rust drift policy apply. This is a bounded numeric
profile expansion, not full CUE syntax/compiler bootstrap closure or performance
measurement. Checked native conversion remains an explicit boundary.
