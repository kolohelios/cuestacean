# CUE-controlled expression lists

JSON AST `listExpression {elements:[Expression...]}` extends the literal-only
list boundary without changing `list {value:[raw scalars...]}`. Rule instruction
`listExpression {field}` selects the expression array. Capacity (at most 64)
checks before any element evaluation. Default CUE control evaluates elements in
source order, admits each concrete scalar or validated closed-list result, appends it and
advances. The first child error or element-admission failure stops processing;
admission of an earlier abstract value precedes errors in later children.
[Nested-value shape guards](../docs/nested-list-values.md) run on append before
later children. This is strict concrete construction, not deferred list solving.

`ClosedList` remains opaque and immutable. Nested expression lists now use
certified child metadata: width 64, value depth 32 and logical nodes 4096 weighted
by occurrences even when Arcs share. The old scalar-only raw literal codec stays
unchanged; safe immutable construction cannot create cycles. Completed results still go through its defensive
constructor. [Source list syntax](source-lists.md) now builds this AST through
CUE primary control; open/abstract lists, structs/references and reference graph
ownership remain separate designs/work.

## Reuse the structured control compiler

The existing Pratt compiler accepts `target` (`parser` by default, or `list`).
Omitted target preserves old parser wire; CUE and native decoding validate whole
family-specific bodies eagerly, including unused branches. Both families share
if/while/return and boolean/and conditions. List conditions add `hasItem` and
`itemKindIs {kind}`; list actions are firstItem/lastItem, evaluateItem, appendItem,
nextItem/previousItem, rejectItem and finishList. Parser-only actions/conditions
reject for list targets and vice versa before even a custom backend executes.
The native bridge supplies validated target metadata; CUE owns frame/header and
all method/loop/branch source emission. No new compiler/seed or native emitter.

Native `ListFrame` owns checked cursor/pending-value/buffer/completion state,
not traversal policy. Evaluation calls the selected child evaluator with the
same `EvaluationContext`. Append follows evaluation; movement follows append;
finish requires exhausted cursor and exactly the input item count collected.
No pending/unfinished result escapes on early return. This checks protocol and
shape, not a proof that arbitrary trusted controls visited each distinct index.
Native callbacks/policies are trusted hooks, not a sandbox. The opaque final
constructor still prevents invalid runtime lists even for altered controls.

Default reference AST execution interprets fresh CUE list control; production
executes its generated driver. Each list-control action/if/while-test shares a
4096-step frame budget, including the final false loop test; a return costs one.
This is separate from shared AST 4096 visits/64 active nodes. The parent list is
one AST entry and each attempted child uses checked AST entry; literal payloads
remain representation data. Existing parser/scanner/compiler bounds stay fixed.

## Evidence

An actual compiled reverse control selects last/previous and yields [2,1] where
production/reference forward control yields [1,2], proving executable iteration.
Fixed outcomes cover computed values, empty/64/65 arrays, admission/error order,
completion/protocol/fuel, target/wire/dead-branch validation, shared AST budgets,
selected evaluator propagation, ownership and exact i64-safe upstream arrays.
Same six artifact-free seeds, build-only JSON/committed Rust drift and feature
profiles apply. All old parser driver Rust stays byte-identical. Go export,
typed validation, counters/cursors, callbacks and representation remain native
boundaries; no full CUE/bootstrap closure or measured performance claim.
