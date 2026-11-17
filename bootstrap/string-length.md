# CUE-controlled string byte length

`stringLength` is a unary AST kind with one `operand`; its rule instruction has
one `value`. The child evaluates eagerly before CUE admits concrete text and
selects the borrowed native `text_length` primitive. Other domains, including
abstract `string`, report `length requires a concrete string`. There is no
coercion, symbolic propagation or new value domain. Function metadata supplies
unary arity, not intrinsic behavior: an alternate CUE policy may return a literal.

Length counts UTF-8 bytes, matching pinned Go CUE `len` for strings, not scalar
values/graphemes or normalized content. Empty/ASCII/combining/non-BMP/control
text is counted after decoding. Native code borrows the shared text, obtains its
byte count and checks conversion to i64. Overflow reports `string byte length
exceeds signed 64-bit integer range`; a private count-conversion boundary permits
checking extrema without allocating impossible strings. The always-available
primitive defensively rejects unsupported representations. The production
`runtime::string_length` facade is semantics-gated and delegates generated CUE
control; compiler-only seeds retain primitives but no generated semantics.

The finite control IR adds left-input `textLength` and input-independent `length`
failure, permitted in unary control with eager unused-branch validation. No dummy
RHS or native domain dispatch fallback is introduced. Existing compiler/control
limits, six artifact-free seeds, JSON build-only/Rust drift and feature isolation
remain. Native validation, byte count/conversion, representation and checked output
remain explicit boundaries, not full CUE/compiler closure or performance claims.

Independent AST/exact JSON, domain/error/arity/ownership/extrema tests, an actual
compiled alternate policy and pinned direct AST-versus-upstream comparisons
establish this value/rule/control boundary. The subsequent
[unary-call slice](unary-calls.md) connects source `len(expression)` through CUE
primary control and grammar-selected constructors, not a native builtin parser.
The subsequent [source-list bridge](source-lists.md) selects generic CUE `length`
for source `len`; `stringLength` remains strictly text-only. Structs/bytes stay absent.
