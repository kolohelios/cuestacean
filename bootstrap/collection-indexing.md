# CUE-controlled generic collection indexing

## Bounded capability

Binary JSON AST/rule `index {left,right}` and runtime::index add one generic
operation over existing concrete values. CUE admits closed list+i64 or closed
record+text; every other pair fails CollectionIndexOperands with `indexing requires
a concrete closed list and integer or closed record and string`. The existing
listIndex/recordIndex ASTs, rules and runtime APIs stay strict and retain their
existing diagnostics. No string-byte indexing, index coercion, negative/from-end
list indices, source structs, new value ownership or general solving.

CUE owns kind-pair dispatch, nonnegative/in-range list checks and record presence/
missing-field selection. Reuse existing listElement/listCount endpoints and
recordHas/recordGet kernels, which independently defend operands. No native
Value-kind switch or generic type-policy fallback. Both children evaluate eagerly,
left first, with the selected evaluator and shared EvaluationContext; child errors
precede domain/range/presence policy. Selected children retain their own immutable
storage after parent/input drop, including across mixed collection kinds.

## Source bridge and compiler boundary

Default optional grammar index metadata changes only its constructor kind from
listIndex to index. Existing CUE postfix sequencing, checked parser stages, typed
binary AST builder, selected-driver recursion/minimum and source precedence remain
unchanged. Absent metadata still disables syntax; explicit legacy listIndex or
custom constructors remain data-selected with no fallback.
[Closed-record source expressions](source-records.md) now make both domains
source-constructible; brackets can chain through mixed list/record values. Former typed-list source domain diagnostics
intentionally become generic; range and successful source values stay unchanged.

Use the same finite binary control compiler and existing kernel/backends. New
Index function name is binary, not intrinsic semantics. The new binary failure
must reject in unary controls even in unused branches; original-wire validation
still precedes custom backends. A real compiled index-zero control (same name/
arity) proves executable policy rather than function-name dispatch. Factor the
instruction call via existing binary aliases if required by the unchanged 512-value
guard; no new emitter/compiler/seed or raised guard.

## Independent evidence and limits

Fixed outcomes: all value-domain/key pairs, missing fields (including empty and
normalization-distinct keys), negative/empty/extreme/out-of-range list indices,
nested/mixed typed-and-generic chains, strict old API rejection, left/RHS error
precedence, shared selected evaluator/4096 visits/64 active AST nodes, clone/parent
release, source exact trees/precedence/stages/budgets and legacy grammar selection.
Existing 64 source nodes/32 recursive levels/4096 control steps and mixed value
32-level/4096-node certificates remain unchanged. Preserve actual parser/scanner/
compiled typed-index driver bytes. Pin exact i64-safe upstream source/AST probes.

Full Rust/strict Clippy, preflight/five real sandbox checks, six fresh artifact-free
seeds, repeated generation/drift and feature profiles remain required. JSON stays
build-only, generated Rust committed but never a seed input. No full CUE/bootstrap
closure, comprehensive hardening or measured performance claim.
