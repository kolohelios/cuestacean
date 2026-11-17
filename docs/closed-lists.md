# Closed scalar lists: ownership and first structural boundary

## Decision and deliberately narrow profile

Add concrete flat lists of at most 64 owned null/boolean/i64/text values. Runtime
`Value::List(ClosedList)` uses an opaque immutable `ClosedList` backed by
`Arc<[Value]>`. Its only public construction path validates capacity first, then
each element, permitting only concrete scalar variants. Storage stays private;
`as_slice`, `len` and `is_empty` expose immutable inspection. No mutation, unchecked
constructor, Deserialize bypass, nested lists, abstract elements or references.

This gives content equality and shared ownership without copying child text on
container clone. Independently allocated equal lists compare equal. Values own
their contents after input JSON is dropped. Flatness plus the 64-item cap bounds
element/depth traversal for comparison/drop/encoding, not text-byte work or
allocations; no recursive value/graph ownership decision
is smuggled in. Future reference/cycle terms need a separate graph design, not
strong Arc cycles in this representation. Nested values still need a separate
design. [Expression-list evaluation](../bootstrap/list-expressions.md) now extends
construction through explicit CUE traversal without weakening this invariant.

## Wire, kernels and executable CUE boundary

Version-1 JSON AST adds `{kind:"list", value:[raw JSON scalars...]}`. Its rule
instruction is `list {field}`. These are literal payloads, not child AST
expressions; bounded native scalar decoding/encoding is a representation kernel,
not a hidden expression-evaluation loop. The node consumes one AST visit; payload
conversion is bounded by 64 items separately. The subsequent `listExpression`
kind makes CUE own child traversal/evaluation rather than reuse this literal codec.

Capacity rejects before element validation. Errors are `closed list exceeds 64
elements`, `closed list element {index} requires a concrete scalar`, or the usual
missing field / `field {field:?} must be an array`. Nested arrays, objects,
fractional/out-of-i64 numbers and native abstract values reject. Scalars preserve
exact i64 and UTF-8 content with no normalization/coercion.

CUE unification handles top identity first, then null/list isolation before
numeric identity: equal lists select an input; unequal lists produce `conflicting
closed list values`; list/non-list conflicts are types, never numeric permission
from absent endpoints. CUE concrete-domain equality and logical selection admit
lists; ordering/arithmetic retain their existing unsupported-domain errors.
CUE JSON-output admission already permits concrete values; the native encoder
adds bounded array layout. Integer-only output still rejects lists.

Unary `listLength {operand}` / rule `listLength {value}` and CUE `list-length-control`
admit a closed list and call borrowed native `list_count`; other values fail
`length requires a concrete closed list`. An actual compiled alternate policy
returns zero independently of input. Existing `stringLength` stays text-only.
[Source lists](../bootstrap/source-lists.md) now build `listExpression`; default
`len` selects CUE text/list `length` dispatch without widening the old APIs.
Open/abstract/nested lists, selectors, structs, list concatenation and general
list solving remain absent.

## Evidence and bootstrap

Pin empty/mixed/extreme/Unicode lists, every mixed domain, equal/unequal/shape
conflicts, exact JSON, 64/65 and malformed-element boundaries, API/Arc lifetime,
left-first errors and compiled/reference policies against independent outcomes.
Pinned CUE JSON marshal produces exact compact expected arrays, including i64
extrema; never pass numeric list contents through floating-point/jq normalization.
Same six artifact-free seeds, build-only JSON/committed Rust drift, feature and
all existing source/evaluation/compiler/scanner limits remain. Native validation,
representation/codecs/kernels and Go export remain explicit boundaries. This is
not full lists/CUE, graph ownership, bootstrap closure or measured performance.
