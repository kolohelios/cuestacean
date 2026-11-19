# Flat scalar source batches with original-mode history

## Selection and review gate

Follow [#202 native batch transitions](native-batch-defaults.md) toward held
[#175](https://github.com/kolohelios/cuestacean/issues/175). Construct successful
trials NATIVELY from a deliberately small hand-built source IR, invoke the
unchanged batch primitive and retain original modes between flat batches.
This is not a raw CUE parser, nested-default evaluator, production solver,
Preference/Outcome/readiness certificate or authorization to unhold #175.
All prior algorithms, counterexamples, trace modes and assertions remain intact.
Milestone [#204](https://github.com/kolohelios/cuestacean/issues/204).
Two focused contract reviews approved this bounded construction hypothesis before
explicit implementation GO; read-only cross-reviews precede validation/PR. Aim <=1500 changed lines.

## Flat source IR and native scheduling

New test-only `bootstrap/spikes/flat_default_sources.rs`, imported only by new
`bootstrap/executor/tests/flat_default_sources.rs`. Reuse the unchanged private
batch_defaults Mode/Trial/Body/Error/execute, not a copied transition algorithm.
Shared interface (Debug/Eq/PartialEq, Clone for inputs/state):

```rust
pub struct Alternative { pub marked: bool, pub value: i64 }
pub enum Clause { Value(i64), Or(Vec<Alternative>) }
pub struct State { pub value: i64, pub mode: Mode, pub origin: Mode }
pub struct Batch {
    pub trials: Vec<Trial>, pub left_drops: bool, pub right_drops: bool,
    pub states: Vec<State>,
}
pub struct Report { pub states: Vec<State>, pub batches: Vec<Batch>, pub work: usize }
// execute(clauses: &[Clause], work_limit: usize<=4096) -> Result<Report, Error>
```

The clause vector represents an explicitly ordered flat conjunction of exact-i64
literals and FLAT disjunctions of marked/unmarked i64 alternatives. No nested
Or/And, abstract types/top, references, structs/lists, priorities/layers or cycles
are representable. This is an IR, not newly admitted production source syntax.

1. Collect all literal conjuncts before disjunction scheduling, as pinned CUE's
   pre-existing node constraints do. Equal literals fix an ordinary candidate;
   conflicting literals yield an EMPTY predecessor frontier. If any Or is
   scheduled, record its empty successful-trial batch and then stop; with no Or,
   return empty states and no batches.
2. Start a neutral internal predecessor with Maybe current/original modes and the
   literal constraint if present; otherwise unconstrained until the first Or.
   This seed is not an exported Top/State or a preferred scalar.
3. Process Or clauses in source order. A flat group's incoming modes are Maybe
   when entirely unmarked; otherwise Default for marked alternatives, Not for
   unmarked. Preserve duplicate alternatives and order until batch processing.
4. Bound the RAW predecessor-count * alternative-count before filtering, even
   when nearly all pairs conflict. Enumerate predecessor-major pairs, retaining
   exact value matches (or accepting the initial unconstrained seed). Construct
   each successful Trial with result mode == predecessor mode (NO nested recursive
   doDisjunct here), both predecessor fields, incoming group mode and Scalar body.
5. Invoke the old native batch primitive with the SAME remaining cumulative work
   allowance. Its report selects current modes/drop flags/order. For each result,
   origin is the incoming mode of the FIRST successful trial for that value.
   Later duplicate Default promotion changes current mode ONLY, not origin.
   Never derive history from current mode or trace output. Retain per-batch facts.
6. Stop after the first empty batch, as upstream stops an empty cross-product.
   Do not skip that first scheduled batch merely because literal pre-refinement
   has already made the predecessor frontier empty.
   Consistent literal-only input returns a neutral exact State and no batches.
   Empty clause input is Unsupported. Report modes/history, NOT a preferred result or Conflict
   category. Broader source scheduling and original-mode propagation remain held.

Original modes are intentionally distinct from final public Default flags: even
when a later batch has Default mode, a forced singleton may have Default=false
in the public observation. Do not derive public flags or readiness from Report.
No row-specific names/values, fallback, cached answers or mutation of caller IR.

Pre-integration falsification: the initial proposal returned no batches for
`1&(*1|2)&2`. Pinned CUE records one empty first batch. The revised generic rule
keeps an empty frontier through the first scheduled group rather than silently
short-circuiting its diagnostic fact. Preserve this regression explicitly; it is
not a value-specific exception or a changed semantic result.

## Additive history observation

Preserve ALL v1 `trace` and v2 `trace-origins` bytes/profiles/caps. Add
`trace-history FILE FIELD`: version3/mode "trace-history", same envelope, batches
and v2 trial-left fields. Nested/final Result adds mandatory `originalMode`
immediately after mode, before value; absent in v1/v2. It snapshots the result
nodeContext.origDefaultMode directly, without evaluation/dereference. Trial's
originalMode remains INCOMING; Result's originalMode is retained result history.

New `CuestaceanTraceHistoryStart() error` enables origin AND result-history capture.
Use existing FOUR hooks at exact stages, no fifth hook or semantic source change.
Zero-layer guards apply as v2. A v3 nested/final result costs two retained records
instead of one: total 3*trials + 2*nested children + 2*results <=4096, with unchanged
128 batches/64-vector/65536-byte/profile128-node/depth16 limits. Fail sticky/no
partial JSON on overflow or invalid modes; stop detaches/recovery. V1/v2 do not
validate/allocate/charge the extra field. Exact old assertions stay unchanged.
Add positive exact/one-over v3 record and ordered-wire/recovery tests, not forged
source to exceed a guard. Both old-mode byte controls and uninstrumented semantic
comparisons remain mandatory. Source restoration must still show FOUR additions.

## Bounds, ownership and evidence

Native admission caps: 16 clauses, 16 Or groups/alternatives each, 128 IR occurrences
(clause roots + alternative leaves), 64 RAW products per batch before filtering,
4096 cumulative work. Empty/unary Or Unsupported; oversized vectors/occurrences/
products/work Limit. >4096 allowance rejects before input admission; otherwise
structural admission precedes execution. Charge entry, literal scans, products,
trial/state/origin searches and copies, delegated primitive work and report output
in one budget. No per-batch reset or raised guard. Preparation/destruction are not
comprehensive allocator/stack/time or hostile wire-input hardening.

Go owner ONLY existing four default-trace Go files. Native owner ONLY new
spikes/flat_default_sources.rs. Coordinator owns the new independent integration
target, Nix new no-default target/history smoke, docs, immutable integration,
source/value comparisons and graph/final validation. At most two stable-main
workspaces; no saved-commit amendments/rebases onto mutable coordinator parents.
Preserve Apache-2.0/upstream notices for translated scheduling/history rules.

Author source IR, raw CUE strings AND full expected states/batches independently.
Native execute takes authored IR ONLY, never JSON/oracle/trace-derived inputs.
Compare native trial construction, drop flags, current/origin modes and order with
hand expectations and fresh v3 observations separately. Include flat live/dead
marks, two/three batches, literal pre-refinement before/after groups, duplicate
Default promotion with different FIRST origins, later-batch origin consumption,
conflicts, exact i64s, raw64/65 before filtering, structural/one-below/shared-work
recovery and input immutability. Synthetic IR is not broad CUE source admission.
Nested genuine failures/asymmetries remain in old targets, not claimed solved or
silently rewritten into flat cases. Exercise the imported old primitive's full
private API with meaningful controls; no lint visibility escape/suppression.
If scoped construction/history differs, preserve the falsification and hold
promotion rather than patch rows or discard failing clauses.

## Construction and validation

Native construction sources, policies, generated Rust, 249 JSON outputs, six old
constructor identities/four policy hashes and exact 8/6 cuts stay unchanged.
Trace identity legitimately changes; three native verifiers and seed verifier
exclude its NEW binary/derivation. Providers stay downstream executor/devshell,
not base rustPackage/seed. No Go-CUE-free/minimal seed-source attestation claim.
Retain recursive derivation/full-build proofs (`--requisites --include-outputs`).

Run new target + all104 retained graph/oracle/solver groups in four profiles,
strict focused Clippy; both full Rust suites/default all-target Clippy debug/
release; pinned offline Go tests, preflight, ten actual Nix checks and seven
packages built AND rebuilt. Preserve six fresh artifact-free compilers, feature
isolation, deterministic generation/drift and independent source/value oracles.
Pins: CUE6d609d768f1686f9a3a2a20197cacdbb70e5c79d v0.16.1, Nixpkgs
ac588ef1c914865dad517b337ea0eb9694a4021c; Linux x86_64 only. #175 stays held.

## Completed bounded evidence

Eight new groups cover 16 independently authored IR/raw-CUE/full hand-batch cases
and six additional group-order permutations. Inputs execute natively before
capture; traces are comparison-only. A four-batch control consumes origins that
changed between earlier batches. Duplicate Default promotion retains different
first origins despite equal current modes. Forced scalar cases explicitly keep
public Default flags separate from native modes. Native-only occurrence/product
boundary cases do not imply admission by the distinct Go AST profile.

Both focused contract reviews, the empty-frontier revision reviews and read-only
implementation cross-reviews approved. Stable-parent immutable handoffs were Go
`b21883be36f6b0b91adaeb91208bd2ca2b4558d9`, kernel
`6596c1bd54c00a747dcaf68e53853e9604eaad20`; independent evidence snapshot
`bf3852e38e5b77322f3bab8b58b1f7615e26a561` was cross-reviewed. Sandbox formatting
caught an import-order mismatch; integration changed only that ordering and reran
validation. Created workers/workspaces were retired before PR.

All 112 research groups passed in four profiles with strict focused Clippy;
both complete Rust suites and default all-target Clippy passed debug/release.
Pinned offline Go tests, preflight 13/13 (seven inapplicable skips), all ten actual
sandbox checks and all seven packages passed build and rebuild on x86_64-linux.
Six old constructors, exact eight/six-file cuts, four policy hashes, all 249 JSON
files and generated Rust are unchanged. Fresh recursive inventories contain
1407 derivations/3392 full build paths for each native verifier and 1436/3466 for
the seed: both oracle binaries/derivations and downstream executor are absent,
as are both provider environment entries. No Go-free compiler seed claim follows.
The trace producer changed to `g28hxwa8cd6l1n8mflcrhpz49frgk1m6`; restoration
retains exactly four hook additions and zero upstream removals.

This establishes only flat exact-scalar IR construction/history under the stated
bounds. Nested genuine failures and pinned asymmetries remain unchanged; no
parser/solver/readiness or preferred-value interface consumes this module.
Production defaults (#175), broader scheduling/equality/layers/references and
collections remain held. The [#206 recursive follow-up](recursive-default-scheduling.md)
separately tests nested exact-scalar IR construction/history; it does not change
this flat API or establish run-mode/partial-vertex equivalence.
