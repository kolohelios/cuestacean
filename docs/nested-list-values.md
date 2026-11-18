# Bounded nested expression-list values

> This records the homogeneous-list introduction. The current
> [mixed collection contract](mixed-collection-values.md) supersedes cross-kind
> exclusions and uses one shared certificate across lists and records; all
> numeric bounds and immutable ownership guarantees below remain.

## Decision and ownership

Extend ClosedList's immutable value representation to contain concrete
null/bool/i64/text **or another validated ClosedList**, not abstract values,
structs or references. Source nested brackets and JSON listExpression already
have recursive ASTs; CUE list control now explicitly admits list-valued children.
The old raw `list {value:[scalars...]}` literal codec/schema stays flat in this
slice. No parser syntax/control or general list-constraint solver is added.

Keep opaque private Arc<[Value]> storage and cache logical value depth/node
count alongside it. No mutation, unchecked construction, Deserialize, public
fields or limit setters. A parent can retain only already-built immutable child
lists: safe construction cannot insert a parent into its own descendants, so
sharing forms an acyclic DAG, not strong Arc cycles. Cloning shares child storage
and preserves certified metadata. Future references/mutable terms still require
a distinct arena/identity/cycle design; this is not that graph model.

## Separate representation limits

- Width remains at most 64 direct elements, checked before elements.
- Maximum 32 value levels, root included. Scalar/empty-list depth is 1;
  nonempty-list depth is 1 + max(child depth).
- Maximum 4096 logical value nodes, including scalar/list roots, weighted by
  child occurrences, **not unique Arc allocations**: nodes = 1 + sum(child nodes).
  Repeating a shared child 64 times charges it 64 times. Cached child metadata
  avoids walking subtrees while constructing a parent and prevents exponential
  expanded output/equality work hidden behind cheap sharing.
- Constructor validates concrete child type then prospective depth before node
  count for each element in order. Frame append enforces those same prospective
  parent limits before later children run; final constructor validates again.
  No unfinished/over-budget value escapes. Error variants ListDepth/ListNodes:
  `closed list exceeds 32 value levels`,
  `closed list exceeds 4096 logical value nodes`.
- ListElement now says `closed list element {index} requires a concrete scalar or closed list`.
  Native representation defense does not replace CUE admission. Abstract values
  still reject per item before later child errors. Trusted policies/hooks are
  not sandboxed; checked append/final constructor retain the value invariant.

These add value-shape bounds, not larger parser/AST/compiler limits. Source still
has 64 AST nodes/32 recursive parse levels, evaluation 4096 visits/64 active
nodes, and list-control 4096 steps per frame. A nested value can hit its shape limit before
those independent budgets. Metadata bounds structural traversal/drop depth, not
text bytes, serde parsing, allocations, wall time or arbitrary Rust recursion.

## Semantics and evidence

Existing CUE concrete-list equality/unification/length/indexing/output policies
apply; indexing clones a selected list Arc without retaining its parent.
Element evaluation/traversal remains CUE-owned; only metadata aggregation and
representation encoding stay native. Flat raw literal decoding remains unchanged.
Ordering/arithmetic/open/abstract lists/structs/field selectors/general solving
are not added. No code uses committed Rust as a seed dependency.

Require independent 32/33 depth and 4096/4097 logical-node boundaries, repeated-DAG
amplification, width-first/type/error order, ownership/parent drop, constant-time
metadata reuse, nested output/equality/conflict/length/chained indices, actual
compiled scalar-only historical admission versus new CUE control, shared AST and
frame budgets, exact pinned CUE arrays and all six artifact-free seeds/profiles/
drift/sandbox checks. No full CUE/bootstrap closure or performance claim.
