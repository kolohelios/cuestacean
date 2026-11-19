# Native scalar coalescing audit

Milestone [#210](https://github.com/kolohelios/cuestacean/issues/210).

## Selection and review gate

Follow [the equality-path observations](default-equality-trace.md). Add an opt-in
native transcript of the actual scalar append loop, not a Go-fed replay or another
copy of mode transitions. The existing [batch API](native-batch-defaults.md), flat
and recursive constructors retain their exact behavior/accounting. No production
import, preferred projection, partial-vertex certificate or authorization to unhold
#175. Current equality/run-mode/projection admission remains unresolved.

Two focused contract reviews and explicit GO precede implementation. Preserve
all old tests/assertions and CUE Authors/Apache-2.0 notices. A scoped transcript
mismatch is preserved as a witness, never repaired by row/value labels, fallback,
weakened comparison or source-order changes. Aim <=1500 changed lines.

## Interface and single execution path

Kernel owner ONLY `bootstrap/spikes/batch_defaults.rs`. Add private-module public
input-independent audit output types (Debug/Eq/PartialEq; Occurrence Copy/Clone):

```rust
pub struct Occurrence { pub trial: usize, pub child: Option<usize> }
pub struct Comparison {
    pub occurrence: Occurrence, pub existing: Leaf, pub matched: bool,
}
pub struct Append {
    pub occurrence: Occurrence, pub candidate: Leaf, pub index: usize,
    pub coalesced: bool, pub comparisons: Vec<Comparison>,
}
pub struct AuditedReport { pub report: Report, pub appends: Vec<Append> }
// execute_audited(&[Trial], work_limit<=4096) -> Result<AuditedReport,Error>
```

Occurrence is an encounter position in this already-successful batch's supplied
trial vector, plus a child index only for Nested (both zero-based; Scalar None,
Nested Some(child index)). Append.occurrence is the candidate encounter;
Comparison.occurrence is the first insertion retained in the compared slot. It is NOT a source-occurrence
identity or inferred from scalar equality. Compare existing occurrences by the
retained FIRST appended leaf's position, unchanged by later Default promotion.
The caller may look up separately authored incoming/nested-origin facts using
these positions; no source history is invented inside the primitive.

Refactor into ONE shared mode/append implementation used by execute and
execute_audited. Do not duplicate/replay maximum/combine/append/normalization.
Capture each expanded leaf's candidate AFTER mode combination but BEFORE append
search and batch-wide Maybe-to-Not normalization. Snapshot each actual existing
leaf immediately before its equality comparison and before any promotion.
matched is the actual exact-i64 equality branch, not a post-hoc set comparison.
Append index is the retained destination slot when coalesced, otherwise the
pre-insertion length. Comparisons retain encounter order and stop at first match.
Record accepted and rejected comparisons, not only final results.

Optional audit state owns bounded copied snapshots plus the first-occurrence
vector parallel to results; no mutable caller input, borrowed pointers, shared
cache or output-derived input. No audit allocation/snapshot/copy/ledger overhead
when using old execute. Old reports, error categories/admission precedence and
EXACT old work counts stay unchanged. Positional bookkeeping must not identify
source causes or change ordering/equality. AuditedReport.report.work is TOTAL
charged work for the audited invocation, not the old unaudited count.

The native transcript deliberately has NO pointer/partial/final kind and NO
runMode: it does not know those facts. Never classify partial vertices from i64
values or from a prior Go observation. Both paths still abstract complete scalar
equality only; the audit is evidence of native operations, not a readiness gate.

## Bounds and failure behavior

Preserve input64 trials/children, 128 total trial+child records, 64 RAW expanded
leaves before filtering/coalescing, <=4096 shared cumulative work. Empty/unary
Nested Unsupported; >4096 allowance rejects before input admission; structural
admission otherwise precedes charged execution. Old zero-work behavior stays.

Audited mode charges, in the SAME ledger, entry/header metadata, every candidate/
occurrence snapshot, actual comparison snapshot, retained occurrence insertion,
audit event insertion and final report retention/moves. Preserve every old charge
and its order. Extra schedule, charged BEFORE covered allocations/copies/enqueues:

- Audit state/header initialization:3.
- Each candidate:3 (occurrence snapshot, leaf snapshot, comparison-vector header).
- Each comparison:3 (retained occurrence, existing leaf, comparison insertion).
- New retained first-occurrence insertion:1.
- Completed append-event insertion:1.
- Final audit retention/moves:1 + append count + comparison count.

Thus on successful calls, audited work = old work + 4 + 5*A + 4*C + U, where A is
expanded candidates, C actual comparisons and U newly inserted slots. Final
retention is a logical charge even when buffers move. Independently pinned anchors:
empty old2/audit6; one scalar old8/audit18; 64 identical scalar trials old323/audit900;
64 distinct scalar trials old2402/audit Limit4096. No quadratic preallocation.
No per-event/child budget reset, second run or unmetered replay within the API. Per-vector
widths remain <=64; retained audit records are bounded by shared work, not a raised
independent guard. Failure exposes no partial report/input mutation and recovers
on a fresh call. Bounds are logical, not complete allocator/stack/time hardening.

Audited calls may hit Limit earlier than execute because of extra evidence costs.
Preserve that coverage cost: 64 raw duplicate leaves can exercise the structural
boundary cheaply; do NOT raise work to make 64 unique leaves' quadratic transcript
fit. Include an independently authored case unaudited accepts and audited limits.
All-success reports must retain identical drops/non-maybe/results across both
paths (except work); old execute must not pay for new snapshots.

## Ownership and independent evidence

Second worker is read-only contract/implementation/evidence reviewer, not an
editor. Coordinator owns NEW `bootstrap/executor/tests/native_coalescing_audit.rs`,
Nix one no-default target, docs and small additive meaningful audit API exercises
in the FOUR old integration targets importing batch_defaults: batch_defaults,
flat_default_sources, recursive_default_scheduling, default_equality_trace.
Do not widen visibility/suppress dead-code lints/delete old assertions. At most
two stable-main isolated workers, immutable handoffs; no saved amendments/rebases
onto mutable coordinator parents. Retire created workers before PR.

Author trial tuples, complete expected native audit transcripts AND paired raw
CUE independently. For recursive cases construct the authored IR and its batches
natively BEFORE invoking Go; audit those natively constructed batches, never
parse trace/JSON into trials or feed captured output to either native entrypoint.
Use the unchanged v4 provider as independent observation only. The source
constructor itself remains unaudited: separately auditing its constructed trials
is a second native evidence invocation, NOT a new one-pass audited source API.
Charge constructor plus all audit invocations cumulatively within4096 in those
comparison tests; do not reset allowance per batch or claim unmetered replay.

For comparison to Go, resolve candidate/retained original modes from the native
Occurrence indices and native constructor's separately retained nested origins.
Compare candidate value/mode/originalMode, destination index, insert/coalesced,
every compared existing value/mode/originalMode and acceptance/order. This is a
SCALAR transcript projection: upstream partial/final/identity kinds and runMode
remain unmodeled. Keep all old v4 full-kind hand expectations unchanged. A finite
matching projected transcript is not a partial-vertex equality certificate.

Separate COMPLETE hand-authored native transcript controls from additional
native-v4 differential controls; never count computed comparisons as independent
hand goldens. Preserve all concrete genuine failures/pinned asymmetries already
covered by the old tests. Explicit source-reachable partial-coalescing control,
ordered duplicate promotion/first origin, nested normalization before/after,
returned versus predecessor modes, empty/failed batches, i64s, deterministic calls,
input immutability, exact/one-below work, admission/raw64/65 and recovery required.
Do not pretend every synthetic primitive tuple is source-reachable.

Mandatory absolute/readable/executable providers, fresh processes, cleared env,
unusable PATH and owned scratch HOME. Setup failure is not semantic evidence or
a fallback opportunity. Public API/CLI facts remain separate from native audit;
no default flag inferred from comparison metadata. No new Go modes/hooks.

## Construction and final validation

Native production/export sources/policies, generated Rust and 249 JSON outputs
stay unchanged. Six original constructors AND current trace constructor, four
policy hashes/exact8/6 cuts remain unchanged. Check recursive derivations AND
full outputs (`--requisites --include-outputs`): both oracle binaries/derivations,
downstream executor and provider env entries excluded from native/seed graphs.
No Go-CUE-free/minimal-source seed attestation claim.

Retain all125 research groups plus new audit groups in four profiles/strict
focused Clippy; both full Rust suites/default all-target Clippy debug/release,
pinned preflight, TEN actual checks and SEVEN packages build AND rebuild. Keep
six fresh artifact-free compiler binaries, feature isolation, deterministic drift
and independent source/value evidence. Linux x86_64 only; CUE
6d609d768f1686f9a3a2a20197cacdbb70e5c79d v0.16.1 and Nixpkgs
ac588ef1c914865dad517b337ea0eb9694a4021c. Immutable cross-reviews, exact-head
protected rebase auto-merge, confirmed required CI; #175 stays held.

## Completed bounded result

Six new audit groups include SIX complete independent hand-native transcripts,
81 separate mode-combination parity/formula controls, eight source controls and
16 ordered supplemental source comparisons. The latter 24 compare the SCALAR
projection of v4, not complete upstream kind/runMode metadata or independent hand
goldens. Original-mode lookups use native encounter indices only. Constructor and
all separate audits consume one aggregate 4096 evidence allowance before invoking
Go; no captured observations construct native inputs. All old full-kind assertions
remain, plus four meaningful additive API exercises without lint escapes.

The actual shared append loop emits candidate/existing snapshots before promotion
and normalization, retaining first insertion positions across promotion. Audited
and unaudited values/flags agree; old accounting is unchanged. Exact anchors are
empty2/6, scalar8/18, duplicate64:323/900. Distinct64 still succeeds unaudited at2402
but hits audited Limit4096: evidence has an explicit cost, not a raised guard.
No source constructor was converted into a one-pass audited API. No partial-kind,
run-mode, source-identity or default/readiness inference is introduced.

Both focused contract reviews settled the exact surcharge before GO. Immutable
kernel `b83b5797d51dbcfbafd65a0807651759786d89fe` (parent67a44d0c) and evidence
`9bacd250f968cdc05ff73699c9b8f3ffed097d8a` passed read-only cross-reviews.
Kernel was frozen before coordinator Go comparisons; no scoped transcript mismatch
was observed. Removing the 53 added lines from each old target restores its entire
old contents. Created workers/workspaces retired before PR.

All 135 research groups passed four profiles/strict focused Clippy; both full Rust
suites/default all-target Clippy passed debug/release. Formatting, pinned preflight
13/13 (seven inapplicable skips), TEN actual sandbox checks and SEVEN packages
passed build AND rebuild on x86_64-linux. All seven constructor identities
(including unchanged trace), four policy hashes, exact8/6 cuts, 249 JSON exports,
generated Rust, source constructors and Go oracle sources remain unchanged.
Fresh recursive inventories: native verifiers each 1407 derivations/3392 full build
paths; seed 1436/3466. Both oracle binaries/derivations, downstream executor and
provider env entries remain excluded. No Go-free/minimal-source seed claim follows.

Remaining gap: reviewed contextual scheduling/equality/projection admission,
including partial-vertex/task effects and a production demand interface. This
native operation transcript is not that certificate. #175 remains held.
