# Residual default-provenance spike

## Reviewed contract

Milestone [#176](https://github.com/kolohelios/cuestacean/issues/176). Two focused
kernel/evidence reviews preceded implementation. User-authorized follow-up to the held [#175 default exporter](https://github.com/kolohelios/cuestacean/issues/175).
This is a **test-only native scalar experiment**, not production default support,
a general solver, a CUE-hosted implementation or permission to merge the held
exporter. Work starts from merged #173; the prototype remains separately saved at
local bookmark `work/native-default-domains`, commit `d416628dcad9f7061736c7e1b0c4efe2fa9bd1ed`.
No production parser/evaluator or native exporter source allowlist changes.

The blocker is not just a contradictory preferred domain. Pinned CUE returns `1`
for `(*1|2)&(1|2|*3)` in **both** operand orders and `2` for
`(*2|3)&((*1|2)|3)` in both orders. Eager value/default pairs return Incomplete.
The control `(*1|2)&(1|*2)` is Incomplete in both tools/orders: both marked value
supports survive, so contradictory preferences are present-bottom, not absent.

Separately, pinned `(*1|2)&((*1|2)|*3)` exports `1` forward but Incomplete reversed;
`(*int|1)&((*1|2)|3)` is Incomplete forward but `1` reversed. These are independent
oracle order discrepancies, not excuses to dismiss the genuine both-order gaps.
Do not imitate operand-order behavior or claim all oracle comparisons match.

## Fragment, identity and ownership

Use an independent hand-built scalar IR under `bootstrap/spikes`, imported only
by an integration test. No CUE parser, production module dependency, fallback,
field references, definitions/imports/bindings, structs/lists, arithmetic or cycle
solver. Atoms are top, int and exact i64; a finite union/meet of these constraints
is sufficient for the questions here. CUE source snippets are separate mandatory
pinned oracles, not inputs to this IR or the production parser.

Proposed shared interface (agree before concurrent implementation):

```rust
pub enum Expr {
    Top, Int, Value(i64),
    And(Box<Expr>, Box<Expr>),
    Or(Vec<(bool, Expr)>), // direct mark flag; each Or is its own syntax group
}
pub enum Domain { Top, Int, Value(i64) }
pub enum Preference { Absent, Bottom, Values(Vec<Domain>) }
pub enum Outcome { Concrete(i64), Incomplete, Conflict }
pub enum Error { Unsupported(&'static str), Limit(&'static str) }
pub struct Report {
    pub constraints: Vec<Domain>, pub preferred: Preference,
    pub outcome: Outcome, pub work: usize,
}
pub struct Graph { /* immutable occurrence arena */ }
// Graph::new(Expr)->Result<Graph,Error>
// Graph::solve(work_limit:usize<=4096)->Result<Report,Error>
```

Derive Debug/Eq/PartialEq for inspectable enums/report and Clone for Expr/Domain.
Reports sort domains as Top, Int, then ascending i64 values; deduplicate identical
domains only, retaining subsumed `[Int, Value(1)]` in the report. Internal worlds
are never deduplicated by scalar value or merged lineage. `Values` is nonempty;
ordinary contradiction reports empty constraints, Absent preference and Conflict.
Reject empty/unary Or as Unsupported; depth is root-inclusive. Zero work exhausts
before solving, and allowances above 4096 reject.

Compile the finite tree to checked occurrence IDs. A repeated spelling/value is
not a shared preference identity. Parenthesized/nested Ors stay separate groups;
consecutive terms of one Or are a single group. Caller-owned tree construction/
destruction is not an adversarial wire-input boundary. Arena IDs are non-owning,
graph-scoped; each solve owns fresh residual/projection/accounting state. No
cross-solve value cache or mutable reference graph in production collections.

## Hypothesis to test, not established full CUE semantics

Retain complete compatible scalar worlds before projection, including their
branch-choice lineage. Give each Or alternative an occurrence choice bit; context
visibility requires its ancestor choice bits. A mark's support is live only when
a surviving ordinary world actually selects a marked value branch of that group.
A world from a different/unentered branch cannot make that support live.

Evaluate a residual preference recipe over this saved ordinary-world set:

- Leaves have no preference. Meets combine present preferences by intersection;
  absent is identity, not an empty preferred set.
- Unmarked unions combine child preferences by union and retain their group
  provenance. A union of only absent associations stays absent; Bottom unions
  a present set as the empty set but remains present when no values survive.
- A marked group drops unmarked terms' *top-level* preference recipes (M3).
  A marked term uses its existing syntactic recipe (M2) or its value worlds (M1).
- If every marked value branch of a group is impossible in the ordinary context,
  its association is absent. This needs the saved value support, not merely a
  test of whether the preferred intersection is empty.
- M1 versus M2 is determined by **syntactic** recipe existence, not contextual
  presence. Never switch to M1 because evaluating the M2 recipe becomes Absent.
  If marked value support survives but the retained inner/default recipe yields
  no preferred worlds, keep **present-bottom**. Do not promote another conjunct's
  preference or blindly revive M3-stripped defaults.
- Only after these steps project preferred worlds, or ordinary worlds when the
  final preference is absent/bottom. Any compatible abstract preferred world or
  distinct concrete choices leaves Incomplete; no first-concrete selection.

This is an executable hypothesis. Review already found a **both-order** predicted
counterexample: `((*1|2)|*3)&(*2|1)` exports Incomplete in pinned CUE both ways,
but the hypothesis predicts `2`. Do not change the recipe merely to fit it or
relabel it as an asymmetric oracle case. The experiment must preserve this as an
independent native-hypothesis versus oracle result; static lineage/activation may
therefore prove necessary but insufficient for production readiness.

A further pinned order case is `(*((*1|2)&(1|*2))|3)&(*3|1|2)`: Incomplete forward,
`2` reversed, while the hypothesis predicts Incomplete both ways. Keep its two
outputs distinct from the both-order counterexample above.

This is an executable hypothesis. Tests may discover additional counterexamples;
record them and narrow the readiness conclusion, not the evidence. Known oracle
order discrepancies get explicit expected native hypotheses plus both pinned
outputs, not a blanket differential-equality assertion. Unsupported/Limit never
become contradiction or trigger another semantic engine.

## Bounds and validation

Pinned oracle: cue-lang/cue source commit
`6d609d768f1686f9a3a2a20197cacdbb70e5c79d`, selected by Nixpkgs
`ac588ef1c914865dad517b337ea0eb9694a4021c`. The executable reports `(devel)` and
CUE language v0.16.1; the display string alone is not source attestation. These
are independently written probes, not copied upstream test/implementation files.

Experiment caps: 128 IR occurrences, 16 syntax levels, 16 Or groups,
16 alternatives/group, 64 worlds per intermediate table/raw Cartesian batch and
4096 cumulative solve work across residual worlds, lineage copies/scans and
preference projection. The raw product cap applies before conflict filtering,
not cumulatively across batches; cumulative work remains separately bounded.
Topology compilation validates its fixed occurrence/depth caps before recursion;
its construction is not charged against a later solve's allowance. In a tree,
choice edges are a subset of nonroot occurrences, hence at most 127; a 128-bit
lineage suffices despite the independently larger group/width maxima.
Charge before worklist/product allocation or enqueue; tests may lower work only.
Bound products before allocating even if most trials would conflict. Do not
raise production guards or use stack overrides/lint suppression. These counters
are not allocator-byte/wall-time hardening or a production-ready API claim.

Kernel owner: `bootstrap/spikes/default_provenance.rs` only. Test owner:
`bootstrap/executor/tests/default_provenance.rs` only. Coordinator owns docs, Nix,
integration and final validation. Complete two focused contract reviews first.
Integrate immutable worker commits from stable merged main; never rebase workers
onto a mutable coordinator parent.

Independent evidence: both-order support-elimination cases and control above;
M1/M2/M3, late marks, unmarked nesting, multiple equal/distinct defaults, abstract
preferred readiness, top/refinement, conflicts, sibling/branch visibility,
repeated solves, structural/work/product limits and a small bounded permutation
matrix. Keep the native hypothesis and oracle-deviation classifications explicit.
Pin CUE version/source/provenance, run a fresh mandatory CUE process per probe
with explicit source-file boundaries and independent expected outcomes. Do not
use golden/generated exports as algorithm inputs.

Run focused debug/release tests under default and no-default features, full both
Rust suites/strict Clippy, pinned preflight/all eight Nix checks and actual sandbox
rebuilds. Add the spike to the existing explicit no-default sandbox test list.
Keep six fresh artifact-free compiler binaries, profiles/drift and independent
source/value checks; no generated JSON/Rust changes. Inspect production exporter
fileset and native artifact derivation identities to prove this test-only work
never enters the native production cut. Scope readiness, not performance or full
bootstrap closure, is the result to record before any later production proposal.

## Findings and readiness decision

The scalar experiment preserves complete occurrence worlds until final reporting.
It distinguishes absent preference, present-bottom and compatible preferred worlds,
and correctly handles the two original both-order elimination gaps plus the
present-bottom control. Sibling values cannot reactivate an unentered occurrence;
abstract preferred worlds remain incomplete; scalar equality is not lineage
identity. Repeated solves and recovery after resource errors use fresh state.

**This static provenance/activation recipe is insufficient.** The independent
`((*1|2)|*3)&(*2|1)` probe is Incomplete in pinned CUE in both orders, while this
explicit hypothesis selects `2` both ways. The test records native constraints,
preferred domain and outcome independently from both oracle expectations. It is
negative readiness evidence, not a passing compatibility comparison. The three
separately pinned asymmetric cases do not explain away this counterexample.

The static recipe drops M3-unmarked child recipes and makes a dead marked-value
support absent. Those operations solve the flat gaps but do not establish the
correct contextual lifetime/reactivation of nested preferences. Do not infer a
complete replacement algorithm from one counterexample, blindly restore all
nested defaults, discard present-bottom controls, or imitate pinned operand order.

Decision: keep production defaults and #175 held. A later proposal must specify
contextual residual preference lifetime and demand/readiness, with surviving and
eliminated branch provenance, before editing production. Reuse the explicit
constraint/preference separation and independent counterexamples as evidence,
not the experiment as a fallback. Collections, lexical references, schema
closedness, recursive instantiation and CUE-hosted policy interfaces remain
unestablished by this scalar experiment. No production integration is authorized
by this result. The [#178 demand-boundary follow-up](default-readiness.md) keeps
this hypothesis intact while making unresolved nested recipe lifetime explicit;
its conservative suspension is not a corrected default-selection algorithm.

The [#186 transparent-wrapper study](transparent-preference-fragment.md) finds a
further genuine both-order counterexample without abstract atoms or marked
ancestors of recipes: `((*3|1)|3)&((*1|2)|(*3|2))`. The unchanged hypothesis selects
3 in both orders; pinned CUE is Incomplete in both. Every mark remains live and
all suppressed recipe lists are empty. The missing contextual semantics cannot
therefore be reduced to explicit nested M3 stripping. No native recipe, previous
assertion or conservative certificate is changed to fit this new observation.

## Validation and production independence

On x86_64-linux, twelve test groups pass in debug/release under default and
no-default features, with strict Clippy in both profiles. They include all six
three-term permutations in both conjunction orders, mandatory fresh pinned CUE
processes, exact-i64 expectations, explicit known deviations, full
constraint/preference reporting, repeated solves and limits. The topology tests
reach 128 occurrences/16 groups/16 levels/16 alternatives, distinguish unsupported
Or arity, and reject 80-world products before conflict filtering. A compatible
64-world product and more than 64 aggregate trials across bounded batches remain
accepted. Work failures remain Limit, not Conflict. These are bounded logical
resource checks, not adversarial-input or native-stack hardening.

Both full Rust suites pass debug/release and strict all-target Clippy. Pinned
preflight passes 13/13 (7 inapplicable checks skipped); all eight Nix checks and
the native exporter/atom/family artifacts pass actual sandbox rebuilds. The new
integration test is explicitly exercised with no default features in the sandbox.
Six fresh artifact-free compiler binaries, feature profiles, regeneration/drift
and independent source/value evidence remain intact. Other architectures are
omitted, not validated.

The native production exporter, atom, builder-family and complete JSON-export
**derivation identities are unchanged from merged #173**; no raw CUE or generated
JSON/Rust changes occur. The production exporter source is still exactly Cargo
metadata plus six Rust files (eight files), excluding the spike and all tests.
Both full native graph proofs retain 3392 build paths, with pinned Go CUE/source/
JSON identities absent. Their separate oracle/downstream verification derivations
change because the downstream Rust test package changes; they are not attested
as Go-CUE-free production inputs. The held exporter prototype is not an input of
this slice or of any production artifact.
