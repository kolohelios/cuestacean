# Proposed contextual default residuals and demand

Milestone [#214](https://github.com/kolohelios/cuestacean/issues/214).

## Status and scope

Gate A proposal following [the admission checkpoint](default-export-admission.md).
**Not an implementation contract yet; no producer/semantic GO.** This slice must
receive two independent focused reviews. A missing selection law is a blocker,
not permission to restore the held eager-pair prototype or narrow #175 silently.
No Rust/CUE/Nix/oracle implementation changes are authorized here.

Target the original #175 profile on current exporter main: literals/types, `_`,
required structs, fixed/typed-prefix lists, conjunction, syntactically grouped
marked/unmarked disjunctions, definition closure and lexical lets. Preserve the
original matrix, actual AST fragment, guards and current bool/prefix/container
readiness behavior. Imports/patterns/optional fields/general field references and
full policy schema stay excluded; binding cycles remain Unsupported. This is a
bounded native exporter proposal, not a general CUE solver or self-hosting claim.

## 1. Parser and instance identity proposal

Preserve the arena/public export API. Proposed syntax additions are Top and
annotated binary Or(left,right,group,left_marked,right_marked), with group a syntax
ID, not a reference edge. Same consecutive chain shares a group; parentheses and
reference/binding expansion never join syntactic groups. Preserve original binary
syntax height/node accounting. Mark flags refer only to immediate alternatives.
A standalone/misplaced star and adjacent bottom-like `_|` spelling remain invalid.

Distinguish syntax identity, lexical capture, definition closure and observed use:

```text
Term = (resolvedBodyId, closureIds, useId, dataPath)
GroupOccurrence = (useId, dataPath, syntaxGroupId)
PathComponent = Field(label) | Index(integer)
```

Current lexical capture is already resolved to immutable Binding body IDs; this
proposal does NOT introduce caller-time environment lookup. A new logical Ref/
Binding ENTRY is distinct from repeated shallow/full/support observations of that
same immutable use. Such reobservations must retain BOTH use and closure identities,
not allocate a new logical group/closure on every query. Independent entries remain
isolated. Binding use preserves declaration-site capture and introduces no
definition closedness. Definition instantiation gets a fresh closure identity;
that identity is not a preference-group ID. Shared template bodies stay immutable.
Repeated values/spellings do not identify uses or transfer support between paths.
**Named blocker IDENTITY-REUSE:** allocation/reuse through same-path aliases,
nested expansions and productive recursive demands is not defined by this tuple.
R-INSTANCE checks independent instances, not that reuse law. Lazy logical entry
keys must be specified before implementation; eagerly expanding every recursive
template or allocating fresh identities on every observation is not authorized.

## 2. Residual representation proposal: retain constraints, not projected JSON

Retain an immutable residual graph at every demanded path:

```text
Residual = {
  ordinary: conjunction of original Term constraints,
  choices: grouped alternatives with body/closure/use references,
  preferencePlan: unevaluated Group/Meet structure,
  support: per-group contextual feasibility facts,
}
```

Each choice retains correlation across the WHOLE alternative: never independently
union struct fields and accidentally manufacture a tag from one branch and payload
from another. Keep list fixed length/minimum, every prefix/tail constraint, inherited
closure admissions, literal/type exclusions and unevaluated child constraints.
Preference plans retain marks, nested groups and explicit absence versus a present
conflicting preference. Do not normalize them eagerly to value/default sets.

Residuals contain arena/use references and bounded metadata, not materialized JSON.
Contextual meet appends constraints to the same immutable observation; refinement
of one use does not mutate another or the template. A later broader observation
must recover the original residual, not a previously selected branch.

**Conservative equality rule for the proposal:** no nontrivial residual coalescing
by value. At most reuse an exactly identical immutable residual handle with the
SAME use/path/closure/preference-plan context. Do not merge merely equal JSON,
container bounds, scalar values or current modes. Projected JSON deduplication is
a final output operation only after preference resolution and residual readiness
are independently established. Retaining alternatives avoids one information-loss
step; it does NOT by itself settle default-selection semantics or prove bounds.
The scalar batch/coalescing model cannot be reused unchanged as a proof of this
no-residual-merge proposal: its operational state and merging assumptions differ.

## 3. Ordinary contextual feasibility, separate from export

Proposed operation `meet(residual, context, ledgers)` retains all ordinary conjuncts
before `observe(residual, demand, ledgers)` can consider preference. Feasibility
needs a constraint-preserving result, not JSON or an export error disguised as Bool:

- Conflict: establish contradiction in the admitted ordinary constraints.
- Possible: admitted ordinary constraints have a justified surviving completion;
  this does NOT imply export is concrete or identify a default.
- Unresolved: feasibility proof not established. Never reinterpret as Conflict.
- Unsupported/Range/Unknown/Limit: propagate the existing distinct failure, not
  alternative elimination, lost support or a second-engine retry.

Only Conflict may eliminate an ordinary alternative. If any potentially relevant
branch is Unresolved, preferred projection is not authorized. A public disposition
for this new internal state must be settled before implementation; do not silently
map it to Incomplete or add a new Error variant here.

Required meet obligations: Top is identity; compatible scalar types/literals meet;
struct fields and closure admission are inspected without losing branch correlation;
fixed lists constrain length, prefixes impose minima, and applicable tails constrain
every demanded position. An undemanded tail conflict does not close an open list.
Missing abstract data is not contradiction. Inspect all relevant shallow/deep
constraints so one incomplete child cannot conceal a conflicting sibling.
Productive schema recursion remains tied to bounded demanded paths; no infinite
normalization of templates. Unproductive recursion/binding cycles retain existing
Unsupported behavior. The old JSON materializer is not a feasibility oracle.

## 4. Preferred demand: laws required, one law still missing

The following are requirements on any candidate `select(residual, context)`, not
a complete executable definition of that operation:

1. Meet ALL ordinary context at the path before selecting preferences. Account for
   each group's marked VALUE support separately from preferred intersections.
2. With no relevant preference plan, use existing ordinary demand semantics. A
   complete ordinary result may export; Incomplete remains Incomplete.
3. A marked branch whose ordinary value support is contradicted can lose preference
   eligibility. Conflicting preferred candidates with live marked value support
   are not the same thing as no preference. Preserve absent/present-bottom/live.
4. A preferred branch that is feasible but incomplete must not disappear in favor
   of a ready nonpreferred result. Unsupported/Limit never justify such fallback.
5. Any ordinary-forced shortcut needs proof of exact residual forcing. The research
   rule for a sole exact i64 does not generalize to minimum JSON for an open struct
   or list: later refinements and defaults may still affect those containers.
6. Nested M2 retention/M3 suppression must be decided in the enclosing ordinary
   context. Do not blanket-reactivate suppressed recipes, condition only on a flat
   mark set, or follow operand encounter order as a production semantic rule.

**Named blocker SEL-NESTED:** a compositional, context-sensitive definition of
nested eligibility/suppression/reactivation and preferred-domain meet/union has
NOT been supplied by this representation. Retaining source structure/support
makes the question expressible, not answered. Eager pairs, static provenance and
context partitions have preserved genuine falsifications; matching the ordered
scalar operational model does not provide the missing order-invariant law.
Until SEL-NESTED is defined and checked, Gate A remains NO-GO for original #175.

## 5. Minimal non-scalar witness obligations

These are independently authored source/expected-outcome requirements for the
proposal, not implemented native capabilities or captured-output goldens. Pinned
CUE CLI probes may validate them; native defaults remain unsupported today.
Use explicit supported root/field wrappers, never infer a new selector interface.

| ID | Constraint expression / context | Required observation / invariant |
| --- | --- | --- |
| R-TAIL | `(*[1,...int] | [1,...string]) & [1,"x"]` | `[1,"x"]`: both initial minimal JSON arrays/bounds agree, but retained tails distinguish surviving ordinary branches |
| R-LENGTH | `(*[1,...int] | [1]) & [1,2]` | `[1,2]`: initial JSON `[1]` cannot freeze the open alternative to fixed length |
| R-INCOMPLETE | `*{n:int} | {n:1}` | Incomplete, not `{n:1}`; adding `{n:2}` yields `{n:2}` |
| R-SIBLING | `(*{a:int,b:1} | {a:2,b:2}) & {b:2}` | `{a:2,b:2}`: incomplete a must not conceal preferred branch's b contradiction |
| R-CLOSURE | `#Closed:{a:1}` then `(*#Closed | {a:1}) & {b:2}` | `{a:1,b:2}` via open nonpreferred alternative; closure identity cannot disappear with JSON dedup |
| R-INSTANCE | Two uses of `#Choice:{let C=(*{kind:"a",n:1}|{kind:"b",n:int}), value:C}`; refine only left.value to `{kind:"b",n:2}` | Left becomes b/2 while right stays default a/1; capture, closure and eligibility cannot be globally specialized |
| R-NESTED | Put preserved `((*3|1)|3)&((*1|2)|(*3|2))` in a required struct field, and reverse operands | Both Incomplete; wrapping a genuine scalar falsification in a container does not solve SEL-NESTED |
| R-AST | Unchanged actual AST integer alternative | Default concat data; `{op:"text",value:"x"}` gives text/x; `{op:"text"}` alone is Incomplete |

Keep the full original scalar/nested matrix and invalid-mark cases in addition to
these witnesses. Preserve pinned order discrepancies separately; they do not
explain symmetric failures. Renaming labels, ordering fields/files/declarations,
repeated uses and broader-after-narrower observations must preserve values and
classifications within the eventually admitted profile.

### Explicit two-file source wrappers and complete expectations

Each pair has these package-bearing `schema.cue` contents (definitions-only,
therefore a struct root—not a top expression):

```cue
package admission
#Closed: {a: 1}
#Choice: {
    let C = (*{kind: "a", n: 1} | {kind: "b", n: int})
    value: C
}
```

R-CLOSURE `program.cue` is:

```cue
package admission
{result: ((*#Closed | {a:1}) & {b:2})}
```

Complete expected export: `{"result":{"a":1,"b":2}}`.

R-INSTANCE `program.cue` is:

```cue
package admission
{left: #Choice, right: #Choice} & {left: {value: {kind:"b", n:2}}}
```

Complete expected export:
`{"left":{"value":{"kind":"b","n":2}},"right":{"value":{"kind":"a","n":1}}}`.
The refinement is nested structural conjunction, NOT a field-value reference.

R-AST default `program.cue` is:

```cue
package admission
{integer: (*{op: "concat", parts: [
    {op: "text", value: "Value::from("},
    {op: "signed", field: "value"},
    {op: "text", value: "i64)"},
]} | {op: "text", value: _})} & {}
```

The complete independently authored expectation is:

```json
{"integer":{"op":"concat","parts":[{"op":"text","value":"Value::from("},{"op":"signed","field":"value"},{"op":"text","value":"i64)"}]}}
```

The other two complete programs use the IDENTICAL preceding source, replacing
only the trailing `& {}` with the following independently authored conjunct:

| Conjunct replacing `& {}` | Complete expected export |
| --- | --- |
| `& {integer: {op:"text", value:"x"}}` | `{"integer":{"op":"text","value":"x"}}` |
| `& {integer: {op:"text"}}` | Incomplete; nonzero export, stderr, no JSON; ordinary eval succeeds |

These are evidence wrappers for the original raw alternative, not a native
selector, full policy schema, default-removal rewrite or instruction execution.
Their outcomes must be checked with fresh pinned CLI processes and owned HOME;
they are still NOT passing native default-support tests.

## 6. Budgets, executable ownership and implementation disposition

**Named blockers FEASIBILITY-TOTALITY / UNRESOLVED:** completion/contradiction
rules, bounded productive demands, public failure disposition and demonstrated
ledger fit remain unspecified. R-SIBLING checks immediate sibling contradiction,
not deep feasibility completeness. An arbitrary extra list element must not be
invented to disprove a valid finite prefix.

No changes to 64 KiB combined source, 4096 tokens, 2048 syntax nodes, 64 definitions/
bindings/container items, 32 syntax/output levels, 4096 evaluation steps, 2048
materialized occurrences, 64 KiB copied strings and 64 KiB encoded output.
Source/parser and evaluation/materialization/copy/output ledgers remain distinct;
all relevant branch/path/use/preferred/ordinary work shares the corresponding
invocation ledger with no resets. Charge residual visits, expansion, metadata,
context meet, support checks and final projection before covered copies/enqueues.
No coalescing optimization or higher guard may rescue an over-budget design.

Representation, lexical resolution, closure/storage, checked meet/feasibility and
budget/encoding primitives are proposed native responsibilities. Preference
selection/demand is not CUE-hosted by this proposal; its unresolved native policy
must be named rather than hidden behind configuration. Later CUE ownership needs
a checked interface, not a claim that this document implements it.

Before GO, resolve SEL-NESTED, instance identity and feasibility completeness on
productive admitted demands, define public Unresolved disposition, and demonstrate
that the residual strategy can satisfy original examples within existing ledgers.
This is not authorization to implement a smaller tagged profile: that still needs
explicit separate scope approval and cannot close #175.

If reviews cannot resolve these points, retain this NO-GO with precise witnesses
and the named missing laws. No new trace endpoint, native algorithm, parser marks
or producer integration should be substituted for the missing admission decision.
Future implementation still needs independent expectations, two focused contract
reviews/explicit GO, immutable cross-reviews and full pinned construction/CI checks.

## Reviewed disposition and witness checks

Both independent semantic/domain reviews approved publication of immutable design
`11a67c030c259cecd49bff80a7ce6e5e85a57299` as a precise NO-GO. Neither approved
semantic implementation. SEL-NESTED, IDENTITY-REUSE, FEASIBILITY-TOTALITY, public
UNRESOLVED disposition and ledger fit remain unresolved. The representation is a
proposal for retaining information, not a solution to those laws.

Fresh pinned CUE CLI processes confirmed ten concrete and four incomplete one-file
obligations, plus all five two-file wrapper outcomes above. Concrete expectations
were independently authored before invocation and checked again in fresh processes;
two-file concrete outputs also survived file-order reversal. Incomplete cases
failed export with no JSON while ordinary eval succeeded. Processes cleared the
environment, used an unusable PATH and owned scratch HOME. A review corrected the
R-INSTANCE separator from a rejected semicolon to a comma/newline; this was witness
syntax repair, not a semantic algorithm fix. No observed output seeds native code.

R-TAIL shows why minimal JSON plus bound flags cannot certify a retained DEFAULT
residual under future tail refinement. It is not a claim of a bug in the current
ordinary exporter, whose final projection happens after its contextual meet.
These checks establish expected oracle outcomes only—not native default support,
feasibility completeness, bounded implementation or Gate A admission.

Next decision: supply the missing compositional contextual selection law and its
instance/feasibility dependencies, or preserve this NO-GO. Do not start producer
edits or another trace endpoint to stand in for that decision; #175 remains held.
