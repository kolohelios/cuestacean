# CUE-controlled source lists

Design for the source bridge from `listExpression` (not a new value model).
Bracket lists contain zero or more expressions separated by explicit commas;
trailing commas and leading/after-comma whitespace/comments are supported.
Existing conservative newline-after-operand rules remain: no implicit comma
insertion or newline before a closing bracket. Open/ellipsis lists, declarations,
references and general constraints remain outside the profile. Nested syntax may
parse and now evaluate through the separate [bounded nested-value contract](../docs/nested-list-values.md).

Optional grammar `list: {kind, elementsField}` selects the constructor; absence
disables it with no native fallback. Fields are validated before parsing and
cannot be empty or use `kind` as the element field. Production selects
`listExpression`/`elements`. CUE bracket/tail scanners choose token acceptance,
closing-versus-comma flags and diagnostics. No native byte/token dispatch.

[Postfix indexing](list-indexing.md) subsequently adds `list[indexExpression]`
through a separate CUE expression loop; it can select a shared nested child.

The existing parser control target adds scanList/hasList/advanceList,
probeListEnd/hasListElement, parseListElement with explicit minimum,
appendListElement, advanceListTail and buildList. The primary program sequences
these in a CUE while loop at minimum 0. Private native stages check cursor
progress, element/result ownership, recursion and completion; one element parse
calls the selected expression and primary drivers at the supplied minimum.
There is no native source element loop. Native buffering/JSON-array wrapping are
representation primitives; the existing CUE unary builder chooses object fields.

Both drivers and all 64 AST nodes/32 expression levels/4096 steps per expression
frame remain shared/unchanged. The list container costs one AST node, so at most
63 single-node elements fit the default source budget. Runtime list construction
retains its separate 64-item limit and CUE evaluation/admission control, shared
AST evaluation context and immutable ClosedList representation; nested values
add separate 32-level/4096-logical-node limits.

Default `len` selects a new `length {operand}` AST/rule. CUE unary control
selects textLength (UTF-8 bytes) or listCount (elements), rejecting other values.
Existing stringLength/listLength APIs stay domain-specific; no coercion. Child
errors and concrete-list admission still precede enclosing length dispatch. Checked
native counting/encoding/constructor kernels remain explicit.

Evidence retains actual pre-list primary code byte-exact with lists disabled.
The existing compiled `primary-call-high-minimum` now also sets list-element
minimum 100, isolating each boundary in separate inputs without duplicating a
whole generated primary. Fixed trees/values/errors, metadata/stages/budgets,
selected-driver propagation and atomic CLI checks supplement exact-i64-safe
pinned Go CUE comparisons. Array comparisons reimport/marshal both outputs
through CUE, normalizing only layout/string escapes rather than jq numbers. JSON remains
build-only, Rust committed, same six artifact-free seeds and feature checks.
Source list support is not open/abstract list solving, full CUE/bootstrap closure
or measured performance evidence.
