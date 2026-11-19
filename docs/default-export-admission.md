# Default exporter admission: distance to #175

Milestone [#212](https://github.com/kolohelios/cuestacean/issues/212).

## Current planning role: compatibility umbrella

After #221, the user explicitly approved treating #175 as a long-term
compatibility umbrella, NOT the critical delivery gate. Its broader original
profile, counterexamples and held prototype remain unresolved; none are silently
reclassified as satisfied. The failed eager pair representation is evidence, not
a mandated implementation architecture. The historical gate inventory below
still describes those semantic obligations, not a requirement to solve all of them
before every useful compiler-input milestone.

Choose bounded work from actual remaining Go-dependent inputs, with an exact
source/consumer contract, independent evidence and construction proof for each.
Keep incomplete preferences live, error/fallback discipline, current bounds,
retained assertions and protected validation. Smaller capabilities close their
own issues, not this umbrella. Scope/authority choices remain explicit.
The [#222 AST-policy data cut](native-ast-lowering-policy.md) follows that direction
without adding exporter semantics or claiming full native schema evaluation.

## Decision checkpoint, not semantic admission

This is the next decision after the [native coalescing audit](native-coalescing-audit.md).
The original [#175](https://github.com/kolohelios/cuestacean/issues/175) remains
OPEN and its saved prototype remains held. This document does not authorize
production imports, default selection, a new gate or a narrowed definition of done.
Two independent focused reviews must check the gap inventory and next selection
against the live exporter, original issue and actual raw AST fragment before the
decision is finalized. There is no semantic implementation GO in this slice.

**Assessment: not one integration/cleanup PR away.** We have bounded scalar
operational evidence, not an admitted exporter algorithm. There is no defensible
completion percentage or date while the first semantic admission gate is open.
The three gates below are acceptance gates, NOT a promise of three PRs.

## What is implemented, and what that does not establish

| Boundary | Current evidence | #175 acceptance still missing |
| --- | --- | --- |
| Mode transitions and lifetime | Native successful-batch primitive, flat/recursive exact-i64 IR constructors, original-mode history | A reviewed semantic admission for exporter contexts; matching pinned operational order is not confluence or readiness |
| Scalar coalescing | Actual native append transcript; explicit scalar projection matches bounded v4 controls | Justification for equality/residual deduplication under partial tasks and non-scalar constraints, not merely another trace field |
| Exporter source syntax | Current production exporter supports ordinary Or, lexical lets, booleans and prefix lists | #218 adds marks/Top only under the separately reviewed one-group tagged guard; general grouping/support admission is still missing |
| Struct/list residuals | Existing required-field, closure, typed-tail and container-bound behavior | Default support and preferred demand must preserve these constraints across contextual refinement |
| Public demand | Independent Default/Unify/export observations; conservative research suspension | Production choice among ordinary/preferred/incomplete/conflict without error-triggered fallback |
| Construction | Existing native producer/build isolation remains checked | Fresh production exporter admission and graph attestation after its implementation sources change |

The research constructors consume hand-built exact-i64 IR. They do not parse raw
exporter source, represent abstract `int`/top/string/struct/list domains, instantiate
productive schema templates, or implement preferred export demand. Their matching
corpora therefore cannot be counted as implemented exporter cases.

The saved `work/native-default-domains` prototype is based on #173 and has known
genuine contextual failures. Later boolean/prefix-list/container-readiness changes
are on main. Do not merge the saved branch wholesale or discard those later
invariants. Saved prototype tests passing is not a production admission signal.

## Concrete vertical target: the real AST instruction data

The original target locator was
`bootstrap/ast-lowering-fragments.cue:#AstLowering.cases.integer`. The SAME literal
alternative is now canonically owned by `bootstrap/ast-integer-policy.cue` at
ordinary data path `cases.integer`, with a required provider slot in the remaining
AST policy ([#220 source boundary](native-ast-integer-policy.md)). Relocation and
partial production use do not change original #175's matrix or definition of done:

```cue
*{op: "concat", parts: [
    {op: "text", value: "Value::from("},
    {op: "signed", field: "value"},
    {op: "text", value: "i64)"},
]} | {op: "text", value: _}
```

Acceptance requires native construction of that instruction-tree JSON by default,
and a separately authored refinement forcing the text alternative and supplying
its value. Refinement with `{op:"text"}` alone must remain Incomplete: `_` does
not supply a concrete value. The dotted path above is a documentation locator,
NOT a new native selector interface. Evidence needs explicit supported source
wrappers/root embeddings around the unchanged fragment, not a full policy schema. `op`, `parts`, `field` and `value` are ordinary data labels; there must
be no opcode/filename-specific evaluator rule. Instructions are data, not CUE
operations to execute. The raw default fragment must not be rewritten to erase
its nonpreferred alternative or seeded from expected JSON.

This target is NOT full #AstLowering/#LoweringPolicy construction, full scanner
schema admission, or six-seed closure. Passing it is necessary for #175, not
sufficient to close the issue.

## Three explicit remaining gates

### A. Broader semantic admission design — open, not a global delivery gate

Before production edits implementing that BROADER profile, agree a parser-to-demand
contract over the current exporter representation. It must state:

- Which residual value, preference/support, group and closure facts survive at
  EACH demanded data path, including nested fields and reused definitions/lets.
- How all ordinary contextual conjuncts constrain branch feasibility before
  preferred projection, without treating Incomplete/Unsupported/Limit as Conflict.
- Which equality/readiness certificate permits residual alternatives to coalesce;
  merging alternatives must preserve ordinary feasibility, preference/support
  behavior and demand classification under EVERY admitted subsequent refinement
  and demand. Equal JSON even with matching container bounds, equal i64, current
  mode or finalized-only status alone is not a general default-residual certificate. A sufficient native invariant need not imitate every
  internal Go task, but it must justify the admitted semantics independently.
- How production order-invariance is retained. Preserve genuine both-order
  counterexamples separately from pinned CUE order discrepancies. Do not copy
  oracle order dependence into production or silently delete discrepant cases.
- The exact admitted/excluded class, failure/suspension behavior, existing separate
  cumulative ledgers (4096 evaluation steps, 2048 materialized occurrences, 64 KiB
  copied value strings and 64 KiB encoded output), structural caps, and which
  executable rules remain native versus CUE-owned. Charge residual expansion,
  support/group metadata, equality and preferred/ordinary trials before covered
  work/copies; no branch/path/binding/projection ledger resets. A
  test-only Suspended result is not automatically an exporter Error/API decision.

A possible SMALLER separate capability is a mutually exclusive tagged-struct
choice with one preference group at a demanded path and no nested preference
dependency. The actual fragment motivates investigating this, not admitting it:
source-level group counts are insufficient through repeated references, closure,
container fields and other disjunction contexts. Any smaller production profile
needs its own reviewed contextual guard and explicit scope approval. It would not
settle the original #175 case matrix. Do not silently redefine #175 as that slice.

### Original obligations versus the candidate smaller profile

| Original #175 obligation (all remain required) | Missing admission/evidence | Smaller tagged candidate |
| --- | --- | --- |
| Grouping, mark placement, `_`, M0–M3/D0–D2/U0–U2 and nested symmetric failures | Contextual residual/support and demand contract; invalid syntax rules | Might exclude nested dependencies only with explicit scope approval; cannot satisfy the original nested matrix by exclusion |
| Scalar override, equal/multiple/abstract defaults and conflicting preferences | Preserve absent versus present-bottom and marked VALUE support; incomplete candidates remain live | Tag exclusivity alone supplies none of these rules |
| Struct/list/string defaults, tag refinement, prefix/tail and definition closure | Non-scalar feasibility and contextual residual equivalence | Still required for admitted containers, even if tags are disjoint |
| Let capture/reuse, productive templates, binding-cycle restriction | Per-instantiation preference/group facts and immutable refinements | One source-level mark does not establish a one-group runtime guard |
| Raw AST fragment: default, forced text with value, text without value | Native JSON/concrete and Incomplete cases, independent source wrappers/expectations | Necessary target, not full #175 or whole-policy closure |
| Budgets, errors, order, recovery and construction proof | All separate ledgers, preserved failure categories and Gate C | Still required; no weaker validation |
| Existing exclusions: imports, patterns, optional fields, general field references and full policy schema | Retain exclusions unless separately reviewed | No incidental admission; narrowing requires an explicit separate decision |

The admission review must return an explicit implementation GO or a concrete
counterexample/unresolved invariant. Gate A remains open on NO-GO. Another
observation endpoint or a larger matching scalar corpus does not exit this gate.

### B. Exporter implementation and independent functional evidence — not started

Implement the admitted contract on current main, preserving existing public errors
and guards. Do not route eager errors to a research graph or process/Go fallback.
Keep the held prototype only as evidence, not an implementation authority.

Required cases from the original #175 contract remain:

- Scalar override, multiple/equal defaults, abstract defaults, eliminated marked
  VALUE support versus conflicting preferences, nested M2/M3 and group boundaries.
- `_`, invalid standalone/misplaced marks and adjacent bottom-like spelling.
- Preferred structs/lists/strings, tag refinement, open-list residuals, definition
  closure, productive recursive templates, lexical capture/reuse and binding cycles.
- The actual raw AST integer fragment, default and forced-text paths, with complete
  independently handwritten JSON and fresh pinned CUE comparisons.
- Reordering/reuse/renaming, incomplete preferred candidates, shared-budget/copy
  exhaustion, and no partial CLI output on failure. Preserve every old assertion.

Do not use known CUE asymmetries as an excuse for genuine symmetric failures.
Do not claim that finite passing tests alone prove a general default calculus.

### C. Production integration and construction proof — not started

After A/B, complete two focused implementation cross-reviews and the full pinned
Rust/Go/Nix/feature/drift/source/value validation. Native exporter constructor
identities may legitimately change; expected artifacts are not construction inputs.
Re-attest actual allowlisted sources and recursive full build graphs, not runtime
references. Retain all twelve builder outputs, 249 bundle JSON files, generated
Rust and six fresh artifact-free compiler binaries unless a separately reviewed
semantic change explicitly requires an artifact difference. No raised guards or
Go-CUE-free/minimal-source seed claims. Protected exact-head CI/rebase auto-merge
and confirmed merge remain required.

## Definition of done and near-term direction

Close #175 only when its reviewed semantic profile is implemented in the actual
native exporter, the real fragment AND the original functional matrix are covered,
and Gate C succeeds. Research APIs, Go traces and scalar transcript agreement are
prerequisites/evidence, not substitutes for those conditions.

The historical Gate A design was reviewed in #214 and remains NO-GO for the
broader profile. It is no longer the default next delivery. Follow the user-approved
consumer-driven planning role above: select a real remaining Go-dependent compiler
input, specify the bounded admitted cut and validate it independently. Do not
schedule another trace/audit endpoint or promise #175 is almost done by default.
Any new broader semantic blocker belongs here or on #175 with a minimal witness
and named missing invariant; it need not block an unrelated supported source cut.

## Focused review outcome

Independent semantic and live-exporter reviews approved this docs-only checkpoint
against merged #210, the original issue, saved prototype and real raw fragment.
They resolved contextual equivalence versus JSON/bound equality, separate
cumulative ledgers, original-versus-smaller scope disposition, and the third
fragment path (text tag without a supplied value remains Incomplete).
Approval finalizes the distance-to-completion assessment and next design artifact;
it is explicitly NO semantic/producer implementation GO. Gate A and #175 stay open.
The subsequent [#214 concrete design](native-default-admission-design.md) records
proposed identities/residuals/feasibility and complete non-scalar witnesses; its
reviews explicitly return NO-GO with named missing selection and identity laws,
not an implementation approval.

The user subsequently approved pursuing a separately scoped [#216 tagged-struct
contract](tagged-default-admission.md), without narrowing #175. That future
implementation contract is reviewed for an explicitly finite one-group profile;
it does not satisfy original #175's broader Gate A or authorize general defaults.
The separate #218 implementation now covers the actual fragment's three native
outcomes under that exact guard. It does not satisfy the remaining original matrix
or grant general exporter implementation GO.
