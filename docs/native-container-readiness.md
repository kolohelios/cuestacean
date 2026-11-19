# Native container-bound export readiness

## Selection and pre-implementation evidence

After [#196 typed prefix lists](native-prefix-lists.md), optional-field feasibility
probes exposed a pre-existing native export-demand gap. Equal projected JSON is
not sufficient evidence that two container alternatives export the same CUE value.
Fix this readiness prerequisite BEFORE adding optional-field language support.
No optional/default/range/regex/import/source-parser expansion is authorized.
#175 and its separate prototype remain held; the complete scanner closure must
still reject natively as Unsupported/no JSON.

Observed on merged #196, using independently authored identical raw source pairs
and the pinned native package/Go CUE, with each expression in a `value` field:

| Expression | Current native projection | Pinned CUE |
| --- | --- | --- |
| `[1,...int] \| [1]` | `[1]` | Incomplete |
| `[...int] \| []` | `[]` | Incomplete |
| `{xs:[1,...int]} \| {xs:[1]}` | `{xs:[1]}` | Incomplete |
| `#D \| {a:1}`, with `#D:{a:1}` | `{a:1}` | Incomplete |

These are false native concretizations, not upstream-order discrepancies or a
newly accepted fragment. Both operand orders, recursive positions and controls
must be independently pinned before implementation. Keep successful controls:
identical open prefixes, different typed tails with equal open prefixes, closed
containers with equal contents, concrete scalar alternatives and refinement that
selects one bound/branch. Do not repair by changing CUE inputs, weakening tests,
raising guards or marking all projection-equal containers ready.

Milestone [#198](https://github.com/kolohelios/cuestacean/issues/198).
Two focused contract reviews approved the scoped witness and independent evidence
before explicit implementation GO. Read-only implementation cross-reviews and
full validation precede PR/merge.

## Scoped implementation hypothesis

Keep the existing public API, errors, grammar, parser/model/binding admission and
native eight-file source cut unchanged. Only the isolated native evaluator gains
private materialization metadata: successful demands retain their JSON value and
a canonical preorder witness of every materialized container's closed/open bound.

- A list is closed exactly when a fixed-list conjunct supplies its exact length;
  otherwise its known prefix is open. Selecting a fixed conjunct clears open-list
  status at that demand path, rather than retaining a stale earlier projection.
- A struct is closed exactly when normalized definition-closure admission applies
  at that demand path (`admitted` is nonempty, including an empty admitted set).
  Binding expansion alone does not close it; inherited definition closure does.
- Witness traversal is canonical JSON-container preorder: an object's own flag,
  then child witnesses in sorted field order; a list's own flag, then child
  witnesses in index order. Scalar children have no container flag. Equal JSON
  already fixes container kinds and positions; do not use allocation IDs, raw
  definition names, fields named `op` or filenames as semantic witnesses.
- Disjunction deduplicates successful alternatives only if BOTH exact JSON and
  container-bound witnesses match. Equal JSON with different bounds is Incomplete,
  not Conflict or an error-triggered solver/oracle fallback. Existing incomplete,
  distinct-output, incompatible-branch and Unsupported/Limit propagation remains.
- Preserve each path's residual constraints through conjunction/refinement before
  selecting its final bound. Do not serialize/reimport intermediate JSON, cache
  template results, mutate shared definitions or allocate a preferred/default pair.

This is a bounded readiness hypothesis for the CURRENT supported grammar, not a
certificate for future optional fields/defaults/patterns. Optional-only constraints
can differ while projecting the same JSON; pinned `{a?:1}|{a?:2}` is Incomplete,
whereas either branch alone exports `{}`. A container open/closed flag would not
distinguish those alternatives. Do NOT admit optional fields on this witness.

If independent cases falsify the scoped hypothesis, preserve that evidence and
hold the implementation rather than add row-specific exceptions or silently
expand this contract. Existing production parsing/evaluation/graph policy stays
unchanged; no semantic or performance claims beyond the reviewed isolated cut.

## Ownership, accounting and independence

Kernel owner ONLY `bootstrap/spec-export/src/eval.rs`; private result/witness
structures may live there, not in a new production module. Evidence owner ONLY
new `bootstrap/spec-export/tests/container_readiness.rs`. Coordinator owns docs,
integration, baseline comparisons, graph proofs and final validation. At most two
isolated workers on stable merged #196; exact immutable handoffs, no saved-commit
amendment or rebase onto a mutable coordinator parent.

All current hard caps remain: 64 KiB combined source/copied strings/encoded output,
4096 tokens/work, 2048 syntax/materialized occurrences, 64 definitions/bindings/
explicit items, 32 syntax/output levels. One witness entry per materialized
container is bounded by the EXISTING cumulative materialized-value allowance.
Charge witness concatenation/comparison in the shared existing work allowance;
no reset per branch, child, binding, trial or retry. Fail before publishing JSON.
No new public knobs, raised guards, stack overrides or lint suppression. Logical
limits are not comprehensive allocator/stack/adversarial-input hardening.

Author complete handwritten expected JSON and exact native classifications, then
compare mandatory fresh pinned CUE processes on identical explicit source pairs
(no root-shape setup failures as semantic evidence). Exercise both orders,
repetition/renaming, lexical aliases, productive recursion, independent template
instances, nested containers with SAME NUMBER but DIFFERENT POSITIONS of open
bounds, fixed-length refinements, mixed closed/open structs, all shared-work/depth
limits and post-error recovery. Native semantic failure CLI must have nonzero
status/stderr and no JSON under cleared environment/unusable PATH. Do not use
captured observations or old artifacts as expected answers/construction inputs.

All 12 builders, atom/policy artifacts, four policy hashes, the 249-file bundle,
committed generated Rust and existing independent answers must keep identical
bytes. Native constructor/consumer derivations legitimately change with evaluator
source; observer identity/provider scope and exact 8/6 source cuts stay unchanged.
Re-attest all three recursive native derivation/full-build graphs excluding
CURRENT full JSON/executor/CUE/source/observer; independently compare retained
verifier inventories using `--requisites --include-outputs`. Seed graph/provider
must exclude observer; do not claim Go-CUE-free seeds or minimal source attestation.

Run both full Rust suites/default all-target strict Clippy debug/release, focused
exporter four profiles, retained 76 graph/oracle +14 solver four profiles, pinned
offline Go tests, full preflight, all ten actual Linux checks and six packages
built AND rebuilt. Preserve six fresh artifact-free compilers, feature isolation,
deterministic generation/drift and independent source/value checks. Pins: CUE
6d609d768f1686f9a3a2a20197cacdbb70e5c79d (v0.16.1), Nixpkgs
ac588ef1c914865dad517b337ea0eb9694a4021c. Stay near 1500 changed lines;
x86_64-linux only validated. Self-hosting and performance remain unestablished.

## Completed evidence

Two focused contract reviews approved before GO. Immutable kernel
`ee7b2b107b6cae0c312c18de4c73ae998a865f55` and independent evidence
`d1ac8adde861b17864c777bdd495abb2f8e558f5` both descend directly from merged #196
(`0bfb257c`). Both read-only implementation cross-reviews approved; exact snapshots
were integrated and both worker workspaces retired. Only `eval.rs` and one new
integration test change outside docs; no old assertions, grammar or guards change.

Of eight new groups, five expose failures of the old JSON-only implementation;
resource/refinement/exclusion controls also remain. All 51 exporter groups pass
with the bound witness in four feature/profile configurations with strict
all-target Clippy. An independent 155 ordered-pair scratch matrix also matches
fresh pinned CUE, including empty, closed/open, nested, contradictory-tail and
scalar controls; none of those
observations seeds an expected answer or constructor. This is evidence for the
scoped fix, not proof of full CUE or future optional/default residual semantics.

Both full Rust suites/default all-target Clippy pass debug/release; all 76 retained
graph/oracle and 14 solver groups pass four profiles with strict scoped Clippy.
Pinned offline observer Go tests pass. Preflight passes 13/13 (seven inapplicable
skips); all ten actual Linux checks and six packages build AND rebuild. Six fresh
artifact-free compilers, feature isolation, generation/drift and source/value
checks remain. Whole-repository whitespace checks 1291 text files.

Native atom/family/policy artifact directories and all 249 full-bundle JSON files
compare byte-identical to #196. Four policy hashes, generated Rust and existing
independent expected answers remain unchanged. Exporter/atom/family/policy/full-
JSON constructors change to `9bqy69b5…`, `zhknd1wm…`, `9lnf96c6…`, `0yk9pvbk…`,
`hhjbx2vi…`; observer stays `303m6ck2…`. The exporter source has exactly eight
files at `/nix/store/25cmxma0ph9nnlna5ixhm3b3k595q7qz-source`; the six raw policy
files remain `/nix/store/06c9bq71vs4rldvajyh1wxis8aklcd08-source`.

Independent recursive queries match all three retained native graph inventories,
each 1407 derivations/3392 full build paths. Current forbidden JSON, executor,
pinned CUE/source and observer are absent. The independent seed comparison matches
1436/3466, excluding observer from graph/provider environment. No Go-CUE-free seed
or minimal source-attestation claim follows. Optional fields, full scanner export
and #175 remain held; the separate saved default prototype is unchanged.
