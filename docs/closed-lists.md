# Closed lists: ownership and structural boundary

## Current representation and separate profiles

`Value::List(ClosedList)` owns opaque immutable `Arc<[Value]>` storage plus
private certified shape metadata. The original flat scalar profile is extended
by the reviewed [mixed-value contract](mixed-collection-values.md): elements may
be concrete null/boolean/i64/text or already-built validated closed lists/records.
Both containers use one private ValueShape certificate across alternating kinds. Width is
at most 64, value depth 32 and logical nodes 4096, counting shared occurrences.
No mutation, unchecked construction, Deserialize bypass, abstract elements,
open structs or references. Immutable construction forms an acyclic DAG; this does
not decide future reference/identity/cycle ownership.

Container clones share storage; text/list/record selection retains its own immutable
child after parent/input drop. Content equality applies to independently
allocated equal values. Cached child shape prevents subtree traversal during
parent construction; expanded equality/encoding/drop are structurally bounded,
not text-byte/allocation/wall-time bounded. `as_slice`, `len` and `is_empty`
remain immutable inspection; no public metadata setters or larger-limit APIs.

## Raw literal payloads versus expression evaluation

Version-1 JSON AST `{kind:"list",value:[raw scalars...]}` / rule `list {field}`
**stays scalar-only**. Its codec/schema still reject nested raw arrays, objects,
fractional/out-of-i64 values and abstracts. This compatibility profile charges
one AST entry; bounded native payload decoding is representation, not hidden
AST child evaluation.

`listExpression {elements:[AST...]}` / rule `listExpression {field}` and
[source lists](../bootstrap/source-lists.md) now construct nested values through
[CUE traversal/admission](../bootstrap/list-expressions.md). Each child shares
the selected evaluator and AST context. Native append checks prospective parent
shape before later children; final construction independently validates again.
Capacity checks before elements. Constructor checks concrete type, depth, then
logical nodes per item; invalid/unfinished values never escape. Errors include:

- `closed list exceeds 64 elements`;
- `closed list element {index} requires a concrete scalar or closed collection`;
- `closed list exceeds 32 value levels`;
- `closed list exceeds 4096 logical value nodes`.

## CUE semantics and explicit native boundaries

CUE unification checks top identity, null and list isolation before numeric
identity: equal concrete lists select an input; unequal lists conflict; list
versus non-list conflicts never gain numeric permission from absent endpoints.
Equality/logical/output admission already recognizes concrete lists. Ordering
and arithmetic remain unsupported. Native content comparison and exact JSON
encoding operate only on certified values; integer-only output still rejects lists.

`listLength`/generic source `len` count direct elements. `stringLength` remains
text-only. [Indexing](../bootstrap/list-indexing.md) clones a selected value,
including a shared nested list or record, under CUE domain/range policy. Open/abstract list
constraints, structs/field selectors, concatenation and general solving remain
absent. All parser/evaluator/compiler/scanner limits remain separate and unchanged.

Same six fresh artifact-free seeds, exact pinned upstream/independent outcomes,
feature profiles and deterministic Rust drift apply. JSON is build-only; Rust
committed, never a seed dependency. Go export/native validation/cursors/counters/
representation/encoding/compiler kernels remain explicit; no full CUE/bootstrap
closure or measured performance claim.
