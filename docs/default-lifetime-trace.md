# Pinned default-lifetime batch trace

## Selection and review gate

Return directly to held [#175](https://github.com/kolohelios/cuestacean/issues/175)
after [the failed context hypothesis](contextual-preference-spike.md). Observe the
pinned implementation's actual branch-local default transitions BEFORE proposing
another native lifetime algorithm. This is a separate, instrumented Go-CUE oracle,
not a native solver, readiness certificate, production fallback or authorization
to merge the saved default exporter. All old hypotheses, gates and assertions
remain unchanged. No production/native exporter language, policy or guards change.

The pinned evaluator's `internal/core/adt/disjunct2.go:crossProduct` computes
`leftDropsDefault`/`rightDropsDefault` over the successful trials of a particular
batch, THEN combines current/original default modes and unrolls nested disjuncts.
This is a narrower observation than static all-world support or final public
`Default()` flags. Source layers/priorities are not inferred from nesting.

Milestone [#200](https://github.com/kolohelios/cuestacean/issues/200).
Two focused contract reviews approved the clarified wire/profile/capture contract
before explicit implementation GO. Read-only implementation cross-reviews and
full validation precede PR/merge.
Keep approximately <=1500 changed lines. Preserve Apache-2.0/upstream notices.

The additive [#202 origin-aware extension](native-batch-defaults.md) adds a fourth
v2-only capture hook and `trace-origins` observations, preserving all v1 wire bytes
and limits. The three-hook construction/results below describe the original #200
milestone, not the later producer identity. The subsequent [#204 history mode](flat-default-sources.md)
adds v3 nested/final result origins without extra hooks, preserving both old modes.
The [#208 equality trace](default-equality-trace.md) subsequently adds v4 run-mode/
coalescing observations through six additional diagnostic lines; v1–v3 remain
byte-stable. Four-hook identities below describe their original milestones.

## Instrumented oracle, not changed semantics

Add an independent `default-trace` package. Build in a writable COPY of the exact
pinned CUE module/vendor tree. NEVER modify the store source or public observer.
Add a new authored ADT helper file in that copy and three count-checked hooks to
`crossProduct` only: enter, after successful trial collection/before mode
combination, and before return/after nested unroll/normalization. Assert each
replacement needle occurs exactly once and fail setup on source drift. Retain
pinned source/upstream notices. No original expression, mode, priority, result,
loop, error or branch condition changes. Hooks only copy bounded diagnostic data.

The original public observer executable/source/profile/v1/v2 bytes remain intact.
The instrumented command is a NEW executable and provider, never a substitute
for ordinary `cue` or `CUESTACEAN_DEFAULT_OBSERVER`. Its final public-API result
must agree with independently authored expectations AND fresh uninstrumented
observer/CLI runs. Instrumentation equality on tested cases is evidence, not a
universal proof that instrumentation cannot perturb evaluation.

CLI: `cuestacean-default-trace trace FILE FIELD`. Exactly one raw file, at most
65536 bytes, optional package clause and exactly ONE ordinary ASCII identifier
field. FIELD must be that exact label (1..64 bytes); reject the original public
observer's seven predeclared-shadow labels (`_ int bool string bytes number float`)
so valid trace input stays in its independently compared source profile.
No selectors, imports,
references, aliases, definitions, attributes, optional/required/hidden fields or
containers. Expression profile is finite signed decimal i64 literals,
parentheses, `&`, `|` and direct default `*` marks only. One leading `+`/`-`
may apply to a direct decimal integer literal, including exact i64 MIN/MAX;
nested signs/signs on parenthesized expressions, nondecimal spelling and numeric
separators are excluded. No types/top, other numbers, arithmetic or booleans. Bound admitted expression topology at 128 nodes /
16 levels. Validate BEFORE enabling tracing; AST admission is not comprehensive
hostile-parser/allocator/stack/time hardening.

Start capture BEFORE BuildFile, include lookup and the same public default /
Err / MarshalJSON observation path as the old inspect mode, then stop capture
BEFORE encoding diagnostic output. One fresh context/process per observation.
Do not apply global root.Err rejection, retry another engine or skip a provider.

## Facts and wire boundary

Emit compact deterministic JSON + LF, ordered fields:
`formatVersion:1`, `mode:"trace"`, `defaultSelected:bool`,
`status:"concrete"|"incomplete"|"conflict"`, `value:raw-json-or-null`,
`batches:[...]`. Final value and status describe the public observation, NOT
raw ordinary constraints, Absent/Bottom classification or a native decision.

Each batch records zero-based entry-order `id`, zero-based dynamic crossProduct
nesting `depth` (not nodeContext depth), its two drop-default bools,
pre-combination successful `trials`, and final `results`. Trial facts include
current and incoming/original modes and nested child modes; final results retain
combined mode. Modes are explicitly named `maybe`, `default`, `not` from the
pinned private enum, NOT the native Preference enum. Include a nullable exact-i64
value when already available WITHOUT evaluating a vertex; null means unavailable,
not CUE null/default absence. No pointers, allocation IDs, filenames, addresses,
timing or unstable debug strings. Batch IDs are local dynamic observations, not
source occurrence IDs or stable identities across reordered expressions. Only
successful trials are retained; this is NOT a complete failed-branch/source-
occurrence provenance trace or a reconstruction of preference lifetimes.

Shared wire fields (all present, arrays always `[]`, not null):
`Batch{id,depth,leftDropsDefault,rightDropsDefault,trials,results}`;
`Trial{mode,originalMode,value,nested}`; `Result{mode,value}`. `value` is nullable
signed i64. `nested` holds only the IMMEDIATE `r.disjuncts` child mode/value
records, not recursive trial objects. Available integers come only from direct
`node.BaseValue.(*Num)` with integer kind and successful exact `X.Int64()`;
otherwise null. Do not evaluate OR dereference vertices to discover values.
Modes have only the three spellings above. The ADT helper exports
`CuestaceanTraceStart() error` and `CuestaceanTraceStop() (batches,error)` to the new
command only; hook calls are `cuestaceanTraceEnter() int`,
`cuestaceanTraceTrials(id,tmp,leftDropsDefault,rightDropsDefault)` and
`cuestaceanTraceResults(id,dst)`. Go tests may be staged beside the command/helper
in their respective copied upstream package directories.

Use one private active capture per fresh single-command process; no service or
concurrent clients. Bound retained diagnostics: 128 batches, 64 trials/nested
children/results per vector, 4096 total copied mode records, 65536 output bytes.
On overflow mark capture failed and stop retaining records, but do NOT alter
upstream semantic branches/limits. Finish with nonzero/stderr/no JSON. These are
trace-retention bounds, not upstream solver work/time bounds. Records count
each trial, each immediate nested mode and each final result; batch headers
have their separate 128 cap. Start rejects an overlapping capture; stop always
detaches capture, including on failure. Overflow remains sticky. Completed
captures require balanced hook entry/exit. No partial trace on
setup/admission/overflow/encoding failure, no silent truncation or provider fallback.

## Ownership and independent evidence

Implementation owner ONLY new files under `bootstrap/oracles/default-trace/`:
Go command, ADT hook helper and scoped Go tests. Evidence owner ONLY new
`bootstrap/executor/tests/default_trace.rs`. Coordinator owns Nix wiring/source
hook placement, docs, comparisons, immutable integration, graph proofs and final
validation. At most two stable merged-main workspaces; exact immutable commits,
no saved amendments/rebases onto mutable coordinator parents. Agree additional
ownership before edits. Private spike APIs/gates are not changed or suppressed.

Use separately authored full expectations for flat trace controls and semantic
results, plus explicit hand-authored internal fact expectations for preserved
#176/#186/#188 failures and asymmetric source pairs. Cover both operand orders,
renamings/repeated byte determinism, flat live conflicting defaults versus dead
marks, nested unroll and actual batch drop flags, M2/M3 controls and the correctly
marked M2 source. The old abstract `*int` asymmetric observation stays unchanged
in the public observer; it is an explicit profile exclusion for this finite-i64
trace, not silently substituted by a concrete source. Do not copy captured trace
dumps as goldens, turn a final flag
into a recipe certificate, or assert causality from event correlation alone.
Preserve exact source spellings and separate genuine both-order failures from
pinned asymmetries. Record whether observed transitions explain a native model's
missing mechanism; do not change that model to fit rows in this slice.

Require invalid argc/mode/path/label/profile/cap/provider tests and no-JSON setup
failures. Missing/relative/non-executable `CUESTACEAN_DEFAULT_TRACE` is fatal.
Fresh instrumented processes use cleared environment/unusable PATH. Independent
uninstrumented public observer and CLI comparisons remain mandatory, not fallback.

## Construction and validation

Add `default-trace` package and mandatory absolute provider ONLY to downstream
executor/devshell, NOT base rustPackage/codegenSeed. The existing `default-observer`
verification check explicitly expands to attest BOTH oracle providers outside
seed construction and test both; retain its old observation and proof outputs.
All three native graph verifiers additionally exclude the new trace .drv. TEN
checks remain, now covering this explicitly expanded oracle family; seven selected
packages build/rebuild (the old six plus default-trace). Native constructors,
249-file JSON bundle, atom/12-builder/policy bytes, four policy hashes and exact
8/6 source cuts MUST remain unchanged; observer identity stays unchanged too.

Observer/consumer/verification closures may use Go CUE; trace/helper/test/expected
answers must never feed native construction. Re-attest all three recursive native
derivation/full-build graphs and seed proof (`--requisites --include-outputs`),
excluding CURRENT providers/consumers at the correct boundaries. No Go-CUE-free
seed or minimal source-attestation claim. Six fresh artifact-free compilers,
feature isolation, deterministic generation/drift and source/value oracles remain.

Run focused new+retained 76 graph/oracle +14 solver groups in four profiles with
strict scoped Clippy, both full Rust suites/default all-target Clippy debug/release,
scoped offline Go tests, pinned preflight, all ten actual Linux checks and seven
package builds AND rebuilds. Source pin: CUE 6d609d768f1686f9a3a2a20197cacdbb70e5c79d
(v0.16.1), Nixpkgs ac588ef1c914865dad517b337ea0eb9694a4021c. Linux x86_64 only.
#175 stays held regardless of observations; no default/bootstrap/self-hosting or
performance claim follows from tracing an upstream implementation.

## Mechanism observations, not a replacement calculus

The source predicate for left-drop checks BOTH `p.defaultMode` and
`p.origDefaultMode`. In the preserved `((*1|2)|*3)&(*2|1)` and
`((*1|2)|*3)&(1|2)` failures, the final forward batch has only `not` current modes
but `leftDropsDefault:false`. Current modes alone therefore do not reconstruct
that batch decision; global source-support facts are a different observation.
Trial `originalMode` is the
INCOMING `r.origDefaultMode`, not the LEFT `p.origDefaultMode`; the latter is not
captured. This is an explicit diagnostic limit, not complete causal provenance.

For `(((*1|2)&(*2|1))|1)&(*1|2)`, a nested `not` vector and a plain `maybe`
trial precede final `not` results. `appendDisjunct` also coalesces equal values;
this trace cannot attribute final 1 to a particular source trial or identify
`hasNonMaybe` as the unique cause. Both stages matter to future investigation.
For `(*((*1|2)|3)|(*4|5))&(1|4)`, outer normalization leaves 4 `not`, and later
refinement retains that status rather than reviving its nested mark. These
source-grounded transition observations constrain a future native state machine;
they do not establish one or authorize treating native Bottom as a union identity.
No event correlation is promoted to a general lifetime/reactivation law.

## Completed review and validation

Immutable Go input `b8b241669cef6817dae7d93c3a5e7afa48e868eb` and evidence
`f3bb042a039f5f21d4c7442f75e8f6e81c4fd4d6` descend from merged #198 (`c6876d43`).
Review required positive exact-cap coverage; test-only child
`57ae04e6756d7dc21502e3bdaed9f738b7621dc8` closes 128-batch/64-vector/4096-record
and 65536-byte-including-LF boundaries, one-over, sticky overflow and recovery.
No frozen commit was amended; both implementation cross-reviews approved and
created workspaces were retired. The restored pinned source matches byte-for-byte;
`instrumentation.diff` has exactly three additions, no removed/changed core lines.

Eight Rust groups pass four profiles/strict Clippy: nine full handwritten flat
traces, 24 ordered semantic/drop/result expectations, scoped lifecycle facts and
profile/provider boundaries. All 90 retained graph/oracle/solver groups, both full
Rust suites/default all-target Clippy debug/release and offline Go suites pass.
Preflight passes 13/13 (seven skips); ten checks and SEVEN packages build/rebuild.
Fresh compilers/features/drift/source/value checks remain; whitespace scans 1297
files. Exact 8/6 source cuts, six pre-existing constructor identities, four policy
hashes, 249 JSON files, original observer and generated Rust remain unchanged.

Trace constructor `c324pdqb…` produces
`/nix/store/4x41j5pi7gzcrp99dk9zidvkfk1jn81j-cuestacean-default-trace`.
Independent recursive queries match all three native inventories (1407 derivations /
3392 full build paths each) and seed inventory (1436/3466). BOTH oracle binaries/
derivations are excluded at those boundaries, with neither provider in seed env.
The seed's broad source snapshot is not a minimal-source attestation. Only
Linux x86_64 is validated; #175 and the saved prototype remain held.
