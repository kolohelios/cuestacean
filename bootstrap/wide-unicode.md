# Eight-digit source Unicode escapes

Single-line strings add `\UXXXXXXXX`, exactly eight ASCII hex digits, case
insensitive. Only Unicode scalars 0..10FFFF excluding D800..DFFF are admitted;
wide surrogate units are never paired. Existing `\uXXXX`/UTF-16 pairs and common
escapes remain unchanged. Escaped controls/BOM/noncharacters are allowed, raw
controls/BOM still reject. No interpolation, octal/hex, raw/hash/multiline syntax.

CUE checks prefix range and surrogate classification with bounded predicates,
records an invalid-scalar flag, then repeats eight hex steps. A nonhex/EOF/quote
rejects at its first byte with invalid Unicode escape. Eight valid digits outside
the scalar range reject immediately after those digits with invalid Unicode
scalar escape. Valid wide escapes leave the flag false; only closure returns true.
Admission, diagnostics and digit/range control are not native Unicode policy.

Native checked conversion runs after span/stage/CUE admission. It transcodes only
unescaped wide spellings into JSON-compatible UTF-16 escapes, then delegates the
whole quoted string to the existing JSON decoder. It defensively checks digit
count/range and char validity. Backslash pairs are consumed together, so literal
`\\U0001F980` remains literal text, not an emoji. Quotes, backslashes and control
scalars are escaped safely; malformed trusted-driver spans still fail conversion
with the existing invalid string literal diagnostic. No general CUE decoder,
normalization, exhaustive allocation handling or performance claim is introduced.

A compile-time template option preserves earlier complete scanner policies
byte-exact, including their rejection of wide escapes. The compiled no-wide
baseline accepts existing four-digit forms but rejects the same wide input that
production admits. Independent scalar/plane/edge, ownership/AST/output/operation,
malformed offset, backslash and defensive decoder tests complement pinned Go CUE
source comparisons. Same six artifact-free seeds, build-only JSON, committed Rust
drift and all scanner/parser/compiler limits remain. This is a bounded string
profile expansion, not full CUE syntax or compiler bootstrap closure.
