# Source string profile

This continuation connects [owned JSON strings](../docs/string-values.md) to the
source parser through executable CUE scanner, primary control and AST builders.
Supported literals are single-line double-quoted raw UTF-8 with common escapes:
`\"`, `\\`, `\b`, `\f`, `\n`, `\r`, `\t`, plus four-digit `\uXXXX`
and valid UTF-16 surrogate pairs ([Unicode contract](unicode-escapes.md)), plus
eight-digit scalar `\UXXXXXXXX` ([wide contract](wide-unicode.md)).
The complete `string` atom denotes the
abstract type. Empty strings, grouping, comments around strings and scalar
unification are supported. Equality is content-based, without normalization.

Wide surrogate units, hex/octal and other escapes, interpolation, hash/raw and
multiline literals remain outside this deliberately small profile. Raw ASCII
control bytes and embedded BOM reject; raw ordinary Unicode is accepted.
[CUE addition control](sum.md) now concatenates two concrete strings with `+`;
all other arithmetic still requires concrete integers. Mixed and abstract operands
reject without coercion or symbolic propagation. This is not full CUE string syntax
or support.

## CUE control and native boundary

`string-literal{,-fragments}.cue` chooses recognition, character advancement, escape whitelist,
closing detection and lexical diagnostics. It returns an unchanged span/false flag
for a nonquote start, or a completed span/true flag. It never splits UTF-8. Unknown
non-Unicode or truncated common escapes reject at the byte following the
backslash. Unicode digit/pair diagnostics follow the linked contract; raw unsupported
characters reject at their current byte; EOF before closure rejects at EOF.

The primary program scans strings after bounds and before identifiers. A matching
candidate is selected by CUE via `hasString`/`parseString`. Native frame state
checks require the proper stage, a completed in-bounds character-boundary span,
a valid optional grammar `text` constructor and forward progress. Native checked
string conversion safely bridges admitted wide scalars into JSON spelling, then
decodes the complete literal through the checked JSON decoder;
it is not the lexical policy or a general CUE decoder. The CUE `ast-text` builder
constructs the JSON object; native parser code does not assemble maps.

Grammar metadata can change kind/payload field or omit `text` to disable literal
construction. Default metadata maps text to `kind: string, value` and registers
`string` as `stringType`. Existing alternate primary policies may deliberately
omit the new operations; there is no native primary fallback.

## Invariants and proof

Source parsing retains 64 AST nodes, 32 expression levels, shared 4096 frame steps;
the scanner retains its input-length fuel and existing DSL bounds. A literal is
one AST node. Both selected expression/primary drivers propagate recursively.
Decoded values are owned and flow through the existing JSON string evaluator and
CUE scalar solver. Source and JSON program APIs emit exact string scalar values.

Independent tests pin ASTs, lexical offsets, escaped/Unicode/control data, optional
constructor metadata, state-order guards, default/interpreted control, scalar
incompleteness/conflicts, no-partial-output and original numeric/boolean behavior.
Pinned Go CUE comparisons normalize JSON spelling only for strings, never round
numeric outputs. Six fresh seeds regenerate scanners/builders/control Rust without
committed artifact inputs; Go export, native validation/conversion/kernels remain
bootstrap boundaries, not full self-hosting or measured performance claims.
