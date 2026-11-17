# Checked identifier styles and compiler headroom

The shared lowering DSL extends `identifier {field, snake?}` with optional
`upperFirst`. Both flags default false and cannot both be true, even in unused
branches. Identity rendering is unchanged; snake rendering retains its previous
byte-for-byte behavior. Upper-first rendering uppercases only the initial ASCII
byte and preserves every remaining byte. Thus `lessEqual` becomes `LessEqual`,
`URL` stays `URL`, and `_value` stays `_value`. It is not general PascalCase,
Unicode case folding, keyword validation or Rust name/type resolution.

The native checked kernel still admits only 1..64-byte ASCII identifiers:
initial letter/underscore, then letters/digits/underscores. It checks all input
before writing, streams transformed bytes through the existing budget and
rejects conflicting modes defensively as well as during eager policy validation.
CUE chooses the style and surrounding syntax; native code does not map semantic
enums to Rust names. Trusted raw compiler metadata/policies are not a sandbox.

The CUE meta compiler emits the new kernel argument and self-regenerates its full
implementation. Six artifact-free seeds regenerate all affected lowering drivers;
committed Rust stays a drift baseline, JSON stays build-only. Old policies remain
valid and emit the same rendered bytes; generated driver calls gain a parameter.
The CUE unification backend uses upper-first rendering for validated kind,
endpoint, relation and failure tags instead of literal spelling tables. Wire
names remain closed native enums; this does not admit arbitrary control names or
kernel paths. Existing unary failure/kernel selection tables remain explicit.

This is compiler capacity and lexical rendering work, not a source-language or
solver feature. The original backend used 503 JSON values against the unchanged
512 guard. Compaction must recover headroom without raising that or other limits.
Independent lexical/budget/error-order tests, actual compiled upper-first policy,
reference parity, all semantic goldens and complete self-regeneration establish
the boundary; compiler/solver bootstrap closure and performance remain unclaimed.
