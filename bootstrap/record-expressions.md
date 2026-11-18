# CUE-controlled closed-record field expressions

## Contract (review before implementation)

JSON AST `recordExpression {fields:[{name:string,value:Expression},...]}` and
rule instruction `recordExpression {field}` extend closed scalar records without
changing `record {fields:[{name,value:rawScalar},...]}`. The resulting value still
obeys [closed-record ownership](../docs/closed-records.md): width64, unique
1..64-byte UTF-8 names, null/bool/i64/text or validated closed-list/record children.
[Shared mixed-value certificates](../docs/mixed-collection-values.md) enforce 32 root-inclusive
levels/4096 logical nodes during append before later callbacks. No source syntax,
references or open/abstract records; raw literal codecs remain flat.

Capacity counts input entries and rejects >64 before any field/name/child work.
Default CUE control visits entries in input order. For each entry native checks
name presence/type, byte length and duplication against already appended names
before looking up `value` or invoking the selected child evaluator. A successful
child is immediately admitted as a concrete scalar or closed collection and appended before any later entry is
inspected. Thus an earlier child/admission error wins over later malformed names;
a duplicate name wins over its own missing/erroring child. This is eager concrete
construction, not deferred field solving. The parent consumes one AST entry and
all children share the selected evaluator and EvaluationContext.

## Compiler and checked frame boundary

Reuse the existing structured control compiler and six-seed graph with a `record`
target. Parser/list/record targets validate their full original wire programs,
including unused branches, before custom backends. Record uses firstItem/lastItem,
hasItem/itemKindIs, evaluateItem/appendItem/rejectItem, nextItem/previousItem and
finishRecord. Common if/while/return and Boolean/and retain existing semantics;
finishList is list-only, finishRecord record-only, parser operations parser-only.
CUE owns traversal, concrete-child admission and all generated frame/method/source layout.
No new compiler, native source emitter or opcode policy fallback.

A separate opaque native RecordFrame holds cursor/stage, pending name/value,
checked map, completion, child callback/shared context and 4096 control steps.
It does not loop over fields on behalf of CUE. Selection/evaluation/append/movement
stages are checked; append checks capacity/name/duplicate/type/depth/nodes before consuming
the pending value or mutating the map/stage. Rejected append retains pending state.
Finish requires an exhausted cursor and exactly the input entry count, and invokes
ClosedRecord::new again. Early return never exposes pending/incomplete results.
Arbitrary trusted drivers are not proven to visit every distinct index; duplicate
checks and final opaque construction still prevent invalid records. Callbacks
are trusted, not sandboxed. Scalar byte/allocation/native-stack work is not bounded
by these counters. Existing AST/parser/compiler/scanner guards are not raised.

## Independent evidence

Keep existing parser/list generated drivers byte-identical. Compile a real reverse
record control; successful maps have equal sorted output regardless of traversal,
so independent callback logs and different first errors (abstract first versus
zero-divisor last) must demonstrate reverse execution. Test computed/extracted
scalar/mixed collection values, empty/64/65 entries, names/duplicates/error order, abstract
exclusion, selected callbacks, shared AST visit/depth limits, ownership, protocol,
charged return/4096-step exhaustion and unused-arm/target/wire rejection. Pin exact
i64-safe upstream closed-record output/lookup/length/equality. Run full Rust/strict
Clippy, preflight/five real sandbox checks, six fresh artifact-free seeds, repeated
generation/drift and feature profiles. JSON stays build-only; generated Rust stays
committed, never a seed input. No bootstrap-closure/performance claim.
