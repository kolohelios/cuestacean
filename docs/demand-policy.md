# CUE-owned conservative demand dispatch experiment

## Reviewed contract

Milestone [#180](https://github.com/kolohelios/cuestacean/issues/180). Two focused
kernel/evidence reviews preceded implementation; clarified empty flat requests,
strict admission categories, binary-tree node parity and exact execution charges.

Follow [#178 demand readiness](default-readiness.md) and the
[DSL/native audit](dsl-boundaries.md). Move **one executable decision** behind a
checked CUE-authored instruction policy: choosing constraint-only completion,
flat preference projection or deferral from already-audited scalar graph facts.
This is test-only policy-ownership progress, not production default support,
CUE specification evaluation, a complete contextual default calculus or permission
to unhold [#175](https://github.com/kolohelios/cuestacean/issues/175).

Keep Graph::solve and the conservative native Graph::demand as reference paths;
preserve every existing falsification/assertion. The held exporter prototype is
not imported. Production parser, evaluator, exporter fileset/derivation cut, raw
production CUE and generated artifacts remain unchanged.

The [#184 scoped projection follow-up](preference-projection-policy.md) adds a
separate ordinary/preferred/defer phase. `from_json` remains dispatch-scoped;
projection admission is explicit and wrong-phase requests reject even when that
phase would not execute. The single-policy path below remains reference evidence.

## What CUE owns versus native work

A raw CUE schema and policy program under `bootstrap/spikes/` export a small
instruction tree. At #180, Go CUE was an explicit **test-policy export dependency**,
not an oracle secretly called after semantic failure. The
[#182 native provider](native-demand-policies.md) subsequently closes those two
artifact-construction paths; Go CUE remains a separate mandatory oracle, not the
provider. Supply native policy artifacts before graph calls; missing environment/
files fail setup. The graph/VM never executes processes, reads policy paths or
retries through CUE.
Tests keep independent handwritten expected actions/results plus mandatory pinned
CUE semantic probes, rather than comparing the same policy with itself.

Raw paths: `bootstrap/spikes/demand-policy-schema.cue` supplies definitions only;
`bootstrap/spikes/demand-policy.cue` and `demand-policy-defer.cue` are separate
programs. All use package `demandpolicy`. Reproduce the separate oracle by
exporting the entire root with
`cue export <schema-path> <one-program-path> --out json` (no expression selector).
Never load the canonical and alternate program together. Use fresh processes.

Represented DSL:

```text
{formatVersion:1, body:
    {op:"return", action:"constraints"|"flat"|"defer"}
  | {op:"if", field:"empty"|"forced"|"recipe"|"nested",
     then:decision, else:decision}}
```

CUE owns this ordered dispatch: empty -> constraints; forced -> constraints;
no recipe -> constraints; nested -> defer; otherwise -> flat. The authored tree
is actual executable branching, not a selected answer table. An independently
exported alternate CUE program may defer more, including flat/forced cases;
executing that alternate must visibly change the path without recompiling Rust.
No filename, graph label, literal number or case-specific result lookup.

Native work remains explicitly substantial: scalar meet/world enumeration,
occurrence identity, support/recipe auditing, fact extraction, checked VM/storage/
accounting, response certificates, primitive preference implementation, scalar
extraction and report construction. CUE does not perform those algorithms here.
Native reference dispatch remains independent test evidence, not a production
fallback or evidence the whole graph is CUE-hosted.

## Checked request/response interface

Proposed shared API (agree before concurrent edits):

```rust
// bootstrap/spikes/default_provenance.rs:
#[path = "demand_policy.rs"]
pub mod policy;
// existing Readiness gains PolicyDeferred, only for policy-selected deferral
// Graph::demand_with_policy(refinement:Domain, work_limit:usize, policy:&policy::Policy)
//     -> Result<DemandReport,Error>
// bootstrap/spikes/demand_policy.rs:
pub struct Policy { /* private validated instruction tree */ }
// Policy::from_json(text:&str)->Result<Policy,super::Error>
```

Reuse existing Domain/Error/DemandReport. New policy-driven deferral returns
Readiness::PolicyDeferred/Demand::Suspended/preferred None; the reference still
uses DeferredNested. Existing result categories/assertions do not change.

Refactor only common **predecision** observation (ordinary/refined worlds,
constraints, all group facts, syntactic recipe and entered-nesting facts). Neither
the native decision path nor a computed old action/readiness/outcome is an input
fact. VM inputs are graph-internal, not caller-forged booleans:

- empty: no surviving ordinary worlds;
- forced: exactly one deduplicated exact i64, without an abstract candidate;
- recipe: source syntax contains a preference recipe;
- nested: an entered Or has a direct child containing a syntactic recipe.

The VM generically reads a typed fact and follows if/return instructions. Selected
policy execution charges the SAME Work instance as observation, response checks
and any flat projection; never call reference demand/solve and reset its budget.
Validation checks all policy arms, including dead arms, before any graph execution.

Native action certificates constrain what a policy is allowed to request:

- constraints requires empty, forced, or no source recipe; otherwise reject.
  The checked constraint extractor determines Conflict/Concrete/Incomplete.
- flat requires no entered nested preference dependency; otherwise reject before
  evaluating a preference. This is a safety invariant, not a default algorithm.
  Empty ordinary worlds are permitted and produce FlatPreference/Conflict with
  Some(Absent); forced nonnested requests are also permitted. Never turn an
  empty flat result into Incomplete.
- defer is always permitted and yields PolicyDeferred/Suspended without evaluating
  preference policy, even if native/CUE could produce a concrete value.

A rejected action is Unsupported, not Conflict/Incomplete/Suspended and never a
reason to execute the reference dispatcher. No silent substitution of default
policy. Distinguish policy branch choice from native certificate enforcement;
do not label a duplicated native dispatch table as CUE-owned behavior.

## Decoder, bounds and ownership

Policy JSON admission: 16 KiB original text, exactly formatVersion 1, at most
64 decision nodes, 16 root-inclusive decision levels; only known instructions,
facts/actions/fields and no duplicate keys. Decode the original text with typed
strict serde structures; reject trailing JSON and validate every branch.
Malformed/unknown/duplicate/type/version failures are Unsupported; byte/node/depth
limits are Limit. The text cap counts original UTF-8 bytes inclusively. Decision
nodes are instructions, not JSON values; a full binary if/return tree always has
odd size, so 63 passes and 65 fails the 64-node cap. Validate depth 16/17 separately.
Typed deserialization precedes decision-depth admission and retains serde's own
recursion guard; do not imply the depth cap is pre-deserialization hardening.
Construction is structurally bounded separately, as Graph::new already is. This trusted-test input is not a
comprehensive hostile-wire parser/allocator/stack-hardening claim.

Keep the graph's existing 128 occurrences, 16 syntax/groups/width, 64 worlds/raw
products and 4096 cumulative execution work. Charge one per visited instruction,
one per executed fact read, and one response-certificate charge before checking
or executing the action, shared with all existing work. Unvisited policy arms
are validated at admission, not charged as executed instructions. Preserve all
ordinary constraints/group facts for every action, including conflict/defer.
No raised guards, stack overrides, lint suppression, new Cargo dependencies,
committed generated JSON/Rust, process execution in kernel/VM, or global caches.

Kernel owner: `bootstrap/spikes/default_provenance.rs` and new
`bootstrap/spikes/demand_policy.rs`. Evidence owner: new
`bootstrap/executor/tests/demand_policy.rs`. Coordinator owns raw CUE schema/
canonical/alternate programs, docs/Nix, additive API-exercise tests in both old
integration targets, integration and final validation. Both old private-module
crates must exercise decoding/new API to retain strict dead-code checks without
visibility escapes. New evidence target must exercise reference solve/demand too.
At most two stable-main workers, immutable commits and no coordinator-parent rebase.
Complete two focused contract reviews before implementation.

## Required independent evidence

- Canonical CUE policy agrees with handwritten demand/action expectations and the
  old native decision on ordinary/flat/deferred cases (acknowledge distinct
  PolicyDeferred label); old falsification and conservative coverage-cost evidence
  remain intact. Do not claim nested defaults are now solved.
- Alternate raw CUE policy changes actual demand execution to deferral, including
  a flat/forced case; no Rust switch/expected answer seeds the change.
- Unsafe return-flat on nested and return-constraints on unresolved recipe reject
  by native certificate, without fallback or preferred-value evaluation.
- Unknown/duplicate fields, unknown flags/actions/opcodes/version, malformed JSON
  and invalid dead branches fail at admission. Check 63 accepted/65 rejected
  decision nodes, 16 accepted/17 rejected levels and inclusive text-cap boundaries.
- Shared work exact/one-below limits, failure recovery, repeated calls, both
  refinements/operand orders, preserved group/recipe facts and no cross-call cache.
- Mandatory fresh pinned CUE v0.16.1 processes/source
  `6d609d768f1686f9a3a2a20197cacdbb70e5c79d`, explicit raw inputs and independent
  output expectations. Policy export dependence is outside the attested native
  production cut; compare native production derivation identities/fileset.

Run new and old focused tests debug/release × default/no-default, both full Rust
suites/strict Clippy, pinned full preflight and all eight actual sandbox checks/
rebuilds. Retain six fresh artifact-free compiler binaries, feature isolation,
generation/drift and independent source/value oracles. Only x86_64-linux is
validated. Keep the entire slice, including raw CUE/docs/tests, near 1500 changed
lines. A later production/default-lifetime proposal still needs separate review.

## Findings: one executable policy boundary, not a default solver

The canonical raw CUE program now selects the conservative action by executing
its if/return tree over checked predecision facts. It agrees with independent
handwritten expectations and the retained native reference on constraint-only,
flat and unresolved nested cases (with the explicit PolicyDeferred label).
The alternate raw CUE program visibly changes both forced and flat demands to
Suspended/preferred None, preserving all ordinary constraints/group facts. Rust
is unchanged between those executions: action selection is not a renamed native
dispatch table or an expected-answer lookup.

Native certificates reject unsafe constraint/flat requests as Unsupported at the
certificate-stage work boundary. One less step yields Limit; sufficient work
yields rejection before preference projection, not a reference retry or fallback.
Safe flat requests on empty/forced nonnested observations remain permitted, with
empty yielding Conflict/Some(Absent). Deferral is intentionally conservative and
can refuse a demand that the native/CUE path could complete.

The admission tests preserve original duplicate keys and check unknown/dead arms,
trailing input, version/types and UTF-8 byte accounting. They exercise 63 accepted
and 65 rejected instruction nodes, depth 16/17, inclusive text caps and shared
execution-work exact/one-below thresholds. All policy branches are validated even
when not executed. No larger guard or lint suppression is needed.

Decision: the checked facts -> executable CUE-authored dispatch -> certified
native action boundary is viable for this test-only scalar fragment. Native
world/meet/audit/fact algorithms, checked VM/accounting/certificates and flat
preference primitives remain native, not passive storage. The native reference
is evidence, not a fallback. Go CUE was the explicit test-policy exporter in #180;
[#182](native-demand-policies.md) replaces that provider without changing dispatch
semantics or weakening separate upstream comparisons.
Nested preference suppression/reactivation is still unresolved, the old static
falsification and conservative coverage cost are intact, and #175 remains held.
This does not authorize a production graph or claim CUE specification semantics
are now implemented by the represented policy.

## Validation and unchanged native production cut

On x86_64-linux all eight new policy groups and 14/15 old integration groups pass
debug/release under default/no-default features, with strict focused Clippy in
all four combinations. The two added old-target API exercises avoid dead-code
suppression; no old assertion is removed. Both full Rust suites pass debug/release
and strict all-target Clippy. Fresh mandatory pinned CUE processes use explicit
raw schema/program combinations and independent semantic expectations. Native
policy matrices cover all six three-term permutations in both conjunction orders;
the retained earlier tests also compare the bounded permutation cases upstream.

Pinned preflight passes 13/13 (7 inapplicable checks skipped); all eight Nix checks
and native exporter/atom/family pass actual sandbox rebuilds. The new policy test
is explicitly exercised without default features. Six fresh artifact-free compiler
binaries, feature isolation, deterministic generation/drift and independent source/
value evidence remain intact. Other architectures are omitted, not validated.

Production exporter/atom/family/full-JSON derivation identities are unchanged from
merged #178; no raw production CUE, production JSON or generated Rust changes.
The exporter production fileset still contains exactly eight files (Cargo metadata
plus six Rust files), excluding all experiments, tests and these raw policy files.
Both full native graph proofs retain 3392 build paths without pinned Go CUE/source/
JSON identities. Separate oracle/downstream verifier derivations change with the
Rust test package and are not attested as Go-CUE-free production inputs. The held
exporter prototype remains separate and is not an input to this milestone.
