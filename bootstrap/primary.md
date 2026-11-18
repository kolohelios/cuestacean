# CUE-hosted primary dispatch

[`primary.cue`](primary.cue), using `primary-fragments.cue`, chooses groups, negation prefixes, bounds, strings,
[source lists](source-lists.md), registered unary calls, atoms and integers using the same [control DSL](pratt.md) as the expression loop. The
existing `bootstrap-pratt-codegen` compiler emits its branches and frame calls;
there is no second compiler or native primary-dispatch fallback.

The expression program's `primary` instruction dispatches this generated
program. It is wiring between two CUE programs, not an intrinsic that implements
the selection algorithm in Rust. Production uses sequential guarded clauses with
[early return](parser-return.md) after each completed primary, instead of nested
`else` dispatch. That refactor reduced original JSON depth from 15 to 8;
source-list iteration now uses depth 10, still below the unchanged 32-level guard.
The primary program explicitly sequences:

- `whitespace`, `scanGroup`, and a `groupOpen` condition;
- `advanceGroup`, `parseGroup` with a CUE-supplied `minimum`, then `closeGroup`;
- otherwise `scanNot` and `hasNot`, then `advanceNot`, `parseNot` at CUE
  minimum 101 and `buildNot`, or negation handling;
- `scanNegation` and `hasNegation`, then `advanceNegation`,
  `parseNegation` with CUE minimum 101, and `buildNegation`, or bound handling;
- `scanBound` and `hasBound`, then `advanceBound`, whitespace and `scanGroup`;
  an enabled threshold group uses `advanceGroup`/`parseGroup`/`closeGroup`,
  otherwise `parseBound` keeps literal construction; or string handling;
- `scanString` and `hasString`, then `parseString`, or list handling;
- `scanList`/`hasList`, advance/whitespace/probeListEnd, then a CUE
  `hasListElement` loop parses one selected-driver expression at minimum 0,
  appends, scans/commits the tail and probes for a trailing-comma close;
  `buildList` uses the unary builder with a buffered JSON array, then return;
- `scanIdentifier`, then optional `hasRecord`/`record` dispatches the separate
  [closed-record source controller](source-records.md) on the same frame/fuel;
- otherwise `hasCall`, then `advanceCall` and the existing group
  scan/advance/parse/close sequence at CUE argument minimum 0, followed by return;
- otherwise `hasIdentifier`, then `parseAtom` or `parseInteger`.

The production group minimum is the CUE literal zero. It is no longer reset
inside a native group-parsing algorithm. Scanner results and grammar lookup
provide candidates; CUE chooses which primitive to invoke next. A program may
ignore a candidate (for example by choosing integer parsing for identifiers).

## Native boundary and guards

Checked frame operations still perform scanner/builder calls, grammar lookup,
i64/admitted-string conversion and recursion with the selected minimum/depth. Native primary
stages track candidates, committed spans, group results and completion. Ordinary
and bound/call-group stages are distinct: those groups retain their constructor index
and close through the existing unary AST builder. Optional per-bound expression
metadata controls admission and constructor names; old metadata/policies stay valid.
See [computed bounds](computed-bounds.md) for eager evaluation and unchanged limits.
These guards also apply after early return: they prevent illegal transitions,
primary reentry, expression mutations during
primary dispatch, and returning an unfinished group;
private state cannot be mutated by control documents. Committing a primary token
requires forward progress and a valid in-bounds UTF-8 boundary.

Primary actions use the expression frame's existing 4096-step budget, not a
fresh budget. Recursive expressions still have the existing 32-level and
64-node limits. Every nested expression propagates the selected expression,
primary and record-primary engines, including unary recursion. `parse_interpreted`
interprets all three production programs; explicit three-driver hooks expose each.
The legacy `parse_with_drivers`/`parse_interpreted_with_primary` signatures remain
compatible and select the default record-primary engine. Arbitrary Rust drivers are not sandboxed.

Unary expression negation expands the subset, but does **not** close the Go
bootstrap or make the parser/compiler fully self-hosted. Grammar validation, lookup,
conversion, primitive implementations, top-level input completion, resource
checks, interpreters and compilers remain handwritten. Go CUE still exports
specifications and JSON remains a build-only artifact.

## Evidence

`primary-nested` preserves the actual prior compiled algorithm byte-exact.
Whole-corpus compiled/reference comparisons cover old nested versus flat primary,
while fixed return fixtures test loop/if unwinding, invocation scope, fuel and
unfinished-primary rejection. The return refactor added no source syntax.
Historical whole-corpus comparisons disable subsequent call/list registration to test
the old language; [unary calls](unary-calls.md) add source `len(...)` separately.
`primary-no-records` preserves the actual pre-record primary byte-exact;
`primary-no-lists` explicitly disables records too and preserves the pre-list primary byte-exact; a compiled
`primary-call-high-minimum` now also changes element binding to 100, with fixed errors
inside nested groups/RHS/lists proving both driver propagation and no fallback.
`primary-no-calls` preserves the later flat pre-call algorithm byte-exact, and
`primary-call-high-minimum` pins CUE-selected argument binding and driver propagation.

Tests assert independent complete trees and exact diagnostics, alongside
compiled/interpreted parity over the full source corpus. Runtime grammar tests
change atom spelling, constructor kinds, bound fields and binary fields.
[Logical not](logical-not.md) adds optional constructor metadata and checked
private prefix stages. Older alternate programs may skip its scan, including
continuing from a scanned-but-ignored candidate; no native fallback is added.

Alternate CUE programs demonstrate executable policy rather than tables:
`primary-no-atoms` routes identifiers to decimal parsing and rejects `int`;
`primary-high-minimum` sets grouping's recursive minimum to 100, so `(1 + 2)`
fails at the closing-token check. Both policies propagate into nested groups
and RHS expressions. `primary-low-negation` sets unary minimum to zero,
changing `-(1)+2` from `1` to `-3` in both compiled/reference execution. The deliberately invalid `primary-cross-phase` program checks both completed
atoms and parsed-but-unclosed groups against expression operations. Other tests
cover reentry, unfinished groups, inactive primary operations, invalid stage
ordering, shared compiled/interpreted fuel, and unchanged source resource limits.
Existing exact CLI diagnostics and pinned-upstream comparisons remain in force.

`primary-bound-high-minimum` changes only the threshold group's minimum to 100,
so `>=(1+2)&3` rejects at byte 4 while ordinary groups/literal bounds are unchanged.
Fixed outcomes on compiled/reference paths and nested/RHS propagation tests prove
that CUE owns threshold recursion, not a native expression-bound algorithm.

Nix regenerates all primary drivers with the existing handwritten seed
before compiling/testing the final frontend, and checks committed Rust drift.
See [regeneration instructions](generated/README.md). Focused local test:

```sh
nix develop ./infra/tooling --command cargo test --manifest-path bootstrap/executor/Cargo.toml --test primary
```
