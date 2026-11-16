# CUE-hosted primary dispatch

[`primary.cue`](primary.cue) chooses groups, negation prefixes, bounds, strings,
identifiers and integers using the same [control DSL](pratt.md) as the expression loop. The
existing `bootstrap-pratt-codegen` compiler emits its branches and frame calls;
there is no second compiler or native primary-dispatch fallback.

The expression program's `primary` instruction dispatches this generated
program. It is wiring between two CUE programs, not an intrinsic that implements
the selection algorithm in Rust. The primary program explicitly sequences:

- `whitespace`, `scanGroup`, and a `groupOpen` condition;
- `advanceGroup`, `parseGroup` with a CUE-supplied `minimum`, then `closeGroup`;
- otherwise `scanNegation` and `hasNegation`, then `advanceNegation`,
  `parseNegation` with CUE minimum 101, and `buildNegation`, or bound handling;
- `scanBound` and `hasBound`, then `advanceBound`, whitespace and
  `parseBound`, or string handling;
- `scanString` and `hasString`, then `parseString`, or identifier handling;
- `scanIdentifier` and `hasIdentifier`, then `parseAtom` or `parseInteger`.

The production group minimum is the CUE literal zero. It is no longer reset
inside a native group-parsing algorithm. Scanner results and grammar lookup
provide candidates; CUE chooses which primitive to invoke next. A program may
ignore a candidate (for example by choosing integer parsing for identifiers).

## Native boundary and guards

Checked frame operations still perform scanner/builder calls, grammar lookup,
i64/admitted-string conversion and recursion with the selected minimum/depth. Native primary
stages track candidates, committed spans, group results and completion. They
prevent illegal transitions, primary reentry, expression mutations during
primary dispatch, and returning an unfinished group;
private state cannot be mutated by control documents. Committing a primary token
requires forward progress and a valid in-bounds UTF-8 boundary.

Primary actions use the expression frame's existing 4096-step budget, not a
fresh budget. Recursive expressions still have the existing 32-level and
64-node limits. Every group and RHS call propagates **both** selected control
programs, including unary recursion. `parse_interpreted` interprets both production programs; the trusted
`parse_with_drivers` and `parse_interpreted_with_primary` hooks allow independent
expression/primary selection. Arbitrary Rust drivers are not sandboxed.

Unary expression negation expands the subset, but does **not** close the Go
bootstrap or make the parser/compiler fully self-hosted. Grammar validation, lookup,
conversion, primitive implementations, top-level input completion, resource
checks, interpreters and compilers remain handwritten. Go CUE still exports
specifications and JSON remains a build-only artifact.

## Evidence

Tests assert independent complete trees and exact diagnostics, alongside
compiled/interpreted parity over the full source corpus. Runtime grammar tests
change atom spelling, constructor kinds, bound fields and binary fields.

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

Nix regenerates all five primary drivers with the existing handwritten seed
before compiling/testing the final frontend, and checks committed Rust drift.
See [regeneration instructions](generated/README.md). Focused local test:

```sh
nix develop ./infra/tooling --command cargo test --manifest-path bootstrap/executor/Cargo.toml --test primary
```
