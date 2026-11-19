# Default batch execution and coalescing observations

Milestone [#208](https://github.com/kolohelios/cuestacean/issues/208).

## Selection and review gate

Follow the [recursive scalar scheduling hypothesis](recursive-default-scheduling.md)
toward held #175. Observe the actual run mode and equality path used by pinned
crossProduct/appendDisjunct; matching a value/mode trace alone does not establish
that complete-i64 equality safely represents partially evaluated vertices.
This is an additive diagnostic study, not native admission, a solver change or
permission to project preferred values. All previous hypotheses/assertions stay.

Two focused contract reviews and explicit GO precede implementation. Inspect
pinned source hook locations, ordering, record accounting and no-perturbation
claims before code. Preserve scoped surprises and rejected admission hypotheses;
never reinterpret public Default flags as equality/readiness certificates.
Aim approximately <=1500 changed lines including all tests/docs.

## Additive v4 protocol

Add `trace-equality FILE FIELD`, formatVersion4/mode trace-equality, retaining the
same finite source admission and all v3 predecessor/result-history fields.
ALL v1/v2/v3 bytes, profile, limits and helper APIs remain unchanged.
New `CuestaceanTraceEqualityStart() error` enables origins/history plus equality
capture. Batch appends one mandatory `phase` object after results; absent in old
modes. Its ordered fields are `runMode,appends`. runMode is the actual crossProduct
argument: ignore/attempt-only/yield/finalize (unknown values fail diagnostics).

Append event ordered fields:

```text
{candidate: Result, index: uint, kind: "insert"|"identity"|"partial"|"final",
 comparisons: [{index:uint, kind:"identity"|"partial"|"final",
                existing:Result, accepted:bool}, ...]}
```

Result remains ordered mode/originalMode/value, with mandatory originalMode in v4.
Snapshots copy direct nodeContext modes and direct node.BaseValue exact i64 or
null, never evaluate/dereference to discover a scalar. An append candidate is
snapshotted BEFORE the destination search/promotion; existing is snapshotted at
that actual comparison. index is the current destination index, not a source ID.
Comparison encounter order is retained; accepted is true only for the actual
coalescing decision, not numeric equality or inferred subsumption. A final kind
means BOTH actual comparison vertices were finalized at the branch decision;
partial means at least one was not. It does not mean partial equality is invalid.
Identity is the actual pointer-equality early return. Insert has index equal to
the pre-insertion destination length and no accepted comparison. Duplicate
promotion changes current mode only; snapshot old existing state before promotion.
Append events precede batch-wide Maybe-to-Not normalization. No source identities
or unique causes are inferred from scalar equality or dynamic event indices.

## Checked diagnostic hooks, not evaluator modifications

Pin CUE6d609d768f1686f9a3a2a20197cacdbb70e5c79d v0.16.1. Preserve all four
old hook locations/calls unchanged. Add SIX count-checked diagnostic-only lines:

1. Immediately after old crossProduct entry hook, guard on v4-enabled BEFORE
   evaluating int(mode), then record it for that ID.
2. In appendDisjunct after the nil-x early return and before search, reserve an
   append token and snapshot candidate. Token identifies batch ID/event index,
   never retained pointers or a global last-comparison slot.
3. Inside pointer-identity branch before its return, record identity comparison
   and finish this append at the next comparison index.
4. Immediately after `xv := xn.node.DerefValue()` and before the finalized/partial
   branch, guard on v4-enabled BEFORE evaluating statuses, then snapshot existing
   and whether xv/nv are both finalized. This hook reads
   the already computed vertices/statuses; no second Equal/equalPartialNode call.
5. After ALL comparison/remaining-task/value checks succeed and before freeing/
   promotion, mark the last recorded comparison accepted and finish the append.
6. Immediately before `return append(a,x)`, finish insertion with len(a).

Normal comparison index equals the number of earlier comparisons for that append;
identity records its comparison at the same next index. Every compared existing
entry receives exactly one record; no re-execution of equality predicates. No
upstream semantic line is edited. Removing exactly TEN added hook lines must
restore pinned disjunct2.go byte-for-byte; old oracle source stays untouched.

Exact private hook API (single physical guarded lines preserve restoration):

```go
type cuestaceanTraceAppendToken struct {
    capture *cuestaceanTraceCapture
    batch, event int
}
func cuestaceanTraceEqualityEnabled() bool
func cuestaceanTraceRunMode(id, mode int)
func cuestaceanTraceAppend(x *nodeContext) cuestaceanTraceAppendToken
func cuestaceanTraceIdentity(t cuestaceanTraceAppendToken, existing *nodeContext)
func cuestaceanTraceCompare(t cuestaceanTraceAppendToken, existing *nodeContext, partial bool)
func cuestaceanTraceAccept(t cuestaceanTraceAppendToken)
func cuestaceanTraceInsert(t cuestaceanTraceAppendToken, index int)
```

All-zero token means intentionally inactive/ignored; malformed or stale NONZERO
v4 tokens fail sticky. Track pending tokens per batch in LIFO order, not a global
last-comparison slot; completed children cannot overwrite a parent's state.
Capture only appendDisjunct calls belonging to the top active crossProduct AFTER
its trials hook (stage1) and BEFORE results. Calls outside that phase are ignored,
not mislabeled as the parent's batch. Each token is validated against its own
batch/event, capture identity (including across stop/restart), top-of-stack stage
and completion state. Tokens may identify their capture, never source vertices.
Nested/interleaved calls
cannot use a process-global FIFO or borrowed pointer. At results require every
retained append completed once and runMode observed exactly once. Sticky failure
stops recording, detaches cleanly, balances batch stack and emits no partial JSON.
New hooks are no-ops without v4 capture; old modes do not validate/charge/copy extra
metadata. Guarded mode/status arguments are not evaluated in old modes; pointer/
token arguments do not evaluate CUE nodes. No hook repeats equality calls. Diagnostic failure never changes upstream
evaluation, supplies a semantic result or triggers fallback.

## Bounds, ownership and independent evidence

Unchanged 128 batches, 64 per vector, 4096 retained records, 65536 bytes including
LF, finite source128-node/depth16, priority-zero origin guards. V4 appends<=64 per
batch and comparisons<=64 per append; reserve before copying. Total record charge:
`3*trials + 2*nested + 2*results + 3*appends + 3*comparisons <=4096` (candidate/
existing current+origin snapshot plus event metadata). No silent truncation.
V1/v2/v3 exact-cap and byte tests remain; add v4 exact/one-over record/vector,
unknown run mode, missing/duplicate/stale token/out-of-order completion, identity,
partial/final accepted/rejected, stop/recovery and complete ordered-wire controls.
Synthetic hook controls are not claims of source reachability. Source AST admission
is unchanged, not comprehensive hostile-input/allocator/stack hardening.

Go worker owns ONLY existing default-trace/{adt.go,adt_test.go,main.go,main_test.go}.
Second worker performs read-only contract/implementation/evidence reviews, no
implementation ownership. Coordinator owns infra/tooling/flake.nix hooks/restoration,
new executor/tests/default_equality_trace.rs, docs, integration/final validation.
Use at most two isolated stable-main workspaces and immutable handoffs; no saved
commit amendments/rebases onto mutable coordinator parents. Preserve licenses.

Coordinator independently authors raw CUE and complete expected append/comparison
streams before invocation. Execute unchanged native primitive or recursive IR on
handwritten inputs BEFORE Go observation; never construct native trials/IR from
trace/JSON/captured output. Compare full new phase facts and full old v3 subset,
not only export success. Evidence scope is five hand-phase groups/seven complete
source streams; the sixth group adds two preserved-counterexample semantic/v3/
structural controls only, NOT independent phase goldens. Do not count supplemental
structural agreement as full equality-path evidence. Use mandatory absolute/readable/executable providers in
fresh processes with cleared env/unusable PATH and owned scratch HOME. Missing or
failed provider is setup failure, never semantics. Preserve uninstrumented public
API and CLI checks and old complete wire assertions.

Study one/multiple batches, equal scalar duplicates and ordered Default promotion,
failed candidates, nested singleton versus multi-result wrappers, root versus
active-batch depth, returned mode versus predecessor, exact i64s, record guards
and recovery. Include a concrete witness testing a proposed finalized-only
admission shortcut: if a source-reachable partial coalescence agrees with the
native scalar hypothesis, preserve that coverage cost rather than calling it
final or promoting all partial vertices. A disagreement is preserved, not patched.
No equality/readiness admission is authorized by a finite matching corpus.

## Construction and final validation

Native kernels/production sources, policies, generated Rust, 249 JSON outputs,
six old constructor identities/four policy hashes/exact8/6 cuts remain unchanged.
Trace constructor legitimately changes; native/seed full build graphs exclude
its NEW binary/derivation and both provider env entries. Verifier/consumer closures
may depend on Go CUE; no Go-free/minimal-source seed attestation claim.

Run new target and all119 retained groups in four profiles/strict focused Clippy;
both full Rust suites/default all-target Clippy debug/release; pinned offline Go
suites, preflight, TEN actual checks and SEVEN packages built AND rebuilt. Retain
six fresh artifact-free compiler binaries, feature isolation, deterministic drift
and independent source/value evidence. Query recursive derivations AND outputs
(`--requisites --include-outputs`), not runtime references. Linux x86_64 only,
Nixpkgs ac588ef1c914865dad517b337ea0eb9694a4021c. Retire workers before PR;
exact-head protected rebase auto-merge and confirmed required CI. #175 stays held.

## Completed bounded findings

Five hand-phase groups compare seven complete independently authored source streams;
a sixth group retains two supplemental counterexample semantic/v3/structural checks
WITHOUT claiming independent new-phase goldens. Reviews closed that original scope
ambiguity by narrowing the claim, not inventing expectations from captured output.
All source cases run native authored IR before fresh Go observations. Complete v3
subsets remain equal; public API/CLI facts stay separate from native construction.
Renaming additionally preserves full v4 bytes.

`(1|*1|2)&(1|2)` supplies a source-reachable accepted PARTIAL comparison in the first
attempt-only batch: Default promotion of scalar1 compares against an existing Not
whose original mode is Not. The later batch uses finalize. This conclusion uses
the observed prebranch statuses and actual acceptance hook, NOT runMode alone.
The unchanged native hypothesis matches that control; requiring every comparison
to be finalized would defer it. That is an explicit coverage cost, not a proof
that partially evaluated vertices generally admit complete-i64 coalescing. Candidate
and existing snapshots precede promotion, and nested append snapshots precede the
later batch-wide Maybe-to-Not normalization. No admission rule is added.

Both focused contract reviews approved before explicit GO; read-only implementation
and narrowed evidence reviews approved immutable Go
`ba318aadff697840c2433c7ce9cf87c46ce7a167` (parent15499090) and evidence
`a9ec61f66b63e3a8dd1297f44890a10345533fd4`. Retained Go test bodies/assertions
are byte-identical. Token controls include stale captures, nested pending appends,
a stage0 child above stage1 parent, exact64/65 vectors and exact4096/attempted4097
records, sticky failure and recovery. Created workers/workspaces retired before PR.

All 125 research groups passed four profiles/strict focused Clippy; both full Rust
suites/default all-target Clippy passed debug/release. Pinned offline Go suites,
formatting, preflight 13/13 (seven inapplicable skips), all TEN actual sandbox checks
and SEVEN packages passed build AND rebuild on x86_64-linux. Six old constructors,
exact8/6 cuts, four policy hashes, 249 JSON files, generated Rust and native kernels
remain unchanged. New trace producer `ai6yrb9b…` restores
pinned source after exactly TEN additions/zero removals. Fresh recursive inventories:
all three native verifiers 1407 derivations/3392 full build paths; seed 1436/3466.
Both current provider binaries/derivations, downstream executor and provider env
entries remain excluded. No Go-free/minimal-source seed claim follows.

Remaining work is a separately reviewed contextual scheduling/equality/projection
admission, not selecting values from diagnostic events. Source parsing, abstract
domains, references/layers/collections and production demand remain unestablished.
#175 and its saved prototype remain held.
