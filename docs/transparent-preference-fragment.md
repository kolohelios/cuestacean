# Transparent nested preferences: admission study

## Reviewed research boundary

Milestone [#186](https://github.com/kolohelios/cuestacean/issues/186), following
[scoped projection](preference-projection-policy.md). Two focused reviews preceded
regression implementation; they rejected the proposed admission. This is a **test-only characterization**, not
a new demand API, preference algorithm or permission to unhold #175.

Investigated hypothesis: the existing static association recipe might admit
nested demands safely when all source leaves are exact i64 and every marked Or
has no direct child containing a syntactic preference recipe. Thus nesting above
recipe-bearing children is entirely unmarked. Apply this candidate predicate to
the whole syntax tree, including unreachable branches; source Top/Int are outside
it. A request refinement is not a source atom and does not change eligibility.

The rationale was that unmarked wrappers inherit recipes without syntactic
nested M2/M3 suppression. However, excluding marked ancestors does NOT establish
correct preference lifetime: review found a genuine both-order counterexample
inside this profile, independently confirmed with the unchanged Rust hypothesis
and pinned CUE before any certificate/API change. A candidate predicate in tests is
only a way to identify the investigated cases, never production admission.

## Independent counter-observations

The proposed profile contains these examples:

| Left & right (and reversed) | Static hypothesis, both orders | Pinned CUE forward | Pinned CUE reversed |
| --- | --- | --- | --- |
| `((*1|2)|(*2|1)) & (*1|2)` | preferred 1 / Concrete(1) | 1 | Incomplete |
| `(((*1|2)&(*1|3))|3) & (*3|1)` | present-bottom / Incomplete | 3 | Incomplete |
| `((*3|1)|3) & ((*1|2)|(*3|2))` | preferred 3 / Concrete(3) | Incomplete | Incomplete |

For the first, ordinary constraints are 1/2; the unmarked wrapper inherits both
child preferences, and meeting the sibling preference leaves 1 in the static
hypothesis. For the second, ordinary constraints are 1/3; the wrapper's preference
1 conflicts with the sibling's 3, so the static hypothesis remains bottom, not
absent. Equal scalar values do not erase occurrence identity.

The first two rows are **pinned order discrepancies**, not genuine both-order
falsifications. Do not relabel them, imitate one operand order, silently discard
a preference, or call the static hypothesis correct because one order agrees.

The third row IS a genuine both-order falsification. Ordinary constraints are
1/3 and every mark remains live; all suppressed recipe lists are empty. The
13-occurrence example is not claimed globally minimal. The left
wrapper prefers 3; the right wrapper inherits 1/3, so static intersection selects
3. Nevertheless pinned CUE remains Incomplete in BOTH orders. Thus the gap is not
confined to explicit nested M3 stripping or abstract atoms. This does not establish
a replacement contextual algorithm or license blanket recipe reactivation. The
older genuine [#176 falsification](default-provenance-spike.md) remains unchanged.

The known `(*2|3)&((*1|2)|3)` control exports 2 both ways and the static hypothesis
also returns 2. Existing readiness APIs deliberately suspend that nested case.
One compatible example does not authorize the whole candidate profile.

## Decision: no broader certificate

Do NOT implement the proposed `demand_in_transparent_fragment` API or cache a
structural eligibility bit in Graph. Do not add a return-flat policy/provider
artifact, reinterpret the `nested` fact or broaden any old flat certificate.
The candidate excludes syntactic nested suppression but is falsified by the
third row: this static recipe still lacks necessary contextual default semantics. A future narrower profile or corrected
contextual algorithm needs a separate reviewed proposal, not a mid-test exception
chosen to remove failing rows.

Preserve solve/native/single-policy/two-policy paths, original work thresholds,
all old assertions, conservative suspension and #175's held prototype. Reports
must distinguish None (preference not evaluated), Absent, Bottom and Values.
Graph observation remains immutable/fresh; no error-triggered second engine or
Go CUE fallback. Pinned CUE remains a separate mandatory oracle.

## Regression contract and ownership

Evidence owner: new `bootstrap/executor/tests/transparent_preferences.rs` only.
Coordinator: docs, the Nix explicit no-default target and final integration.
Semantic reviewer: read-only cross-review and independent scratch validation.
Use immutable commits parented on merged `c0d9504e`; no mutable-parent rebases.
No spike kernel, exporter, raw CUE, dependency or generated artifact changes.

The new target imports the private spike and exercises all existing APIs,
including scoped policy constructors, without suppressions/visibility escapes.
Use an explicit candidate-syntax predicate in tests and independently authored
Expr trees and CUE snippets; neither is parsed from the other's answer. Required:

- All table rows, both orders, exact ordinary/preferred/native outcome reports;
  distinguish the genuine both-order falsification from pinned asymmetries.
  For the genuine case pin live marks, retained IDs and empty suppressed lists.
- Native demand -> DeferredNested/Suspended/None; canonical dispatch and two-phase
  demand -> PolicyDeferred/Suspended/None. Unsafe flat -> Unsupported, not fallback.
- Concrete refinements force scalars independently; incompatible refinements
  conflict; broader observations restore original group/recipe facts and suspension.
- Eligibility rejects abstract source atoms and marked ancestors with nested
  recipes, even in impossible branches. This is characterization, not a new API.
- A deterministic bounded value-renaming/permutation matrix: independent expected
  native domains and EACH oracle order, including an agreement control and flat
  present-bottom control. Never replace asymmetry with a blanket equality check.
- Old genuine both-order falsification, old abstract/order cases and all existing
  regressions stay intact. Do not claim the entire exploratory corpus is verified.
- Fresh mandatory pinned CUE processes, explicit raw source boundaries, distinct
  Incomplete versus Conflict checks and no JSON on failed exports. Check exact
  native work / one below, limit recovery and repeated observations.

Existing caps remain: graph 128 occurrences, 16 levels/groups/width, 64 worlds/raw
products and 4096 shared work; policy 16 KiB/64 nodes/16 levels. No raised guards,
stack overrides, global caches or lint suppressions. Caller-owned IR destruction
and policy parsing are not advertised as hostile-input hardening.

## Validation and production independence required

Run focused debug/release × default/no-default and strict focused Clippy; both
complete Rust suites and all-target default Clippy, debug/release; pinned Shaka
preflight and all nine actual Nix checks/builds/rebuilds. Keep six fresh compiler
binaries, feature isolation, generation/drift, twelve builders and independent
source/value oracles. Linux x86_64 only.

All FIVE existing constructor/export derivation identities must remain unchanged:
exporter, atom, builder family, full production JSON and native demand policies.
Policy hashes, exact eight-file exporter and six-file raw policy cuts stay intact.
Only downstream tests/verifiers change; inspect recursive derivation/build graphs,
not just runtime references. No generated JSON, expected tree or test corpus is
an input of native construction. Separate verifier/consumer closures still use
Go CUE; this study makes no new bootstrap-closure or performance claim.

## Review and validation results

Both focused reviews rejected semantic admission before implementation. An
initial wrapper-versus-flat search exposed the two asymmetries; a subsequent
wrapper-pair search found the genuine third row. Independently authored Rust
Expr trees and fresh pinned CUE export/eval processes confirmed it in both orders.
Exploratory generators are not a verified compatibility corpus or production
input. The committed matrix separately checks 60 observations: six value
renamings, five cases (including two controls), and both operand orders.

Read-only cross-review verified the exact genuine-case occurrence IDs, retained
recipes, live marks, empty suppression lists and unchanged four API boundaries.
A separate no-default run passed all six new groups. Reverse docs/Nix review
closed a three-row wording nit; no semantic blockers remained. The immutable
evidence commit is `803fce58d6c676c204c3f69c2b2c92f040bc22e4`, parented on
`c0d9504e`; only the new integration target was changed. Created review workspaces
were retired after integration.

All 6 new plus 50 retained executor groups pass debug/release ×
default/no-default, with strict focused Clippy in each combination. Both complete
Rust suites and all-target default Clippy pass debug/release. Pinned preflight
passes 13/13 (7 inapplicable skips). All nine Nix checks and five native/export
packages were actually built and sandbox-rebuilt, retaining fresh compilers,
profiles, builders, drift and independent source/value checks. Linux x86_64 only.

All five constructor/export derivation identities and four policy JSON hashes
match merged #184 exactly. The eight-file exporter and six-file raw policy cuts
were inspected independently. The unchanged policy constructor's fresh recursive
graph has 1407 derivations; its 3392-path full build closure matches an independent
`nix-store --query --requisites --include-outputs` query and excludes pinned CUE/
source, full JSON and the current downstream executor. The separate verifier
retains build paths, roots and registration, not a Go-CUE-free consumer claim.
No native construction consumes this test target or its expected answers.

Reproduce the focused evidence with:

```sh
nix develop ./infra/tooling --command cargo test \
  --manifest-path bootstrap/executor/Cargo.toml --test transparent_preferences
```

Production defaults remain held. A new readiness policy must explain this
both-order, live-support/no-suppression counterexample, not only the earlier
marked-ancestor case or the separately pinned order discrepancies.
