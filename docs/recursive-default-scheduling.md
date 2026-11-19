# Recursive scalar scheduling hypothesis

## Review gate and scope

Milestone [#206](https://github.com/kolohelios/cuestacean/issues/206).
Follow [flat IR construction](flat-default-sources.md) toward held #175.
Investigate a NEW private recursive exact-scalar IR scheduler; do not change the
flat constructor, batch primitive, any old algorithm/test or production imports.
This is a falsifiable bounded operational hypothesis, not source-parser admission,
a complete CUE evaluator or a default/readiness certificate. Preserve every scoped
mismatch and hold the hypothesis instead of repairing named rows or weakening
comparison. A negative result is a valid completed slice; no production promotion.

Two focused contract reviews and explicit GO precede implementation. Reviewers
must inspect pinned processDisjunctions/doDisjunct and distinguish wrapper state,
returned dereferenced state, child history and predecessor facts. Ambiguous rules
block implementation until resolved or explicitly isolated as the hypothesis.

## Proposed private interface and ownership

New `bootstrap/spikes/recursive_default_scheduling.rs`, imported only by new
`bootstrap/executor/tests/recursive_default_scheduling.rs`. Reuse unchanged
`batch_defaults::{Mode,Body,Trial,Error,execute}`. Reuse flat State shape (new local
State type is allowed to avoid widening old APIs). No oracle/JSON input.

```rust
pub enum Expr { Value(i64), And(Vec<Expr>), Or(Vec<Alternative>) }
pub struct Alternative { pub marked: bool, pub expr: Expr }
pub struct State { pub value: i64, pub mode: Mode, pub origin: Mode }
pub struct Batch {
    pub id: usize, pub depth: usize,
    pub trials: Vec<Trial>, pub nested_origins: Vec<Vec<Mode>>,
    pub left_drops: bool, pub right_drops: bool, pub states: Vec<State>,
}
pub struct Report { pub states: Vec<State>, pub batches: Vec<Batch>, pub work: usize }
// execute(&Expr, work_limit<=4096) -> Result<Report,Error>
```

Inputs/state derive Clone/Debug/Eq/PartialEq; report/batch Debug/Eq/PartialEq.
`nested_origins` parallels trials; Scalar has an empty vector, Nested one origin
per child. IDs are reserved at batch entry. Depth is the number of active parent
batches (root batch0), NOT Expr/frame depth; sequential batches of one frame share
the same depth. These are diagnostic scheduling facts, not source identities. Report publishes no partial state on Error.

Kernel worker owns ONLY the new spike. Second worker initially performs an
independent contract/source review, then read-only kernel/evidence review (no
implementation ownership). Coordinator owns new integration tests, Nix one added
no-default target, docs, evidence and final validation. At most two isolated
stable-main workers; immutable saved handoffs, no mutable-parent rebasing/amending.
Preserve pinned CUE Authors/Apache-2.0 notices for translated rules.

## Hypothesis to test, not an established lifetime law

Pinned source: CUE 6d609d768f1686f9a3a2a20197cacdbb70e5c79d v0.16.1,
`internal/core/adt/disjunct2.go` processDisjunctions/crossProduct/doDisjunct.
Model exact i64 equality only, priority zero, no references, pending external
notifications, abstract domains, containers, imports, cycles or layers. Run modes
and partial-vertex equality are NOT modeled; a mismatch requiring them falsifies
this scope's hypothesis rather than authorizing a guessed substitute.

1. A frame starts with optional exact-value constraint, current mode and origin.
   Root seed is unconstrained Maybe/Maybe. Flatten And nodes in encounter order;
   meet all literal conjuncts before scheduling Or nodes, as the flat constructor
   does. A conflicting literal frontier still records the first scheduled empty
   Or batch and stops; literal-only conflict has no batches.
2. Each scheduled Or reserves its ID and depth BEFORE recursive attempts. Start
   from the frame seed, then from previous phase states. Enumerate predecessor-
   major alternatives. Incoming mode is Maybe for an unmarked group, otherwise
   Default/Not by mark. Attempt an alternative by recursively evaluating its Expr
   with predecessor value/current mode and incoming origin. Contradictory
   attempts contribute no parent trial, but their already-entered child batches
   remain in entry-order diagnostics. No independent eager child evaluation.
3. Distinguish successful returned node shapes. A scalar has a concrete value,
   current mode and incoming origin. A multi-result nested frame retains its root
   current mode and incoming origin plus ordered children/current/original modes.
   Frame completion forwards a singleton child's current AND original modes.
   ONLY an actual enclosing doAttempt overwrites the returned scalar's origin
   with incoming, once on successful return (the doDisjunct dereference/write).
   Do not overwrite intermediate batch snapshots or synthesize an enclosing
   attempt for the root. Wrapper conditional current-mode adoption is distinct
   from the returned child's current mode. No one-child Nested Body is built.
4. Construct parent Trial from returned current mode, TRUE predecessor current/
   original modes, incoming mode and Scalar or Nested child leaves. Thus returned
   current mode may differ from BOTH predecessor modes. Predict a multi-result
   node's direct numeric snapshot as null; do not evaluate it to discover a value.
5. Invoke unchanged batch execute with the same remaining cumulative allowance.
   For output history, preserve the first appended scalar's origin: incoming for
   Scalar, corresponding child's retained origin for Nested. Later duplicate
   Default promotion changes current mode only. Use encounter order, not a value-
   keyed source-identity interpretation. Retain pre-coalescing nested origins.
6. Stop a frame at its first empty batch. A frame with >=2 output states retains
   its wrapper current mode; a singleton forwards child current/original modes,
   pending any actual enclosing attempt's origin overwrite. Root has no enclosing
   attempt: its states retain final root-phase origins, matching the old flat
   report. Root report exposes ordered states and complete recorded batches only;
   it does NOT select defaults, report Preference/Outcome or authorize demand.

This intentionally tests whether the available state is sufficient once recursive
construction replaces supplied trial facts. It does not claim equivalence in
advance. Preserve named genuine failures AND separately pinned order asymmetries
without silently flattening nested source trees or changing spellings.

## Bounds and independent evidence

Admission: 128 total owned Expr/Alternative occurrences (each Expr and each
Alternative wrapper counts once), depth8 Expr levels (root1; each Expr child
adds1; Alternative wrapper does not add a level), 16 Or groups, width16 And/Or
vectors; empty/unary And/Or Unsupported. Each scheduled
batch bounds RAW predecessors*alternatives <=64 before filtering and delegates
old expanded-leaf/record limits unchanged. <=128 dynamic batches, <=4096 shared
work. >4096 allowance rejects before admission; structural admission otherwise
precedes charged execution. Charge traversal, frame/attempt creation, scans,
products, trial/history copies, origin searches, delegated work and reports in ONE
ledger. No recursion-stack overrides, per-child budget reset or raised guards.
These logical limits are not complete allocator/time/hostile-input hardening.

Coordinator authors recursive IR, independent raw CUE and complete hand batches/
final states BEFORE oracle invocation. Native construction executes first; no
trace-fed survivors, parsed JSON input or captured output goldens. Compare every
v3 field/order/null value/history, not merely public results. Use mandatory fresh
absolute/readable/executable providers in empty-environment processes with an
unusable PATH and owned HOME. Provider failure is setup failure, never semantics.

Cover flat compatibility against old constructor, unmarked/marked wrappers,
singleton collapse, nested conjunction, duplicate first origins, returned mode
versus predecessor, later origin consumption, failed attempts/empty batches,
renaming/order permutations and shared-budget/admission/raw-product recovery.
Exercise old private APIs meaningfully, without lint suppression/visibility escape.
All preserved genuine failures/asymmetries remain old tests; newly unsupported IR
cases are not upstream-invalid source claims. Any scoped mismatch receives a
hand-authored minimal witness and an explicit held/falsified result in this doc.

## Construction and validation gate

Use existing v3 provider unchanged: no new Go hook/mode/package. Six original
constructors AND current trace constructor, exact8/6 cuts, four policy hashes,
249 JSON exports and generated Rust must remain unchanged. Keep both providers
out of all three native construction/full build graphs and compiler seed graphs;
inspect recursive derivations AND outputs (`--requisites --include-outputs`). No
Go-CUE-free/minimal-source compiler-seed attestation claim.

Retain all112 research groups plus the new target in four profiles with strict
focused Clippy; both full Rust suites/default all-target Clippy debug/release,
pinned preflight and ten actual checks/seven packages built AND rebuilt. Retain
six fresh artifact-free compiler binaries, feature isolation, deterministic drift
and independent source/value evidence. Linux x86_64 only, pinned Nixpkgs
ac588ef1c914865dad517b337ea0eb9694a4021c. Aim <=1500 changed lines. Require
immutable cross-reviews, retired workers, exact-head protected rebase auto-merge
and confirmed required CI success. #175 remains held regardless of this result.

## Completed bounded result

Seven new groups pass. Twelve independently authored IR/raw-CUE/full hand-batch
controls distinguish singleton frame forwarding from the enclosing attempt's
one-time origin write, retain failed-child batches, preserve duplicate first
origins and cover exact i64s. Twenty-six additional ordered source comparisons
retain the concrete genuine counterexamples and separately pinned asymmetries,
including a returned Default whose predecessor current/original modes are BOTH
Not. Every observed trial, nested history, drop flag, result and dynamic entry
order/depth agrees with the frozen hypothesis on these controls. No scoped model
falsification was observed; this is not proof of equivalence over the whole IR.

Two focused reviews resolved singleton/root origin and active-batch-depth
ambiguities before explicit implementation GO. The immutable kernel is
`fc7295084e42e984a3fb0f259e44eccf52420fc1` on merged `84d3ea1d`; SHA-256
`09682817c6c5f4709a0244c9e5105c7a87ca0af2e55e91e505d08a588e3ef2de`.
It was frozen before coordinator oracle comparisons and remains byte-identical.
Both read-only cross-reviews approved evidence snapshot
`ff6ed167aaa1d8dcf725f110eedd409c2d3e2ff5`. Initial test corrections fixed an
occurrence-count arithmetic error and restored the old public observer's distinct
T:incomplete/F:incomplete expectations, not native construction or assertions
about mode transitions. No captured observations construct native inputs. Created
workers/workspaces were retired before PR.

All 119 research groups passed in four profiles with strict focused Clippy; both
full Rust suites/default all-target Clippy passed debug/release. Formatting,
pinned preflight 13/13 (seven inapplicable skips), all ten actual sandbox checks
and seven packages passed build AND rebuild on x86_64-linux. Six original
constructors AND existing trace constructor, exact8/6 source cuts, four policy
hashes, 249 JSON files and generated Rust remain unchanged. Fresh recursive native
inventories each contain 1407 derivations/3392 full build paths, seed1436/3466;
both oracle binaries/derivations, downstream executor and both provider env
entries remain excluded. This is not a Go-CUE-free/minimal-source seed claim.

The remaining gap is a separately reviewed scheduling/equality/projection
admission, including run-mode/partial-vertex effects and a production demand
interface. Raw source parsing, abstract domains, references/layers/collections and
general CUE compatibility are not supplied. Old falsified models and preserved
asymmetries remain untouched. Do not connect this hypothesis to a production
preferred-value/readiness path; #175 and its saved prototype stay held.
