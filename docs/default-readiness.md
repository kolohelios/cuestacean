# Contextual preference lifetime and demand readiness

## Reviewed contract

Two focused kernel/evidence reviews preceded implementation. The private-module
lint integration finding was resolved with additive API-exercise tests, not a
visibility escape or suppression.

Milestone [#178](https://github.com/kolohelios/cuestacean/issues/178). Follow-up to [#176](default-provenance-spike.md), while production default export
[#175](https://github.com/kolohelios/cuestacean/issues/175) remains held. This slice
adds a **test-only conservative demand boundary**, not another guessed default
calculus or permission to integrate the held exporter. Production parser, eager
evaluator, exporter fileset and generated artifacts stay unchanged.

The previous scalar hypothesis retains ordinary-world lineage, but selects `2`
for `((*1|2)|*3)&(*2|1)` where pinned CUE is Incomplete in both orders. Suppressed
nested recipe lifetime is therefore an unresolved semantic dependency. Do not
repair it by blindly restoring M3 recipes, discarding every conflicting preference,
or following operand order. The existing hypothesis and its falsification tests
remain unchanged and available as diagnostic evidence, never a fallback.

## Explicit residual facts

Extend only the independent scalar Graph in `bootstrap/spikes/default_provenance.rs`.
Use the existing Top/Int/i64/And/grouped-Or IR, immutable occurrence IDs, complete
ordinary worlds, source recipes and structure/work limits. No source parser,
references, collections, arithmetic, general cycles, incremental solver or CUE-
hosted policy implementation is introduced.

Expose all Or-group facts after ordinary contextual refinement, including groups
inside suppressed or impossible branches. A group occurrence and its child recipe
IDs remain stable across observations of the **same immutable graph**, not across
rebuilt/reordered graphs. Repeated spelling/value never identifies an occurrence.

```rust
pub enum Support { Unentered, Unmarked, MarkedLive, MarkedEliminated }
pub struct GroupFact {
    pub group: usize,
    pub support: Support,
    pub retained: Vec<usize>,
    pub suppressed: Vec<usize>,
}
pub enum Readiness { ConstraintOnly, FlatPreference, DeferredNested }
pub enum Demand { Concrete(i64), Incomplete, Conflict, Suspended }
pub struct DemandReport {
    pub constraints: Vec<Domain>, pub groups: Vec<GroupFact>,
    pub preferred: Option<Preference>, pub readiness: Readiness,
    pub demand: Demand, pub work: usize,
}
// Graph::demand(refinement: Domain, work_limit: usize<=4096)
//     -> Result<DemandReport, Error>
```

Derive Debug/Eq/PartialEq for public audit enums/report; use the existing Domain,
Preference and Error. Do not change existing Graph::solve or its Report/Outcome.
`preferred: None` means **not evaluated**, never an Absent association; evaluated
preferences remain Some(Absent/Bottom/Values). Domain report ordering/dedup follows
#176, retaining subsumed domains. Facts sort by group occurrence ID; recipe ID
lists follow source alternative order and never conflate equal values.

- A group's ordinary context is visible only in worlds with all ancestor choice
  bits. No such worlds means Unentered, even if an equal value exists elsewhere.
- An entered unmarked group is Unmarked. An entered marked group is MarkedLive
  iff at least one marked **value** branch survives; otherwise MarkedEliminated.
  A contradictory preferred intersection alone cannot mean eliminated support.
- `retained` lists direct child roots with a syntactic recipe that are inherited
  by an unmarked group or marked terms of a marked group (M2).
- `suppressed` lists unmarked direct child roots with a syntactic recipe in a
  marked group (M3). Preserve these IDs even when the child/group is impossible;
  they are dormant source dependencies, not a directive to reactivate defaults.
- Syntax/recipe presence and contextual support stay separate. Report facts for
  every group, including unentered groups, without evaluating suppressed recipes.

## Conservative export-demand decision

Each call recomputes fresh ordinary worlds from the immutable graph, then meets
its explicit scalar refinement against every world before auditing support. It
is a new observation, not mutation of a prior projection or an incremental edit.
A later broader observation must still see the original worlds/recipes.

Decision precedence, after all bounded factual auditing:

1. No ordinary worlds: ConstraintOnly/Conflict, preferred None.
2. Exactly one deduplicated exact i64 domain, with no abstract domain: ordinary
   constraints force the value independently of preferences. Return
   ConstraintOnly/Concrete(value), preferred None. This rule is only for the
   ready scalar constraint IR, not the suspended arithmetic/schema cases.
3. No syntactic recipe anywhere: ConstraintOnly/Incomplete, preferred None.
4. If **any entered Or has a direct child with a syntactic recipe**, nested
   preference policy is deliberately unresolved. Return
   DeferredNested/Suspended, preferred None. This conservative rule may defer
   cases CUE or the old hypothesis can resolve; it is not a minimal classifier.
   Record retained/suppressed recipes rather than silently erasing them.
5. Otherwise use the existing flat preference calculation in the **same remaining
   work allowance**. Return FlatPreference with Some(preferred) and its concrete,
   incomplete or conflict demand. Absent and present-bottom remain distinct.

A single concrete candidate alongside Int/Top or another candidate is not forced.
Do not compute/report a provisional nested value and then label it suspended.
Unsupported/Limit propagate directly; no fallback, error-triggered second engine,
budget reset, default-policy choice by label/value, or silent oracle delegation.
The readiness names describe this experimental decision path, not general CUE
readiness or production export admission. No maximum-coverage claim is made.

## Reviewed boundaries and evidence required

Complete two focused contract reviews before implementation. Kernel owner:
`bootstrap/spikes/default_provenance.rs` only. Test owner:
`bootstrap/executor/tests/default_readiness.rs` only. Coordinator owns docs/Nix,
a small additive API-exercise test in the old integration target, immutable-commit
integration, production-cut inspection and final validation. The separate old
and new test crates include the same private spike module: both must exercise
both APIs to retain strict dead-code linting, without suppressions or deleting
any old assertions.
Keep workers on stable merged main, not the mutable coordinator parent.

The old spike's 128 occurrences, 16 groups/levels/alternatives, 64 worlds per
intermediate table/raw product and 4096 cumulative work remain unchanged. New
refinement, support/audit scans, ID copies, report allocation and flat projection
must be charged before allocation/enqueue in that same allowance. Topology
construction remains separately structurally bounded; caller-tree destruction is
not a hardened wire boundary. No raised guards, stack overrides or lint suppression.

Tests must retain all #176 assertions and independently check:

- Both-order counterexample yields Suspended, not the old hypothesis's `2` or
  a false Conflict/Incomplete. Pinned CUE remains Incomplete in both orders.
- Top/int/exact refinements expose live/eliminated/unentered support and stable
  dormant/retained recipe IDs. Refining to 1 or 2 forces that scalar; incompatible
  3 conflicts. Subsequent Top observation recovers original residual facts.
- M2 syntactic retention, M3 suppression, sibling visibility and equal-spelling
  occurrences, including unentered subgraphs; no blanket preference reactivation.
- Flat original support-elimination case remains independently concrete; conflicting
  flat preferences remain present-bottom/Incomplete, not eliminated/absent.
- Nested known oracle order discrepancies are deferred, not matched by choosing an
  order. Broader nested inputs may also defer intentionally; pin that cost.
- Abstract readiness, forced-value precedence, report determinism, repeated calls,
  work/product limits, failure recovery, both operand orders and a bounded matrix.

Use mandatory pinned CUE v0.16.1 source `6d609d768f1686f9a3a2a20197cacdbb70e5c79d`
with independent expected classifications, explicit schema/program files and fresh
processes. Suspended is a native **not-established** classification, not a claim
that CUE reports Suspended or cannot export. No copied upstream implementation.

Require focused debug/release × default/no-default tests, both full Rust suites and
strict Clippy, full pinned preflight, all eight actual Nix checks/rebuilds, six
artifact-free compiler binaries, feature isolation and independent source/value
checks. Add the new test to the explicit no-default sandbox list. Check unchanged
native production derivation identities/fileset and JSON/generated Rust. Record
x86_64-linux scope honestly. A later production/contextual policy proposal still
needs its own reviewed semantics; this boundary alone cannot unhold #175.

## Findings and decision

The demand boundary now exposes the unresolved decision rather than publishing
an experimental nested value. In both operand orders the #176 counterexample
retains ordinary candidates 1/2 and source recipe facts but returns
DeferredNested/Suspended with preferred None. The old hypothesis still returns
its falsified `2` through the unchanged solve API; the independent tests preserve
that result and both pinned CUE Incomplete outputs. No evidence was overwritten.

Refining the same graph to 1 or 2 forces that ordinary scalar, independently of
preference policy; incompatible 3 conflicts. All source group facts remain
available even then, including Unentered groups and suppressed recipe IDs. A
subsequent Top observation restores the original broader facts and suspension:
there is no destructive default assignment or cross-observation cache. Equal
values from sibling/unentered occurrences cannot confer support on each other.

Flat preferences remain distinguishable from suspension: the original flat
support-elimination gap selects 1, while the conflicting-preference control has
Some(Bottom)/Incomplete. None means policy not evaluated, not an absent or bottom
preference. Abstract candidates never become forced merely because one concrete
candidate also exists. Audit and projection share one work allowance, including
forced/conflict observations; exact refinement cannot bypass raw product caps.

This boundary deliberately defers the previously resolvable nested example
`(*2|3)&((*1|2)|3)` even though pinned CUE exports 2 in both orders. That coverage
cost is explicit: classification is conservative and not minimal. All three known
pinned order discrepancies are also deferred rather than imitated. Suspended is
our not-established policy state, **not** a claim about CUE's output category.

Decision: this provides a usable test-only constraint/support versus preference-
policy boundary, not a solution for nested preference lifetime or a production
solver interface. #175 remains held. A future policy proposal must explain
contextual suppression/reactivation and readiness, preserve these counterexamples
and resource limits, and name which executable rules live in CUE versus native
kernels. Do not treat a Suspended result as an error to retry through another
engine or use this conservative classifier as full CUE export admission.

The [#180 executable dispatch-policy experiment](demand-policy.md) reuses only
predecision observation and checked primitives. Its CUE-authored branches select
action requests; native certificates constrain execution. The native demand path
remains reference evidence, not a fallback, and nested semantics remain held.

The [#186 transparent-wrapper study](transparent-preference-fragment.md) tests
whether concrete-only sources with no marked recipe ancestor could safely relax
this gate. It finds a genuine both-order falsification with all marks live and
no suppressed recipes: `((*3|1)|3)&((*1|2)|(*3|2))`. Thus empty suppressed lists
or the absence of nested M3 stripping are not a sufficient readiness certificate.
This study adds evidence only; the conservative gate and all prior results stay
unchanged, including deliberately suspended compatible controls.

## Validation and unchanged production cut

On x86_64-linux, all 14 new test groups and 13 old groups pass debug/release under
default/no-default features, with strict focused Clippy in both profiles. The old
12 hypothesis groups/assertions are intact; one additive API test keeps both
private-module integration targets under the unchanged dead-code lint policy.
Focused reviews closed prohibited unwrap calls and exact native/oracle alternative
ordering; unentered sibling retention is pinned explicitly. No lint suppressions
or visibility escapes were added.

Both full Rust suites pass debug/release and strict all-target Clippy. Pinned
preflight passes 13/13 (7 inapplicable checks skipped). All eight Nix checks and
the native exporter/atom/family pass actual sandbox rebuilds; the new integration
target is explicitly exercised without default features. Six fresh artifact-free
compiler binaries, feature isolation, regeneration/drift and independent source/
value evidence remain intact. Other architectures are omitted, not tested.

Production exporter/atom/family/full-JSON derivation identities are unchanged from
merged #176. The native source allowlist remains Cargo metadata plus six Rust files
(eight total), excluding both experiments and tests. The full native proof graphs
retain 3392 build paths, excluding pinned Go CUE/source/JSON identities. Separate
oracle/downstream verification derivations change with the Rust test package;
those checks are not attested as Go-CUE-free production inputs. No raw CUE,
JSON-export or generated Rust changes occur. The held exporter prototype is not
an input or a merge candidate for this slice.
