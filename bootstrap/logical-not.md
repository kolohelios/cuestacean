# CUE-owned unary logical not

`logical-not.cue` checks a concrete boolean and selects its inverted value in
finite CUE control. Other scalars, abstract types and bounds report `invalid
logical operands`; child errors precede that check. JSON AST `logicalNot` has
one `operand`; its rule instruction has one `value`. Source `!` recurses at
minimum 101, like expression negation, above all configurable binary levels.
`!!false` is false; `!true == false` is true. No coercion or abstract propagation.

## Real unary control

The existing compiler gains a validated `logicalNot` entry with a one-Value Rust
signature. Native arity facts feed CUE header layout. Unary programs allow only
boolean results, left-input selection, logical/arithmetic failure, checked integer
negation, integer value literals and conditional branches
whose kind/boolean predicates inspect the left input (including `and`/`or`).
Every branch is checked, even unused ones. Right references, same-pair tests,
endpoint conditions and binary arithmetic/concatenation fail validation.
[Bound control](bound-control.md) now permits bounds construction with left-only
endpoint projection/literals/empty/choose; right endpoints and locals still reject. The
[integer-control extension](integer-control.md) adds `negate` unary arity alongside
`logicalNot`; [output control](output-control.md) adds two unary admission
entries and six input-independent output/incomplete failures. Metadata names do
not enforce operator semantics. Binary and unary
reference entry points reject the wrong program arity. No dummy right value,
new compiler, callbacks or native logical-not policy is introduced.

The bridge supplies root `unary` metadata and projects unary logical failure to
serialization-only `unaryFail`; original control wire still uses `fail` with
`logical`. CUE emits that failure without referencing a nonexistent right input.
`interpret_unary` executes the same validated subset on a single value. Production
`runtime::logical_not` delegates the generated unary policy under `semantics`.
Existing binary controls and goldens remain compatible through false arity facts.

## Source boundary

CUE `not-prefix` scanning recognizes `!` but not the `!=` infix prefix. Primary
control explicitly scans/selects/commits/parses/builds not before negation.
Optional grammar `logicalNot` metadata configures the unary constructor; the
existing AST unary builder owns object construction. Native private stages check
progress, ordering, UTF-8 spans and completed operands. Older alternate primary
programs may skip the new scan; there is no native fallback. Both selected drivers
propagate through recursion. Signed i64 literals and minus-prefix behavior stay
unchanged. One prefix adds one AST node and one recursive level.

Six fresh seeds, build-only JSON, committed Rust drift, parser 64-node/32-level
limits, shared 4096 frame steps and existing compiler/control budgets remain.
Independent values, unary arity rejection, actual compiled alternate control,
AST/token/metadata/state/error/limit checks and pinned Go CUE probes cover the
slice. This is not full CUE support, graph ownership, general solving, bootstrap
closure or measured runtime performance.
