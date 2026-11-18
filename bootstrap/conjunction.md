# CUE-controlled expression conjunction

## Contract: review before implementation

Add JSON AST `conjunction {operands:[Expression...]}` and rule instruction
`conjunction {field}`. This is an n-ary unification expression, not a runtime list:
abstract int/bool/string/bounds/top values may be operands without widening the
concrete closed-list/record profile. [Source and([...])](source-conjunction.md)
now lowers directly to this AST without a list wrapper. No disjunctions,
references, mutable graph, new value representation or general solving.

At most 64 operand entries; capacity checks before any child callback. The parent
costs one AST visit; each child enters the selected evaluator once, in CUE-chosen
order, sharing the existing 4096-visit/64-active context. Default CUE folds input
order incrementally: evaluate one child, unify with accumulated result, then move.
An earlier conflict stops before later malformed/erroring children; child failure
precedes that child's merge. Empty conjunction is top, incomplete alone but an
identity under enclosing unification. No synthetic list AST/extra operand visits.

## Existing compiler / checked frame

Add conjunction target to the existing structured control compiler, not a seventh
compiler/seed. CUE owns seedTop, firstItem, while hasItem {evaluateItem,unifyItem,
nextItem}, finishConjunction and all header/method/charging/layout. Reverse control
uses lastItem/previousItem; independent callback logs and differing first errors
prove it, not commutative successful results alone. Common if/while/return and
hasItem/itemKindIs/boolean/and work; appendItem/rejectItem and list/record finish
remain collection-only, parser operations parser-only, seedTop/unifyItem/
finishConjunction conjunction-only. Entire original wire, including dead arms,
validates before any backend. Canonical action metadata is still not wire syntax.

Opaque native ConjunctionFrame supplies checked stages, cursor/pending candidate,
accumulator, successful merge count, completion and 4096 control steps. seedTop is
an explicit CUE-selected primitive, not a native implicit fold identity. Evaluate
requires ready cursor, unify requires pending child, movement requires successful
merge; finish requires exhausted cursor and exactly input-count merges. Failed
unification preserves both pending value and accumulator; no partial result
escapes early return or protocol failure. Controls/callbacks are trusted hooks,
not a proof arbitrary controls visited every distinct index or a sandbox.

Native frame calls an explicit Unifier function pointer (two owned Values to
Result<Value,Error>) for one atomic merge. This permits compiler-only builds
without generated semantics or a fallback/stub. Production and interpreted AST
facades both supply runtime::unify, itself pure generated CUE policy. Native
never supplies traversal/merge-domain policy. No extra AST charge for merging
already-evaluated values. Owned immutable child handles/certificates remain
unchanged; existing scalar/collection conflict and abstract-output errors apply.

APIs: ConjunctionDriver fn(&mut ConjunctionFrame)->Result<(),Error>, Unifier
fn(Value,Value)->Result<Value,Error>; conjunction_with_driver(node,field,context,
evaluate:&mut dyn FnMut(...),unify:Unifier,driver), conjunction_interpreted(...,
evaluate,program), conjunction(...,evaluate). Latter two are semantics-gated and
pass runtime::unify. Expose equivalent default/interpreted/with-driver patterns
to existing list/record APIs. New capacity/state/budget errors are conjunction-
specific; no larger counters/stacks or lint suppression.

## Evidence

Independent full AST/results/errors for zero/one/many operands, abstract bound
refinement and scalar/closed-collection unification, top identity/incomplete
output, capacity 64/65, early conflict vs later child errors, selected callbacks,
shared AST budgets and retained child lifetimes/certificates. Private protocol,
failed-merge transactional ownership, completion/charged return/exact frame fuel.
Real compiled reverse control, wrong-target/dead-arm/internal-tag/duplicate-wire
checks. Compiled/reference empty folds cost four steps; adding a tail return costs
five, including failure after completion at one step short without publishing a
result. All prior parser/list/record generated controls stay byte-identical.
Existing evaluator instruction layouts also remain byte-identical after factoring
one logical-and binary alias; evaluator/control backends have 500/333 original
values under unchanged 512 guards.
Pin upstream and([...]) exact-i64 outcomes/conflicts without jq rounding, atomic
CLIs and full Rust/strict Clippy/focused review/pinned preflight/five real x86
sandboxes/six artifact-free seeds/profiles/repeated generation+drift. JSON stays
build-only; committed Rust never seeds generation. No full CUE/bootstrap closure,
comprehensive byte/allocation/native-stack bound or measured performance claim.
