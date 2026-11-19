# Native scalar default-batch transition primitive

## Scope and review gate

Follow [#200 lifetime tracing](default-lifetime-trace.md) directly toward held
[#175](https://github.com/kolohelios/cuestacean/issues/175). Implement a NEW private
native transition primitive for one already-successful scalar trial batch, plus
an additive origin-aware trace mode to independently check its missing inputs.
This is NOT construction of successful trials from source, a graph solver,
Preference/readiness certificate, production defaults or a corrected contextual
calculus. All old algorithms, guards, assertions and the saved prototype stay held.

Milestone [#202](https://github.com/kolohelios/cuestacean/issues/202).
Two focused contract reviews approved the scoped algorithm and clarified wire
before explicit implementation GO; read-only cross-reviews precede validation/PR. Target approximately 1500 changed lines.
Translated upstream rules retain CUE Authors/Apache-2.0 notices.

## Exact native input and operation

New test-only `bootstrap/spikes/batch_defaults.rs`, imported only by new
`bootstrap/executor/tests/batch_defaults.rs`. No production import or fallback.
Shared private-module interface (derive Debug/Eq/PartialEq; Clone for inputs):

```rust
pub enum Mode { Maybe, Default, Not }
pub struct Leaf { pub value: i64, pub mode: Mode }
pub enum Body { Scalar(i64), Nested(Vec<Leaf>) }
pub struct Trial {
    pub mode: Mode, pub left_mode: Mode, pub left_origin: Mode,
    pub incoming: Mode, pub body: Body,
}
pub struct Report {
    pub left_drops: bool, pub right_drops: bool, pub has_non_maybe: bool,
    pub results: Vec<Leaf>, pub work: usize,
}
pub enum Error { Unsupported(&'static str), Limit(&'static str) }
// execute(trials: &[Trial], work_limit: usize<=4096) -> Result<Report,Error>
```

The distinction is essential: `mode` is the successful RESULT context's current
`r.defaultMode`; `left_mode`/`left_origin` are the predecessor `p` modes inspected
by the drop predicate. Recursive `doDisjunct` can change r before that predicate;
never assume r.mode equals p.mode. `incoming` is r.origDefaultMode from the
current disjunct, not the predecessor's origin. No source occurrence identity is
inferred from equality of i64 values.

Scope is complete exact-i64 trial leaves, an INITIALLY EMPTY destination, and a
SINGLE source layer of priority zero. Exact-i64 equality is a declared scalar
coalescing abstraction, not reproduction of pointer/partial-vertex equality. The primitive does not produce/validate survivor facts or reproduce partial
vertex equality, priorities, reference cycles, abstract domains or source scheduling.
It implements this explicitly ordered pinned phase, not an order-independent CUE
semantic law or permission to imitate source-order discrepancies in production.

1. Derive leftDrops iff no successful trial's left_mode OR left_origin is Default;
   derive rightDrops iff no incoming mode is Default. Empty batch yields both true
   and an empty local result, NOT a global Conflict or Absent classification.
2. Ordering is Maybe < Default < Not. Combine(a,b,dropA,dropB) resets dropped sides
   to Maybe then takes max. For Scalar, combine(mode,incoming,leftDrops,rightDrops).
3. For each Nested child x, m=max(incoming,x.mode), then combine(mode,m,leftDrops,
   false). Track hasNonMaybe if ANY such combined nested mode is not Maybe.
4. Append each scalar result in encounter order. Equal exact i64 values coalesce;
   keep the existing mode unless the newly appended mode is Default, which promotes
   it to Default. Do not sort before this step or merge source recipe identities.
5. After the whole batch, if hasNonMaybe, convert remaining Maybe results to Not.
   Return ordered modes/facts, NOT a preferred scalar, Outcome or default set.

All decisions are generic input-state operations, never filename/value/case-label
rules. No JSON, trace output, expected answer or Go oracle is a native input.
Retain immutable caller input and fresh per-call state; errors publish no report.

## Additive trace input evidence

Keep existing `trace FILE FIELD` v1 bytes, profile and assertions EXACTLY unchanged.
Add `trace-origins FILE FIELD`: formatVersion 2, mode "trace-origins", otherwise
same envelope/batches. Each Trial adds `leftMode` and `leftOriginalMode` immediately
after originalMode, before value; all fields mandatory in v2 and absent in v1.
These are snapshots of p.defaultMode and p.origDefaultMode, not source identities.

A FOURTH count-checked diagnostic call immediately before `tmp=append(tmp,r)`
records the successful predecessor: `cuestaceanTraceOrigin(traceID,p,int(priority))`.
Existing three hooks remain at their exact stages. New helper capture start API
`CuestaceanTraceOriginsStart() error`; old start/stop and v1 wire stay stable.
Origin hooks are no-ops for v1/inactive capture. For v2, require priority,
p.priority and p.origPriority ALL zero; otherwise fail the observation, not the
upstream algorithm. Do not guess missing origin/priority facts or evaluate nodes.
Buffer copied facts PER BATCH ID (not pointers/global FIFO), matching top-of-stack
and stage; nested batches interleave. Match one buffered origin to each successful
tmp entry in order; mismatched counts or hook ordering fail closed. Charge the two
additional records when buffering, not again when attaching them to trials. Source-restoration comparison removes exactly FOUR
added calls and must reproduce pinned source byte-for-byte; no semantic lines change.

Keep 128 batches/64 per-vector/4096 retained records/65536 bytes and the finite
128-node/16-level source profile. In v2 each buffered left-mode and left-origin
snapshot costs an additional record (two per trial); total is 3*trials + nested
children + results. Stop on sticky retention failure, no truncation/partial JSON.
Old v1 exact cap tests stay intact. Add exact v2 origin-vector/record/zero-priority/
hook-matching/recovery and ordered-wire tests. Old public observer stays unchanged.

## Resources, ownership and independent tests

Native caps: 64 trials, 64 children per Nested, 128 total trial/child records and
64 raw expanded leaves BEFORE duplicate coalescing; empty/unary Nested is
Unsupported (upstream nested unroll requires >=2). Limit >4096 rejects before
input admission; otherwise structural admission precedes execution, whose entry
cost makes zero work fail. Charge scans, combination, append, equality/coalescing,
normalization and report copies in ONE shared <=4096 allowance. No branch resets,
raised limits, stack overrides or lint suppression; logical bounds are not full
allocator/stack/time hardening. Do not raise a guard to make another reachable.

Go owner ONLY existing default-trace/{adt.go,adt_test.go,main.go,main_test.go}.
Kernel owner ONLY new spikes/batch_defaults.rs. Coordinator owns new independent
executor/tests/batch_defaults.rs, Nix fourth hook/no-default target, docs,
integration/graph proofs/final validation. At most two stable merged-main workers;
immutable commits, no saved amendments or mutable coordinator-parent rebases.

Author native trial tuples and expected reports independently, paired with authored
raw CUE source. Compare Go v2 facts to those handwritten inputs and results, and
execute the NATIVE kernel on handwritten inputs only—NEVER parse a trace into
native input. Cover 3-mode combinations, original/current mismatches, nested M2/
M3 and hasNonMaybe, order-sensitive scalar coalescing/default promotion, empty
batch, i64 extrema/renamings, limits/one-below/recovery and source pairs from all
preserved genuine failures plus separately pinned asymmetries. Not every synthetic
primitive state is claimed source-reachable. Preserve all v1 flat/control bytes,
public observer/CLI comparisons and old source spellings. Any scoped mismatch is
preserved and blocks broader admission; no row-specific repairs or promotion.

## Construction and validation

No native construction source, policy, generated Rust or exported JSON changes.
Six original constructor identities/four policy hashes and exact 8/6 source cuts
stay unchanged. Trace constructor legitimately changes; all three native and seed
verifiers exclude its NEW binary/derivation. Provider stays downstream executor/
devshell, NOT base rustPackage or seed. Ten checks/seven packages build AND rebuild;
retain six fresh compilers, profiles/drift and independent source/value evidence.
Inspect recursive full derivation/build graphs (`--requisites --include-outputs`),
not runtime references. No Go-CUE-free/minimal-source compiler-seed claim.

Require new and retained 98 graph/oracle/solver groups across four profiles with
strict scoped Clippy; both full Rust suites/default all-target Clippy debug/release,
scoped offline Go tests, pinned preflight and actual sandboxes/rebuilds. Pins: CUE
6d609d768f1686f9a3a2a20197cacdbb70e5c79d (v0.16.1), Nixpkgs
ac588ef1c914865dad517b337ea0eb9694a4021c. Linux x86_64 only; #175 remains held.

## Completed evidence

Immutable Go `fd4b0b28f0c7e33fccaf13274a90fdb49e0c357e` and kernel
`52317a1f12c38a0fc8d448ec9152e1bad8469a88` both descend from merged #200 (`d5b1b369`).
Both focused reviews preceded GO; read-only cross-reviews approved those snapshots
and coordinator evidence snapshot `c36d0e58e5b013e8b172809127343a394a99e381`.
All created workspaces were retired; no frozen worker commit was amended/rebased.

Six new groups pass four profiles/strict Clippy. They check 81 synthetic flat
states, nested M2/M3 and hidden pre-coalescing non-maybe flags, ordered promotion,
raw64 caps and exact work/recovery. Ten independently handwritten source/batch
inputs match both native reports and fresh v2 facts. In the transparent-wrapper
failure, r.mode for value1 is Default while BOTH predecessor modes are Not:
actual tracing confirms why using r.mode for the left-drop predicate is wrong.
Twenty ordered public semantic observations remain separate from native reports.
No trace data becomes native input, no report selects a preferred scalar, and no
survivor/source-scheduling certificate is established.

All 98 retained groups (including unchanged v1 full byte answers), both full Rust
suites/default all-target Clippy debug/release and offline Go tests pass. Preflight
passes 13/13 (seven skips); ten actual checks/seven packages build AND rebuild.
Fresh compilers/features/drift/source/value evidence remains. Whitespace checks
1300 files. Six old constructor identities, four policy hashes, 249 JSON files,
generated Rust and exact 8/6 source cuts remain unchanged. New trace constructor
`cn7p88d0…` emits `/nix/store/518shzmdbl312bsb9n711p68r0q7gndm-cuestacean-default-trace`.
Its diff has exactly four added calls; removing them restores pinned source.

Independent queries match all three native inventories (1407 derivations/3392
full build paths each) and seed inventory (1436/3466). Current provider/consumer
exclusions and absent seed-provider environment are verified. This is not minimal
seed-source or Go-CUE-free seed attestation. #175 and its saved prototype stay held.

The [#204 flat source-batch follow-up](flat-default-sources.md) reuses this exact
primitive to construct successful trials and retain first-origin history across
flat hand-built IR groups. It does not rewrite this API, admit nested sources or
supply a production default/readiness certificate. The [#210 opt-in audit](native-coalescing-audit.md)
subsequently shares this same transition/append path while recording native
candidate/comparison snapshots. The unaudited API's reports, admission and exact
work counts remain unchanged; audit costs are additional and explicitly bounded.
