# Explicit-endpoint closed-list slicing (JSON AST)

## Contract: review before implementation

Add JSON AST `listSlice {operand:Expression,lower:Expression,upper:Expression}`,
with all three fields required. One slice node and three ordinary child ASTs;
no synthetic endpoint/container nodes, absent-endpoint defaults or duplicated
operand evaluation. Source slicing syntax remains excluded in this slice.

Evaluate operand, lower, upper exactly once in that order through the selected
evaluator and shared EvaluationContext. All child errors precede enclosing domain
or range checks; an earlier child error prevents later child entry. Field lookup
precedes attempted entry, as for existing binary operations. Preserve AST limits
4096 visits / 64 active nodes and error charging/restoration. Rule instruction
`listSlice {value:Instruction,lower:Instruction,upper:Instruction}` exposes the same
ordered semantics; CUE owns generated child-call/Result layout.

Domain: closed list plus two concrete i64 endpoints. Bounds are half-open:
0 <= lower <= upper <= length, allowing empty/end slices. No negative wrapping,
clamping, coercion, text slicing or abstract lists. Errors:
`list slicing requires a concrete closed list and integer endpoints` and
`list slice out of range`. CUE owns domain/range selection using existing integer
endpoint comparisons and list-count endpoint. The native kernel defensively
checks types/range before copying the selected bounded child handles and calling
ClosedList::new for final independent certification. The result owns immutable
storage, shares immutable completed children, and cannot reset mixed depth32 /
4096 occurrence-node certificates. The original and input may be dropped.

## Three-input executable control

Extend the existing finite unification-control compiler, not its seed count or
native emitter count, with function listSlice and input side third. Preserve
existing unary and binary signatures/renderings. A ternary generated control is
`execute(_left:Value,_right:Value,_third:Value)`; the public reference API is
`interpret_ternary(program,left,right,third)`. Existing interpret and
interpret_unary reject arity mismatch; ternary API rejects unary/binary programs.
Original-wire validation eagerly rejects third references in unary/binary
programs, including endpoints/conditions/dead arms, before custom backend use.
Unary right references and binary-only operations retain their existing rejection
rules. listSlice primitive is ternary-only; it uses left list/right lower/third
upper. Do not introduce dummy operands or permit malformed third references by
falling through binary validation. Existing per-expression scope/limits remain.

Runtime `list_slice(left,lower,upper)` is a pure generated ternary delegate;
`list_slice_elements(&left,&lower,&upper)` is the checked copy/constructor kernel.
No native range/domain policy fallback, loop emitter, general solving, mutable
storage or new representation. Existing lowering/compiler bounds remain unchanged;
factor existing CUE layouts with existing aliases if necessary and prove prior
rendering stability. Shared choose/if and two collection-index layouts keep the
unification/evaluator policies at 484/498 values under the unchanged 512 guard.
Regenerating the prior rule maps without the new slice rule reproduces all three
prior evaluator modules byte-for-byte. JSON stays build-only; committed Rust never seeds generation.

## Evidence

Independent AST/output/error oracles across reference/generated evaluators and
owned/borrowed control paths. Exact-i64 pinned upstream list slices, including
empty/end/mixed nested lists/records and Unicode. A compiled same-function
identity alternate returns the already-certified input list and must genuinely
change successful slicing behavior without bypassing child evaluation or opaque
storage invariants. Newly constructed slices always use final certification. Check all three error priorities,
selected evaluators/AST visits/depth, arity/dead-arm/duplicate-wire validation,
negative/extreme/reversed/out-of-range endpoints, value lifetimes and shape
boundaries. Existing source parser and compiled controls remain unchanged.
Full Rust, strict all-target Clippy, focused reviews, pinned preflight/five actual
x86_64-linux sandboxes, six fresh artifact-free seeds, profiles, repeated generation
and drift remain required. No full CUE/bootstrap closure or performance claim.
