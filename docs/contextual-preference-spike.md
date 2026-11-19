# Context-partitioned preference hypothesis

## Reviewed research contract

Milestone [#188](https://github.com/kolohelios/cuestacean/issues/188).

Follow [the rejected transparent profile](transparent-preference-fragment.md).
Investigate a NEW, independent, test-only finite scalar hypothesis, not a repair
of the existing solver or permission to admit nested production defaults. Two
focused reviews tested the algorithm below with independent pinned CUE probes
before implementation and approved research only. They found additional genuine
both-order falsifications. The frozen candidate remains a research result, not
semantic/admission approval; do not repair it by excluding failing contexts.

All prior APIs, gates, assertions, falsifications and work thresholds remain
unchanged. #175 and its separate prototype stay held. No production parser,
exporter, policy artifacts, generated Rust/JSON, dependencies or raised guards.
The algorithm below is native research, NOT CUE-owned solver semantics.

## Candidate algorithm to review

Use exact i64 source leaves, conjunction and grouped/marked Or, including marked
ancestors. Reject source Top/Int for this experiment rather than pretending to
solve the separately pinned abstract/order cases. Reuse Expr/Domain/Preference/
Outcome/Error data types only; do not invoke an old solver/demand as a fallback.

1. Compile immutable source occurrences and syntactic recipe presence. Retain
   complete ordinary worlds with branch-choice lineage; never merge equal-valued
   occurrences. No source rewrite erases group or preference origin.
2. A partitioning group is an UNMARKED Or with at least one direct child that
   contains a syntactic recipe. Its alternative choices, not those of flat marked
   groups, contribute to a context signature. This is a specified hypothesis,
   not a certificate or a proven general normalization law.
3. Partition surviving ordinary worlds by the exact intersection of their lineage
   with all partitioning choice bits. Unentered partition groups contribute zero.
   Deduplicate context signatures, not scalar worlds. Contexts are disjoint and
   cover the entire ordinary world table; no Cartesian product beyond saved worlds.
4. Within EACH context, evaluate source recipes over that context's whole world
   table. Crucially, do NOT condition on every individual marked choice: flat
   conflicting live defaults must remain present-bottom, not independently revived
   values. At And, absent is identity and present sets intersect.
5. At a marked Or, first inspect surviving marked VALUE support in this context.
   If any survives, retain the original syntactic M1/M2/M3 behavior. M2 never turns
   into M1 merely because an inherited recipe becomes absent. If NONE survives,
   the candidate instead inherits child recipes as an unmarked union, with no
   value-world default invented from an eliminated mark. This conditional step
   must be falsified independently; it is not blanket M3 recipe resurrection.
6. Unmarked Or unions inherited child associations. Retain Absent versus present
   empty/Bottom. Union context preference associations: absent is identity; Bottom
   is a present empty contribution. Only final reports deduplicate scalar values.
7. Prefer the nonempty combined preferred domain; otherwise project ordinary
   constraints. Compatible distinct values remain Incomplete; no first-value rule.
   Ordinary emptiness is Conflict. Each call owns fresh state and one work budget.

The intended distinction from the old static hypothesis is explicit context
normalization before preference intersection, plus conditional marked-support
normalization. Neither rule is established by the examples it was designed to
explain. Do not characterize the new algorithm as correct simply because it
matches those motivating examples or one operand order.

## Motivating observations and mandatory controls

- `(*1|2)&(1|2|*3)` -> 1 both orders.
- `(*2|3)&((*1|2)|3)` -> 2 both orders.
- `((*1|2)|*3)&(*2|1)` -> pinned CUE Incomplete both orders; old hypothesis 2.
- `((*3|1)|3)&((*1|2)|(*3|2))` -> pinned CUE Incomplete both orders; old 3.
- `(*1|2)&(*2|1)` stays Bottom/Incomplete in the new hypothesis, not Absent or
  spurious independently revived preferences. This is a joint-live flat control.
- `((*1|2)|*3)` alone must still prefer 3. A marked group's own alternatives may
  not eliminate its mark by selecting one of its unmarked alternatives as context.
- `((*1|2)|*3)&(1|2)` is Incomplete in pinned CUE BOTH orders, but conditional
  inheritance predicts preferred 1 / Concrete(1), another genuine unsafe result.
  A value-dead outer mark does not by itself authorize reviving an inner recipe.
- Preserve nested M2 retention, unentered/equal-valued siblings, late/dead marks,
  ordinary conflicts, refinements by conjunction, repeated solves and limit recovery.
- Keep ALL separately pinned asymmetric CUE outputs distinct. Record independent
  new-hypothesis outcomes rather than imitate order or assert blanket equivalence.
- Fresh review probe: `(*((*1|2)|3)|(*4|5))&(1|4)` exports 1 in pinned CUE in
  BOTH orders. Independently confirmed candidate execution revives 4 in the
  context outside the marked child and aggregates preferred 1/4 -> Incomplete.
  Preserve this genuine regression versus the old hypothesis, not an exception
  excluding partition groups below marked ancestors after the fact.
- Independent review found `(((*1|2)&(*2|1))|1)&(*1|2)` is Incomplete in pinned
  CUE in BOTH orders. The candidate predicts Concrete(1): one context is Bottom
  over 1/2, the other prefers 1. This is an UNSAFE concretization, not merely a
  conservative coverage gap. Preserve both context states and the genuine failure.
- Separately the same marked-parent left operand conjoined with `(1|2|3|4|5)` exports 1 forward
  but Incomplete reversed. Preserve that asymmetry; do not label it both-order.
- A fresh independent bounded exploration must attempt additional both-order
  falsifications, not just replay old cases. Preserve any discovered gap in
  durable tests/docs and keep production/readiness integration unauthorized.

## Proposed isolated interface and ownership

New private spike `bootstrap/spikes/context_preferences.rs`, imported only by new
`bootstrap/executor/tests/context_preferences.rs`. The integration crate declares
`mod default_provenance` for the shared data types and reference evidence.

```rust
// context_preferences::solve(expression: &Expr, work_limit: usize)
//     -> Result<ContextReport, Error>
// ContextReport { constraints: Vec<Domain>, preferred: Preference,
//     outcome: Outcome, contexts: Vec<ContextFact>, work: usize }
// ContextFact { choices: Vec<(usize, usize)>, constraints: Vec<Domain>,
//     preferred: Preference }
```

Choice pairs are (postorder source node ID, source-order alternative index),
scoped to one compiled input; no identity claim across reordered syntax. Sort
pairs by node ID and contexts lexicographically by their pair vectors (an empty
key sorts first). Domain reports sort ascending i64 and deduplicate only there.
Reports derive Debug/Eq/PartialEq. A >4096 allowance rejects before compilation;
otherwise structural admission precedes charged execution (unsupported source
may therefore reject before zero-work exhaustion). The report is
research evidence, not a demand/readiness certificate. It cannot be passed to
production or used as an error-triggered retry. Source grouping stays explicit.

Kernel ownership: new spike only. Evidence ownership: new integration target only.
Coordinator: docs, Nix no-default target, integration/final validation. Existing
private-module APIs must be exercised by the new test target for strict linting,
not made public or suppressed. Stable merged-main workspaces; integrate immutable
commits, never mutable coordinator-parent rebases. At most two workers.

## Bounds and evidence

Keep 128 source occurrences, 16 levels/groups/alternatives, 64 worlds per table/raw
product BEFORE filtering, 64 contexts (a partition of existing worlds), and 4096
shared cumulative execution work. Topology admission is separately bounded before
compilation. Admission/compilation are separately structurally bounded preparation;
execution charges one entry step before compilation, not per metadata node.
Caller-owned Expr destruction is not hardened wire-input handling.
Unsupported source/profile/arity failures are Unsupported; structural/work/world
limits are Limit. Zero work exhausts; >4096 allowance rejects; no partial report.

Charge before ordinary-world products/copies, context signature scans/comparisons/
allocations, subgroup support/recipe traversal, domain projection/dedup/copies and
report materialization. Do not reset work per context or silently delegate an
expensive context elsewhere. Work counters do not claim allocator-byte, stack or
wall-time hardening. Each context still obeys existing world bounds.

Use complete handwritten expected context/domain/state outcomes, mandatory fresh
pinned CUE processes with explicit raw schema/program files, both operand orders,
separate Conflict/Incomplete, and exact/one-below work recovery. New context facts
must show why motivating outcomes differ without seeding algorithm inputs from
old reports, expected answers, generated exports or oracle results.

After focused cross-reviews: four profile/feature combinations with strict focused
Clippy, both full Rust suites/default all-target Clippy debug/release, pinned
preflight, all nine actual Nix checks and five native/export package builds/rebuilds.
Keep fresh seeds/features/drift/twelve builders/source/value oracles. All five
constructor identities/four policy hashes and exact eight/six source cuts remain
unchanged; inspect recursive construction/build graphs, not merely runtime refs.
Pinned CUE v0.16.1/source 6d609d768f1686f9a3a2a20197cacdbb70e5c79d; Linux x86_64 only.
Stay near 1500 changed lines. No general contextual calculus/production/bootstrap
closure or performance claim; the known genuine failures block semantic admission.

## Findings and readiness decision

The candidate explains the two older genuine both-order failures by different
mechanisms: a context union keeps preferred 1/3 in the transparent-wrapper case,
and conditional inheritance keeps conflicting defaults Bottom in the earlier
marked-ancestor case. It also preserves the flat joint-live Bottom control.
These successes are not a corrected contextual calculus.

Three independently pinned genuine failures block admission:

| Expression (and reversed) | Context hypothesis | Pinned CUE, both orders |
| --- | --- | --- |
| `(((*1|2)&(*2|1))|1)&(*1|2)` | preferred 1 / Concrete(1) | Incomplete |
| `(*((*1|2)|3)|(*4|5))&(1|4)` | preferred 1/4 / Incomplete | 1 |
| `((*1|2)|*3)&(1|2)` | preferred 1 / Concrete(1) | Incomplete |

The first loses a Bottom contribution when unioning it with another context's
preferred 1. The second partitions inside a marked child, then revives an outside
recipe in a context where that child is unentered. The third shows that even
eliminated marked-value support without partitioning cannot authorize inheritance.
These identify failed hypotheses, not established replacement rules. In particular,
the first and third publish unsafe concrete values; this is not merely a
conservative coverage gap. Do not repair the frozen experiment with row-specific
exceptions or reinterpret the source grouping.

The correctly marked M2 probe `(*(*3|1)|2)&(1|2)` is separately asymmetric:
Incomplete forward, 1 reversed; the candidate remains Bottom/Incomplete. The
similar unmarked source `((*3|1)|2)&(1|2)` is a DIFFERENT expression and must not
stand in for this M2 evidence. Existing pinned asymmetries remain order-specific,
not genuine both-order failures or instructions to imitate evaluation order.

Decision: the new ContextReport is inspectable research evidence only, not a
readiness certificate or input to production. Old Graph APIs and all conservative
certificates remain untouched; no error-triggered retry, CUE-owned semantic claim,
policy-provider change or authorization to unhold #175 follows from this work.

## Review and validation results

Both focused contract reviews preceded implementation and independently pinned
new failures rather than authorizing semantic admission. Implementation stayed
frozen after falsification. Cross-review covered shared work, pre-copy bounds,
context sorting/dedup, M2 presence and aggregate-before-fallback behavior. Evidence
review caught an oracle snippet missing its outer mark; the exact marked M2
source and BOTH pinned orders were corrected before immutable integration. No
kernel correction was made to fit an oracle row. Final reviews had no blockers.

Immutable inputs, both parented on merged `b3d2b294`:

- Kernel `d4b9c4beb964b51610126d2ef7507cff54ebe533` (new spike only).
- Evidence `a8d20b57e0635a8cf987fdd518d0d9294e98d7a9` (new target only).

All created workspaces were retired. The six new groups pass debug/release ×
default/no-default, with strict focused Clippy, alongside all 56 retained groups.
The committed matrix has 54 observations (three renamings, nine cases, both
orders), each checking new hypothesis, unchanged old result and independent CUE
expectations separately. Additional exact context/M2/asymmetry/resource probes
are not a claim that the whole exploratory generator corpus matches CUE.

A six-wrapper input has 64 compatible ordinary worlds and distinct context keys;
it safely exhausts shared work. This is NOT a successful 64-context report or
permission to increase the allowance. Exact/one-below recovery and a successful
64-world single-context case are separately pinned.

Both complete Rust suites/default all-target Clippy pass debug/release. Pinned
preflight passes 13/13 (7 inapplicable skips). All nine Nix checks and five native/
export packages were actually built and sandbox-rebuilt, retaining fresh six
compilers, profiles, twelve builders, drift and independent source/value oracles.
Only Linux x86_64 is validated.

All five constructor/export derivation identities and four policy hashes match
merged #186; the exact eight exporter files and six raw policy files are unchanged.
A fresh recursive policy-constructor graph has 1407 derivations and 3392 full
build paths, independently matching `--requisites --include-outputs` and the
verifier's retained paths. Pinned CUE/source, full production JSON and current
downstream executor are absent from construction, not the separate verifier/
consumer closure. No new kernel/test/expected answer enters native construction.

Focused reproduction:

```sh
nix develop ./infra/tooling --command cargo test \
  --manifest-path bootstrap/executor/Cargo.toml --test context_preferences
```
