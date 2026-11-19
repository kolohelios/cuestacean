# Separately scoped tagged-struct default admission

## Scope decision and review gate

[#216](https://github.com/kolohelios/cuestacean/issues/216) is explicitly separate
from original #175. The user approved pursuing this smaller admission contract,
not changing #175's definition of done or granting implementation GO. This slice
must obtain two independent focused reviews of the predicate, selection law,
feasibility proof and evidence obligations before its disposition is recorded.
No code changes are authorized by an unreviewed draft.

Target: construct the unchanged AST integer instruction alternative as DATA, plus
forced text/value and tag-only Incomplete paths. No label, opcode, filename or
expected JSON has privileged semantics. This is not full AST policy export,
general nested defaults, closure/template default support or a solution to
SEL-NESTED. Keep original #175 open, its saved prototype held, and all original
assertions/classifications intact.

## Exact proposed profile predicate

Apply this WHOLE-DOCUMENT predicate whenever marked syntax is present; check it
BEFORE branch evaluation, even for branches/contexts that will not be selected.
Preference-free documents retain their existing language, error behavior and
execution path. No broad default implementation or error-triggered fallback.

1. Exactly ONE occurrence of a flat TWO-alternative Or, with exactly ONE immediate
   marked alternative. No other Or anywhere in the marked document: no nested or
   unmarked disjunctions, multiple marked groups/marks, third alternatives or
   ambiguous grouping. Count actual tree occurrences, not just distinct arena IDs.
2. Root embeddings are conjunctions of ordinary struct constructor trees. The
   unique choice is a direct field value at a statically determined FIELD-ONLY
   path. No list ancestor, root-level scalar/default choice or alias can replicate
   the preference occurrence. Duplicate ordinary context fields mean conjunction,
   not overwrite; they do not erase the one choice occurrence.
3. BOTH alternatives are finite ordinary struct constructor trees with no
   duplicate labels within an alternative constructor. The marked alternative is
   fully concrete literal data: scalar literals, structs and FIXED lists only.
   The other alternative may additionally contain scalar types and Top leaves.
   Alternatives contain no And, Or, references, bindings or computed operations.
4. There exists a common direct field label whose alternatives have unequal
   direct scalar literal values. This generic disjointness witness may be ANY
   label/primitive literals; never hardcode `op`, `concat`, `text` or file paths.
   Those required literal constraints make the two whole alternatives disjoint.
5. All other context is finite conjunction-only constraints: scalar literals,
   string/int/bool types, Top, required ordinary structs and FIXED lists. Forbid
   inequality, open/prefix lists and ellipses in a marked document. No optional
   fields, operations or other already-excluded source forms.
6. NO definitions, let declarations, Ref/Binding/unresolved Name edges or productive
   recursion in a marked document, including unused declarations. This deliberately
   excludes definition closure and lexical reuse, rather than guessing identities.
   Preserve parser declaration metadata until this check; a source-level mark
   count after dropping aliases is not sufficient.
7. Top is an identity constraint within this admitted marked profile. Do not
   incidentally expand the ordinary preference-free language: its previous `_`
   rejection/error category remains unchanged: bare `_` is currently
   `Unsupported("value reference or type")`; an undefined `_Unknown` follows name
   resolution and is Unknown. Preserve binding/shadowing behavior too. Quoted strings/labels containing
   stars or underscores are ordinary data, not profile indicators.

Malformed mark placement, standalone stars and adjacent bottom-like `_|` spelling
remain syntax/profile failures, never parsed as an unrelated successful value.
Valid CUE outside this predicate is Unsupported, not a semantic Conflict. All
parse/range/name/resource errors remain errors even in nonselected alternatives.
Any required metadata must be derived from admitted syntax, not source-name tests.

## Representation and contextual selection

The parser/evaluator interface must explicitly represent Top and the two
ordered alternatives plus which is marked. Preserve binary syntax/node/depth
accounting; do not join parenthesized or referenced groups. No parser-only mark
may leak to ordinary materialization. Admission must reject the entire document
on predicate failure before any preferred/ordinary result can publish JSON.

At the unique choice path, collect ALL ordinary conjuncts from both root files,
merged duplicate fields and every enclosing structural conjunction before choice.
Keep both branch syntax trees intact. Do not materialize/deduplicate branches or
cache a chosen branch before contextual meet. The path is a field sequence, not
a selector/value reference. With refs/lets/lists-as-ancestors excluded, one source
occurrence has exactly one demanded logical path; shallow/full observations can
recompute pure feasibility without allocating persistent group/closure identities.

Let P be the marked branch, Q the other, and C the complete ordinary context at
that path. Compute BOTH `feasible(P & C)` and `feasible(Q & C)` under the SAME
remaining invocation evaluation ledger. Errors propagate directly. Selection:

| P feasible | Q feasible | Demand |
| --- | --- | --- |
| false | false | Conflict |
| true | either | Ordinary materialization of P & C |
| false | true | Ordinary materialization of Q & C |

Never fall through to Q because P's materialization is Incomplete, Unsupported,
Range, Unknown or Limit. P may gain an incomplete extra field through C, even
though its unrefined literal tree was concrete. That incompleteness stays live.
Feasibility is NOT implemented by catching export/materialization failure.

Shallow demand checks ordinary possibility (Conflict only when neither survives),
not JSON/default selection. Full demand selects by the table, then uses the
ordinary finite materializer. Every recomputation is charged; no branch/path or
shallow/full budget reset or selected-result cache. Other document fields still
must satisfy ordinary full export; a ready default cannot hide their failures.

## Feasibility algorithm and proof obligation

Proposed helper: `feasible(terms, depth) -> Result<bool, Error>`. No public error or
Unresolved state is needed IF the following finite-profile totality proof holds.

Flatten only And and discard Top identity at the current path. Classify every
remaining term as scalar, ordinary struct or fixed list; incompatible shapes
contradict. Scalar type constraints must agree, literals must agree and satisfy
those types; abstract scalar-only constraints are feasible but not concrete.
All-Top constraints are feasible, not an invented concrete value.

For structs, union required field labels and recursively check the conjunction
of every term at each label. There is no closure/optional-field/reference state
in this profile. For fixed lists, every supplied list length must agree and each
position's complete child conjunction must be feasible. Do not create arbitrary
extra list elements. No open/prefix tail case is admitted here.

Structural induction gives a completion exactly when every local compatible
shape/type/literal and required child has one: finite constructor trees, required
open struct fields and fixed list positions have no hidden cross-field choices or
cycles after selecting P or Q. This is a proof of EXISTENCE, not permission to
emit canonical dummy completions. Scalar types/Top remain Incomplete at output
unless actual data constrains them. Inspect every relevant sibling so an abstract
child cannot conceal another child's contradiction. A proved contradiction may
short-circuit because unsupported syntax has already failed whole-document
admission. Depth/work exhaustion remains Limit, not false feasibility.

Before context, this single marked group has ordinary domain P|Q and preferred
domain P (M1/D1). With default-free C, both ordinary P&C and preferred P&C use
exactly the same constraints (U1). Thus ordinary feasibility of P&C also proves
preferred feasibility; there is no independent live-but-conflicting preference
intersection to resolve. When P&C contradicts, every marked VALUE alternative is
eliminated and contextual elimination leaves Q&C as ordinary. This argument is
specific to the excluded-interaction profile, not a general M/D/U rewrite rule.

Selection follows M1/D1/U1 plus contextual elimination for ONE preference group:
if its only marked VALUE branch survives, it is preferred; otherwise the remaining
unmarked branch is ordinary. There is no second preference, nested retention/
suppression, or interacting default meet. Disjoint required literal tags and whole
branch retention prevent ambiguous overlap or tag/payload cross-products. No
residual-value merge, pointer/partial-node certificate or Go task emulation is used.
Commutativity comes from conjunction meet over complete field/position constraints,
not from copying a pinned evaluation order. Work/first diagnostics may vary with
order, but with sufficient budget values/classification must not.

## Explicit admitted two-file target and incomplete-preference control

`schema.cue` contains only `package fragment`. Default `program.cue` is:

```cue
package fragment
{integer: (*{op: "concat", parts: [
    {op: "text", value: "Value::from("},
    {op: "signed", field: "value"},
    {op: "text", value: "i64)"},
]} | {op: "text", value: _})} & {}
```

The alternative is the original instruction DATA; no full policy schema or
selector is included. Complete expected JSON is:

```json
{"integer":{"op":"concat","parts":[{"op":"text","value":"Value::from("},{"op":"signed","field":"value"},{"op":"text","value":"i64)"}]}}
```

Replace only the trailing `& {}` with `& {integer:{op:"text",value:"x"}}` for
complete `{"integer":{"op":"text","value":"x"}}`; replace it with
`& {integer:{op:"text"}}` for Incomplete/no JSON. Both files remain package-bearing,
with no definitions/lets/references or incidental extra choices.

A required counterfactual-fallback control, using the same package-only schema:

```cue
package fragment
{choice: (*{kind:"p"} | {kind:"q",n:1})} & {choice:{n:int}}
```

P&C is feasible but incomplete; Q&C would be concrete. Required result is
Incomplete, never Q. This shows why concrete P at admission does not permit
catching later materialization errors as branch elimination.

## Existing bounds, ownership and evidence required

Keep all source/token/syntax/name/width/depth guards, 4096 cumulative evaluation
steps, 2048 materialized occurrences, 64 KiB copied strings and 64 KiB encoded
output. Preserve separate ledgers. Charge profile node/metadata inspections,
context/field/position collection, feasibility visits and BOTH branches before
covered allocations/copies. Materialization/output uses the existing ledgers;
no feasibility work bypass, branch reset, stack override or raised guard.
Do not promise every syntactically admitted input avoids existing Limit errors.

Implementation ownership must be agreed AFTER reviews: parser/model/name
resolution versus profile/feasibility/evaluator, with coordinator independent tests,
raw-fragment evidence, Nix and integration. Use current main, not a wholesale merge
of the held prototype. Keep native construction free of exported JSON, generated
Rust, expected answers and Go/process/graph fallback. Native policy here is stated
openly; no CUE-hosting/self-hosting claim is introduced.

Required independent hand/native/pinned-CUE cases BEFORE production GO:

- The exact original integer alternative in supported package-bearing root
  wrappers: default concat JSON, forced text/x JSON, forced text-only Incomplete.
- Rename labels/tags/files and reverse root-file/conjunction/field order. Swap
  alternative order without moving which branch is marked. No privileged tags.
- Preferred branch survives; preferred eliminated by shallow/deep context; both
  eliminated; nonpreferred still incomplete; feasible preferred gains an abstract
  extra field while nonpreferred is concrete—Incomplete, never fallback.
- Nested struct and fixed-list conflicts, array length mismatch, bool/string/i64
  and Top identity. Correlated tag/payload branch results must not be fabricated.
- Every exclusion above, including unused/nested declarations, repeated aliases,
  multiple groups, extra Or in unrelated/nonselected data, open tails and list
  ancestry. These are #216 Unsupported cases, NOT changed #175 requirements.
- Admission-before-demand, input immutability, fresh observations, lowered-budget
  failure/recovery and unchanged old preference-free cases. No partial CLI JSON.

Original #175 default/nested/closure/reuse tests remain on their existing paths,
not rewritten into this profile. Passing this task cannot close #175 or admit
full AST policy/schema construction.

Implementation still requires explicit GO, immutable cross-reviews, both full Rust
suites/all-target strict Clippy debug/release, all retained research profiles,
pinned preflight, TEN actual checks/SEVEN packages build AND rebuild, fresh native
construction graphs and exact-head protected CI/rebase merge. Production source
identities may legitimately change; all twelve builder outputs, 249 JSON exports,
generated Rust and six fresh artifact-free compilers must remain independently
checked. No Go-free/minimal-source seed claim. Linux x86_64 only.

## Focused review disposition

Two independent semantic/feasibility reviews approved the future implementation
contract in immutable snapshot `a49b9e57065062ffac0a8929e97a876d66eb6163`, conditional
on the EXACT whole-document guard, complete contextual meet, both feasibility
checks and required evidence. Neither review grants implementation/ownership GO
or claims that the producer exists. Agree the concrete parser/evaluator interface
and file ownership, then obtain explicit implementation GO before edits.

Unlike original #175, this finite one-group class has a stated selection law and
structural feasibility proof. Its exclusions remove the unresolved nested/default
identity/closure interactions; they do NOT solve those interactions generally or
change any original #175 requirement. No existing assertion was reclassified.

Fresh pinned CUE CLI processes checked seven concrete two-file outcomes (including
the actual default/forced-text instruction data, deep/list conflict elimination,
generic bool tags, Top containers and deep field paths), two Incomplete cases and
two Conflict cases. Expectations were independently authored first; concrete
outputs also survived file-order reversal. Incomplete export failed without JSON
while eval succeeded; Conflict also failed eval. Processes cleared the environment,
used an unusable PATH and owned scratch HOME. These are oracle observations, NOT
passing native feature tests. A separate semantic review also reported 12 probes
and eight alternative/field/file-order variants; no aggregate coverage claim follows.

At the #217 documentation merge, production source/parser/evaluator behavior,
artifacts and construction identities were unchanged. Original #175 and its
saved prototype stay open/held. The next bounded task is implementation of THIS
reviewed profile, not broader default admission or another trace endpoint.

## #218 concrete interface and implementation GO

Two independent focused parser/evaluator interface reviews approved these
additions against merged #217 before implementation. The earlier semantic
predicate is unchanged. Coordinator implementation GO is limited to this exact
interface, predicate and ownership; it is NOT production acceptance or #175 GO.

```rust
// Crate-private additions; public export/error APIs stay unchanged.
Expr::Top;
Expr::Marked(Id); // counted syntax wrapper; never ordinary materialization
Document { nodes, definitions, roots, marked: bool, lets: usize }
```

`marked` is token-derived presence across both already-lexed source files, NOT an
admission certificate. Quoted/comment stars do not activate it. Without it, `_`
still takes the exact legacy Unsupported path. `lets` retains the parser binding
count, even after name resolution; never infer declaration absence from resolved
reachable IDs. Cycle traversal includes the Marked child. A mark parses one
operand above conjunction/disjunction precedence (minimum 3 versus 2/1), with its
own syntax node/height; parenthesized operands remain intact. A mark around And/Or,
nested marks, extra Or, or non-struct alternatives cannot pass admission.

The evaluator runs charged whole-document admission before demand, including all
arena nodes, root-tree occurrences, declaration metadata and nonselected context.
Only a private successfully admitted state enables tagged demand. At a field path,
And adds conjuncts without navigating or duplicating that path; list ancestry is
still excluded. Collect complete local terms before either trial. Deep structural
possibility is required for shallow requests in this profile: a shallow container
check alone cannot certify absence of deep contradictions. Compute both branch
feasibilities without boolean short circuit, sharing the same remaining ledger;
full demand selects, then ordinary materialization may still return Incomplete.
Top/Marked on the unadmitted ordinary path fail defensively, never reach an
unreachable materialization match or enable new preference-free language.

Agreed distinct ownership: parser owner changes only
`bootstrap/spec-export/src/{model,parser,bindings}.rs`; evaluator owner changes only
`bootstrap/spec-export/src/eval.rs`, including defensive Document initializers.
Coordinator owns independent tests, docs, Nix and integration. No new production
module or expansion of the eight-file source cut. Immutable stable-main handoffs,
read-only cross-reviews, all required evidence and final production acceptance
remain mandatory. Future implementation changes to the admitted class require
renewed focused review, not a permissive guard adjustment.

## Implemented capability and independent local evidence

#218 implements this exact profile on main's current exporter, not the held #175
prototype. Immutable parser handoff `2b3dad789334fdfdc3c12d1f823ae98aa2a46c07`
and evaluator handoff `09c6f694c1fd8bcd94dffae47f15d615d518cc81` were integrated
by owned path. Cross-review caught two P/Q struct inspections preceding their
charge; the coordinator added the charge before inspection. Duplicate target
answers/unit probes were removed from production source and retained as independent
integration evidence. The existing private defensive tests gain unadmitted
Top/Marked rejection; no old assertion or held counterexample changes.

Eight independent integration groups in
`bootstrap/spec-export/tests/tagged_defaults.rs` compare complete authored JSON
and exact native errors with fresh pinned CUE/native processes. The real raw
integer choice is read intact from `ast-lowering-fragments.cue`, including BOTH
alternatives; package-bearing wrappers do not introduce a selector or full policy
schema. The old exporter failed this default-path regression as Unsupported after
the oracle produced the independently expected JSON. New coverage includes the
three target outcomes, preferred-feasible Incomplete versus concrete Q, deep/list
elimination, ancestor/unrelated-sibling conflicts, duplicate conjuncts, all-Top
incompleteness, generic bool/string/exact-i64 tags, ordering/renaming, every profile
exclusion including unused/nonselected content, name/range errors, and exact
lowered-work failure/recovery. Comparing a pruned Q with a fully feasible Q checks
that a surviving P does not skip Q's charged feasibility work.

The `native-spec-closure` sandbox additionally runs the minimal PRODUCTION exporter
on raw source wrappers, before consulting expected answers or Go CUE. It checks
default/text complete JSON, repeated reversed inputs and tag-only Incomplete/no
JSON, under cleared environments, unusable PATH and owned HOME. It retains
`default.json`, `text.json` and `incomplete.err` alongside existing proof outputs.
These are verification outputs, never producer inputs. The raw full fragment
source and hand answers are test/verifier-only; production construction still has
exactly eight allowlisted files, at
`/nix/store/11ns7kl8kpb7inipfymx6fwig9mml5cd-source`. No target opcodes, source
filenames, expected JSON, process execution or provider environment are embedded
in the production implementation. The raw policy cut remains six files.

Local Linux x86_64 evidence: all 59 exporter tests in four profiles, both complete
Rust suites/default strict all-target Clippy debug/release, all retained 135
research groups/four profiles and strict focused Clippy pass. Pinned preflight
passes 13/13 (seven skips); TEN actual checks and SEVEN packages build AND rebuild,
including the new production-binary sandbox probes. Six fresh artifact-free
compilers, feature isolation, generation/drift and independent source/value checks
remain. Other architectures are omitted, not passed.

All twelve builder outputs, atom/policy artifacts and all 249 full-bundle JSON files
are byte-identical to #217; committed generated Rust and independent old answers
are unchanged. Exporter/atom/family/full-JSON/policy constructors legitimately
change to `5ip9r0q3…`, `ls22c5vl…`, `2klnfln2…`, `331kc4ih…`, `w0jszfsi…`.
Both oracle constructors remain unchanged. Fresh independent recursive derivation
and full-build queries match all three native verifier inventories at 1407/3392
each; the seed inventory matches 1436/3466. Both current providers' outputs and
derivations, current downstream executor and provider environments are absent;
native graphs also exclude pinned Go CUE/source and the current JSON bundle.
No Go-free compiler-seed/minimal-source, full-policy, self-hosting or performance
claim follows. Final immutable review and protected exact-head CI/confirmed merge
remain required gates. Original #175 stays open/held and unchanged in scope.

The subsequent [#220 source/consumer integration](native-ast-integer-policy.md)
moves that unchanged BOTH-alternative expression into its own canonical ordinary
raw owner and independently native production artifact. The exporter and this
admission predicate are unchanged. Canonical AST default-policy construction uses
the materialized artifact; override fixtures explicitly use the unresolved raw
owner before context. This does not admit full AST policy/schema construction or
turn selected JSON into a residual preference/refinement interface.
