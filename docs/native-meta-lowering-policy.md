# Native meta-lowering policy data and flat scalar preferences

## Consumer-driven selection and gates

Milestone [#224](https://github.com/kolohelios/cuestacean/issues/224).
After #222/#223's native AST policy DATA, construct canonical `meta-lowering.json`
without Go CUE, as a second complete compiler-policy input. #175 remains the
user-approved OPEN compatibility umbrella; nested/default reuse/closure laws are
not a delivery prerequisite when this exact source cut does not rely on them.

Assessment inspected all remaining policies. The ordinary scanner body exceeds
4096 native evaluation steps; do NOT raise that guard. The ordinary meta body
already exports within existing bounds. Its two alias defaults need one precise
new admission class; they do NOT justify general default support. Obtain two
focused semantic/source and independence/evidence reviews plus explicit GO before
code. Stop on an unresolved law, hidden budget increase or broader class.

## Exact additional scalar admission class

Keep the current tagged-struct class unchanged, including its disjoint literal
witness. Add a separate flat literal-preference class at a field-only path:

- Whole marked document still has exactly ONE binary Or and ONE immediate mark,
  no other Or/marks, no list ancestry, root choice, definitions, lets, references,
  aliases, recursion, inequality or open lists anywhere, including unused data.
- P is one concrete scalar literal: string, checked i64 or bool. Q is one scalar
  literal, scalar Type(string/int/bool), or Top. Structured branch syntax, And,
  computed values and incomplete preferred leaves remain excluded in this class.
  Q may overlap P, even equal it; do not coalesce branch syntax or infer readiness
  from equal output. A Top Q can be refined by ordinary context to a container.
- Other context remains the existing finite conjunction-only literal/type/Top/
  required-open-struct/fixed-list language. Complete contextual meet precedes BOTH
  branch feasibility trials, under the same remaining ledger. No cached choice,
  default-specific field name/opcode or filename. Model/public APIs and valid-input
  parser syntax stay unchanged; the invalid-token correction below is explicit.
  Preference-free `_` behavior and all limits remain unchanged.

Proof: M1/D1 gives ordinary P|Q and preferred P; default-free C constrains ordinary
and preferred P identically (U1). A feasible P&C therefore has a feasible preferred
P&C; contradiction eliminates the only marked VALUE branch and leaves Q&C ordinary.
The same structural existence proof applies to finite C. Neither disjoint tags nor
branch inequality is needed for THIS one flat scalar group. No second preference,
nested support or alias expansion enters. This is not a general pair-normalization
rule. Unsupported/Unknown/Range/Limit errors never mean false feasibility.

Full demand selects P whenever feasible, otherwise Q if feasible, else Conflict.
Incomplete remains Incomplete, never a fallback or invented completion. Shallow
checks deep ordinary possibility, not preference selection. Keep existing charged
admission/context/clone/field/position/feasibility/materialization/output ledgers.
`eval.rs` admission classification is extended; reuse unchanged selection and
possibility, without duplicating scheduler/mode logic or error catching. The precise
invalid-token lexical correction approved below is the sole parser exception.

One old #218 profile exclusion, `{v:(*1|2)}`, is now intentionally admitted. Move it
from the exclusion table into strict independently expected native/CUE positive
coverage. Do not weaken it into an either-result assertion. Root `*1|int`, nested
Or, multiple groups, abstract P and mixed/structured alternatives remain excluded.
Original #175 research assertions/counterexamples and held prototype stay intact.

## Canonical source modules and validation bridge

Move unchanged ordinary instructions/aliases from `meta-lowering-fragments.cue`
to `bootstrap/meta-lowering-body.cue`, ordinary root DATA with explicit version1.
Remove ONLY the two defaulted alias entries from that body; keep every other
represented instruction/alias unchanged. No native source rewriting or selectors.

Give the two unchanged expressions separate canonical field-only raw owners:

```cue
package bootstrap
{aliases: {text: *"emitValue" | _}}
```

```cue
package bootstrap
{aliases: {nextRule: *"emitModuleAction" | _}}
```

Use `meta-text-alias.cue` and `meta-next-rule-alias.cue`, each exported DIRECTLY
with unchanged package-only `native-policy-package.cue`. Each marked invocation
contains one group and no declarations; the ordinary body is a different supported
invocation. These are three independently materialized modules, not admission of
one combined mixed-let/default or multiple-group native document.

The Go-only #MetaLowering bridge retains #LoweringPolicy plus required slots for
ALL original cases and both alias providers. Raw default/override/schema oracle
processes load body, BOTH raw alias owners, bridge and selector/context before
selection. Existing forced text/previous-rule fixtures keep their exact outcomes;
never apply their context to a materialized default artifact. Missing body or
either alias must fail Incomplete/no JSON, not silently omit it.

Aliases here contain represented DSL target-name STRINGS, not CUE references across
modules. Typed Policy parsing still validates target existence, collisions and
chains. No schema-language or arbitrary-policy validation claim is added.

## Native producer and production consumer

Add `native-meta-lowering-policy`: unchanged native CLI API exports body and each
alias module twice with fresh cleared environment, unusable PATH and owned HOME;
all providers absolute/readable/executable. Compose through the UNCHANGED generic
`compose-policy.jq` in two binary stages, each exactly two bounded materialized
object inputs. Shared root aliases is a namespace; their leaves/cases/version are
disjoint. Compare repeated and reversed/grouped assembly bytes; enforce64KiB on
every intermediate/final output before publication. Do not reset any invocation
ledger or claim one shared4096 allowance across independent component calls.

Production `jsonExports` copies canonical `meta-lowering.json` from this artifact
and SKIPS Go for exact membership. Overrides explicitly choose raw sources upfront,
not fallback. No exported/captured expected answer, generated Rust, Go/schema,
full bundle, executor/compiler, oracle or verifier feeds native construction.
Source-derived native JSON intermediates are legitimate build-only data.

New package/check `native-meta-lowering-policy`: retain TWELVE old checks/NINE old
packages, now THIRTEEN/TEN. Its native derivation roots include exporter+artifact;
inspect recursive full graphs/registration, not runtime references. Exclude current
Go CUE output/drv/source, bundle/executor outputs/drvs, both oracle outputs/drvs and
provider environments. Separate verifier may depend on Go/bundle/compiler, never
feed construction. Exporter source remains eight files; meta raw union is FOUR CUE
files (package, body, two aliases), with two-file per-invocation cuts; structural
composition script remains unchanged. Existing artifact source cuts stay intact.
Exporter-dependent identities legitimately change with eval.rs; old artifact bytes,
all249 JSON files/all12 builders/generated Rust must not. Oracle identities stay.

Second native policy DATA input does not mean native full #LoweringPolicy schema
semantics, all-six closure, Go-free/minimal-source seeds, self-hosting or performance.
Existing typed/compiler/VM guards, including meta472/512-value pressure, stay fixed.

## Evidence, ownership and implementation gate

Kernel/build owner ONLY `bootstrap/spec-export/src/{eval,parser}.rs` (parser limited
to the separately approved lexical correction below),
`infra/tooling/flake.nix`, `bootstrap/meta-lowering-fragments.cue`, and new
`bootstrap/{meta-lowering-body,meta-text-alias,meta-next-rule-alias}.cue`.
Evidence owner ONLY new `bootstrap/spec-export/tests/scalar_defaults.rs`, new
`bootstrap/spec-export/tests/meta_policy.rs`, and the single fixture reclassification
in `tests/tagged_defaults.rs`. Coordinator owns docs, integration, source/artifact/
graph proofs and final CI. Stable merged #223 workspaces; immutable saved handoffs,
no saved amendment/rebase onto mutable coordinator; at most two workers. Format
owned files only. Apart from the exact lexical correction below, parser/model/
other runtime code and compose-policy.jq remain unchanged.

Before production acceptance require:

- Independently authored scalar/CUE expected values/errors: literals and types,
  equal/overlapping Q, Top, forced scalar and container completion through Q Top,
  both false, incomplete Q, whole-root unrelated/ancestor conflict, arbitrary labels,
  field/file/alternative/conjunction orders, no partial CLI JSON, limits/recovery.
- Every whole-document exclusion remains effective in unused/nonselected context;
  range/name/setup errors remain errors, no conditional fallback. Both trials share
  work; exact lowered-budget boundary/recovery and no preference-free expansion.
- Complete hand-authored alias module expectations and independent whole-policy
  raw Go/canonical production/baseline comparison. Body's represented data and both
  literal default expressions relocate unchanged; metadata version1 is explicit.
  Missing providers and late-default overrides have distinct expected failures.
- Native complete meta policy drives actual `bootstrap-lower-codegen` over ALL six
  canonical compiler policy inputs, matching all committed driver Rust, including
  byte-exact meta self-regeneration. Existing compiled forced/previous controls,
  alias validation, full-driver work and output bounds retain original assertions.
- Independent source/effect controls for body and aliases yield predicted emitted
  Rust; malformed source data may materialize but schema and typed compiler reject
  it without Rust. Do not claim generated-program execution unless actually run.
- Two final immutable implementation/evidence reviews; both full Rust suites/default
  strict all-target Clippy debug/release, exporter/four profiles and retained135
  research/four profiles, full preflight, THIRTEEN actual checks/TEN packages build
  AND rebuild, six fresh artifact-free compilers/profiles/drift/source/value checks,
  full graph/source proof and protected exact-head rebase CI through MERGED. Retire
  workers/workspaces before PR. Coherent slice target approximately<=1500 lines.

Two independent focused reviews approved this exact scalar selection proof,
whole-document/source/consumer boundary and ownership before implementation.
Read-only native probes show the ordinary meta body fits existing bounds (scanner
body does not). Explicit coordinator implementation GO is now recorded ONLY for
this contract, including strict positive reclassification of the one newly admitted
scalar-field fixture. No broader default/schema/native-seed claim is authorized.
Final immutable cross-reviews and all validation/CI gates remain required.

### Focused lexical correction discovered by independent comparison

The reversed witness `_|*1` is NOT legal CUE; `_ | *1` is. Enabling scalar Top
exposed a false native acceptance of `{v:(_|*1)}`, previously rejected by the
struct-only guard. Both reviewers independently confirmed native success versus
pinned CUE rejection. A witness-only whitespace repair would leave that bug.
Two focused correction reviews approved this precise ownership/interface exception
before explicit coordinator GO: in parser Word lexing, reject exact `_` immediately
followed by source byte `|`, with the inherited
`Unsupported("value reference or type")` diagnostic. Other words, strings,
comments and whitespace-separated syntax are unaffected. No Bottom syntax,
model/API, guard or valid-language expansion is added. Strict API/CLI no-JSON and
oracle rejection, legal reversed positive, standalone `_` and quoted `"_|"`
controls are required. Kernel owner may now implement only that correction in
`parser.rs`; final immutable cross-review/full validation remain required.

## Implemented evidence and local validation

Build `5c1f0e3467fc7e4bcaff302869eb17e1c0f27969` and hand-authored expected-Rust
layout child `9b93439f076617fd73638dc250453a59d2b57970` were integrated by owned
path. Independent tests `3fad8dddfc6ad1cd87c63a3877d5b2ccd6e4beb9` gained legal
reversed syntax in `aa9042f29e36635fbe8512d7734d32b2ac1ad378`, then strict malformed
adjacency regression `1fb796960cf238da8bd0fd7beb69d8b4eefb82ab`. After BOTH correction
reviews, parser-only child `c8ab666d58506c41db3af40e7e9d457ccdbc04c5` closed the real
false acceptance. No weakened expectation, Bottom support or broader parser claim.

Six scalar and four Meta groups compare complete independent values/aliases,
raw Go/schema/default production policies, both-trial same-node budget thresholds,
permutations, errors and exclusions. All80 exporter groups pass four profiles;
both full Rust suites/default strict all-target Clippy debug/release and retained
135 research groups/four profiles with focused strict Clippy pass. Preflight
passes13/13 (seven skips); THIRTEEN actual checks/TEN packages build AND rebuild,
retaining fresh six compilers/isolation/drift/source/value evidence. Only x86_64-linux.

All six lowering-driver goldens, including byte-exact meta self-regeneration,
compiled forced/previous controls, source effects and typed rejection checks pass.
ALL249 JSON outputs/all12 builders/old native artifacts/generated Rust remain
byte-identical to #223. Exporter-dependent constructors legitimately change;
oracle constructors do not. Exporter source still has eight files at
`/nix/store/4043zzwvjqgmwl86jpgz891mi56rv8nx-source`; the Meta raw cut has four at
`/nix/store/1vgal3bgx9zzmq08brswckkl44wrg0zw-source`; composer/AST raw owners and
six experimental-policy files are unchanged. Ordinary body projection and both
original default expressions compare unchanged at the documented source boundary.

Fresh recursive queries match retained inventories: four original native cuts
1407 derivations/3392 full paths each, AST policy1412/3410, Meta policy1411/3407,
shared seed1439/3476. Both current oracle outputs/drvs, current executor/provider
environments are absent; native graphs also exclude pinned Go CUE/source/current
full JSON. Two policy DATA inputs are now independently native, four remain Go-built.
No native full-schema, Go-free/minimal-source seed, self-hosting or performance
claim follows. #175 stays an open compatibility umbrella. Final immutable review
and exact-head protected CI/confirmed merge remain release gates.
