# CUE-owned flat preference projection experiment

## Reviewed contract

Milestone [#184](https://github.com/kolohelios/cuestacean/issues/184). Two focused
kernel/evidence reviews preceded implementation, clarifying scope-admission
precedence and projection/certificate/extraction work order.

Follow [#180 dispatch policy](demand-policy.md) and
[#182 native artifacts](native-demand-policies.md). Move one further executable
choice into a CUE-authored policy: **which already-computed flat domain to project**
(ordinary candidates, preferred candidates, or neither/defer). This is test-only;
it does not solve nested default lifetime or unhold
[#175](https://github.com/kolohelios/cuestacean/issues/175).

Native code still computes ordinary worlds, flat association/set operations,
occurrence support, reports and certificates. CUE owns only dispatch and this
projection choice, not complete specification/default semantics. Keep solve,
native demand and the single-dispatch-policy API as independent reference paths;
no old assertion/falsification may disappear. No production src/, source language,
raw production CUE, Cargo dependency, guard or generated Rust changes.

## Two scoped policy phases

Reuse the existing strict if/return instruction decoder/VM, with explicit private
policy scope. Existing `Policy::from_json` constructs dispatch scope; add
`Policy::from_projection_json` for projection scope. Wire formatVersion remains 1.
Admission validates EVERY arm against its scope, including dead arms:

- dispatch facts: empty/forced/recipe/nested; actions constraints/flat/defer;
- projection facts: absent/bottom; actions ordinary/preferred/defer.

Neither constructor accepts the other phase's facts/actions. The shared `defer`
spelling is deliberately not enough to infer scope: phase comes from the checked
constructor, not filename, action or source data. Unknown/duplicate/invalid or
wrong-scope policies are Unsupported; existing text/node/depth caps remain Limit.
Both APIs reject wrong-phase arguments at request admission, even when dispatch
would not execute the projection program. Scope rejection precedes allowance
checks too: wrong phase with zero or >4096 allowance is Unsupported; a correctly
scoped excessive allowance is Limit. Admission scope checks are like graph/
policy structural typing, separate from execution-work charges; do not reset Work.

Proposed shared API (agree before implementation):

```rust
// Existing Policy type; add from_projection_json(&str)->Result<Policy,Error>.
// Readiness adds ProjectionDeferred; existing variants/reference meanings stay.
// Graph::demand_with_policies(refinement:Domain, work_limit:usize,
//     dispatch:&policy::Policy, projection:&policy::Policy)
//     ->Result<DemandReport,Error>
```

The new entry uses checked predecision observation and the dispatch VM as before.
Constraint-only/defer actions preserve existing behavior. A flat action MUST pass
its existing no-entered-nested certificate before calculating association or
calling projection policy. Do not call reference demand and retry on errors.

## Projection fact/action certificate

From the computed flat association, supply graph-internal facts only:

- absent = Association::Absent;
- bottom = Association::Present(empty);
- absent=false and bottom=false means nonempty preferred candidates, including
  abstract candidates. These facts do not contain a selected result/action.

CUE canonical policy: absent -> ordinary; bottom -> ordinary; otherwise preferred.
A separate raw CUE alternate returns defer. VM instruction visits, fact reads,
both action certificates, set/report copies and scalar extraction share the same
4096 execution allowance. Execution order is dispatch VM -> dispatch certificate
-> flat association -> projection VM -> projection certificate -> preference
report materialization -> selected-domain extraction (unless deferred) -> report
completion. Unsafe requests reject before preferred-vector materialization.
On the new two-policy path charge one extra scalar-extraction step for successful
ordinary/preferred projection; deferral does not extract. Charge one final report
completion step as before. Preserve old API thresholds/work behavior; old native
extraction remains covered by its existing report accounting. Canonical code must
not be a hidden native if-chain.

Native projection certificates:

- ordinary is permitted ONLY for absent/bottom association; otherwise Unsupported
  (cannot erase viable preferences just to obtain a concrete ordinary candidate);
- preferred requires a nonempty preferred association, otherwise Unsupported;
- defer is always permitted AFTER flat's safety certificate; no scalar extraction
  or native reference-policy fallback follows.

Build `preferred: Some(Absent/Bottom/Values)` as the observed flat preference
report, distinct from ordinary constraints and selected domain. On projection
deferral retain that evaluated report, return ProjectionDeferred/Suspended, and
publish no concrete demand. Dispatch deferral remains PolicyDeferred/Suspended
with preferred None because no flat association was evaluated. Canonical
projection otherwise returns FlatPreference plus Concrete/Incomplete/Conflict;
empty ordinary candidates yield Conflict. Compatible abstract preferred candidates
stay Incomplete even if ordinary candidates contain a concrete value.

## Native policy artifact family

Add three raw test-only files, package `projectionpolicy`:

- bootstrap/spikes/preference-projection-schema.cue (definitions only);
- bootstrap/spikes/preference-projection.cue (canonical 5-node if/return tree);
- bootstrap/spikes/preference-projection-defer.cue (single defer).

Extend the native-demand-policies artifact's explicit manifest/fileset to these
schema/program pairs, producing two additional build-only JSON files. Preserve
its existing mandatory environment and producer -> consumer / separate oracle
boundary. Every policy exports twice under empty env/unusable PATH; exact schema
plus one program, never wildcards or both programs together.

The policy family's producer/raw fileset/full graph proof legitimately changes
from three to SIX raw files. Re-attest the new native construction cut; keep the
original exporter/AST atom/family/full-production-JSON derivations unchanged.
The old two dispatch JSON byte hashes/results must remain unchanged. The
exporter's eight-file production source allowlist stays intact; only its test
variant gains the three new fixtures. No default/phase special cases in exporter.
Go CUE is a separate raw-source/semantic oracle, never provider/fallback.

## Ownership, bounds and independent evidence

Two focused contract reviews precede implementation. Kernel worker owns only
bootstrap/spikes/{default_provenance,demand_policy}.rs. Evidence worker owns new
bootstrap/executor/tests/preference_projection.rs and
bootstrap/spec-export/tests/demand_policies.rs. Coordinator owns raw CUE, Nix,
docs and additive API exercises in the three older integration targets to avoid
private-module dead-code suppression. New tests exercise all reference APIs too.
Use stable merged-main workspaces and immutable commits, not coordinator rebases.

All existing caps remain: 16 KiB original policy text; 64 decision nodes (63
accepted/65 rejected due binary-tree parity); 16 levels; graph 128 occurrences,
16 levels/groups/width, 64 worlds/raw products and 4096 cumulative execution work.
No stack overrides, lint suppression, global caches or new dependencies.

Required evidence:

- Canonical native-built dispatch+projection match handwritten action/domain/
  readiness expectations and old native/single-policy references on flat cases.
- Alternate raw projection changes concrete flat preference output to Suspended,
  preserves Some(preferred), ordinary/group/recipe facts, and does not recompile
  Rust. Distinguish dispatch's earlier None deferral and do not execute projection
  on constraint/defer paths.
- Phase-mixing including common defer and invalid DEAD arms rejects before use;
  unsafe ordinary/preferred requests reject at certificate exact/one-below budget,
  not Conflict/Incomplete or a reference fallback. Nested flat stays forbidden.
- Absent/bottom/nonempty/abstract preferences, conflict/forced safe edges, source
  grouping and both operand orders, repeated observations and failure recovery.
- Handwritten full JSON expectations for both new native exports, pinned CUE and
  native CLI/library byte/order/mutation checks, with all old provider tests intact.
- Admission caps, shared two-phase execution allowance, and unchanged source/
  artifact independence. Scope/policy typing is not hostile-input hardening.

Run focused debug/release × default/no-default and strict Clippy, both full Rust
suites, pinned preflight and all NINE actual Nix checks/builds/rebuilds; keep six
fresh seeds, profiles/drift, all twelve builders and independent source/value
oracles. Pin CUE v0.16.1/source `6d609d768f1686f9a3a2a20197cacdbb70e5c79d`; explicit
raw source files/fresh processes/independent expectations. x86_64-linux only.
Stay near 1500 changed lines including CUE/docs/tests/Nix. Nested preference
semantics and production integration still need separate reviewed proposals.

## Review and validation findings

Both contract reviews preceded implementation and clarified phase-first admission
and the certificate/materialization/extraction sequence. Focused cross-review
required an exact boundary for unsafe ordinary selection with nonempty preference:
for `*1|2`, safe preferred work minus five reaches the certificate (three preferred
projection steps, one extraction, one report remain). One less is Limit; exact is
Unsupported. The added test closes that finding before preferred allocation.
The reverse review found no remaining kernel/artifact-boundary blockers.

Immutable inputs, both parented on main `c18cd995`:

- Kernel `4762ef296ffc46aa3a6a9376dbf6042505a8b36e` (two spike files).
- Evidence `1e006895917e4e7f910b89812e9c5acb365b3cd7` (two test files).

The 8 new and 42 older/additive executor groups pass debug/release ×
default/no-default, with strict focused Clippy. The 7 exporter groups pass
both profiles. Both complete Rust suites and all-target strict Clippy pass
in debug/release. No-default checks intentionally target the spike integrations
and existing feature-isolated seeds: default-only integration imports do not
constitute a supported all-target no-default suite.

Pinned preflight passes 13/13 (7 inapplicable skips). All nine current Nix checks
and the five native/export packages were explicitly built and sandbox-rebuilt;
six fresh compilers, feature profiles, drift, twelve builders and independent
source/value oracles remain intact. This validates Linux x86_64 only.

The original four production derivation identities match #182 exactly. The two
dispatch JSON hashes in [the original cut](native-demand-policies.md) are unchanged.
Projection JSON SHA256: `4bde2ab024774c9ed162b1c29cb99786b58cd176b3317cf9f31e7172c9e40108`.
Projection-defer SHA256: `873ab27094b14372adcce54685d13b0c2cf85d60aa6f86755cda68b0e419511a`.
The latter deliberately equals dispatch-defer's wire bytes: the explicit decoder
constructor, not the common return action, supplies private phase identity.

Re-attested construction identities:

- Producer `/nix/store/m8gfkv10sckly85arqsfkkwl771wxv18-cuestacean-native-demand-policies.drv`.
- Output `/nix/store/bsmhl6w5q9pbqbdmb3xfbblv925n4rnc-cuestacean-native-demand-policies`.
- Exact six-file raw source `/nix/store/06c9bq71vs4rldvajyh1wxis8aklcd08-source`.
- Unchanged eight-file exporter source `/nix/store/7gjdbv2crqz1b67qfi93sayc6n27qjiw-source`.

The fresh recursive derivation graph has 1407 derivations; its full build closure
has 3392 paths. An independent `nix-store --query --requisites --include-outputs`
query agrees with the verifier's retained `native-build-paths`; querying only
requisites without outputs is insufficient. The verifier also retains roots and
registration. Pinned CUE/source, full production JSON and the downstream executor
are excluded from construction, not from the separate verifier/consumer closure.
Reproduce using the native-family build/check commands in the linked runbook and
`nix derivation show --recursive` on this producer (Nix 2.34 JSON uses the
`derivations` object). No generated JSON or handwritten expected tree is an input.
