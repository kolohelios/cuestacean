# Native unification-lowering policy DATA (#232 proposal)

## Selected dependency and claim

After #230, AST/meta/scanner policy DATA and three compiler construction paths are
native. Construct canonical unify-lowering.json as the FOURTH complete native DATA
input. Two other policy constructors/joined six-seed path still need Go CUE; existing
THREE isolated compiler constructors remain unchanged. This is NOT native control/
#LoweringPolicy schema evaluation or a fourth isolated compiler. #175 stays OPEN.
No exporter/executor/core/scanner Rust, Cargo, semantics/API/guards, composer, schema,
CLI or runtime provider ABI changes. No new default class, fallback or self-hosting.

Read-only source partition exports five supported programs and assembles14473 bytes
matching baseline. Recombining ordinary body yields existing Limit(evaluation steps).
That is feasibility only, never captured expected data or construction input. Do not
raise limits/reduce charges or claim one4096 ledger across independent programs.

## Raw ownership and finite helper refactoring

Under bootstrap/, relocate original instructions with ONLY these changes: six local
let declarations inside closed #UnifyLowering become six concrete package definitions
in unify-lowering-helpers.cue (prefix #UnifyLowering, suffixed
Side/Endpoint/Conditional/Relation/L/R); rename
references OUTSIDE quoted strings ONLY. Rust paths such as ::Endpoint/::Relation
are literal DATA and MUST NOT be renamed. Original helpers inherited definition
closure; preserve this finite canonical/override boundary, not general contextual
let/definition equivalence. No source extraction/scraping in production construction.

- unify-lowering-values.cue: ordinary brace-wrapped root, explicit formatVersion1,
  aliases.choose="if",21 root cases boolean/empty/inputEndpoint/inputValue/
  integerEndpoint/integerPrimitive/integerSum/integerValue/listCount/
  listCountEndpoint/listElement/listSlice/local/offset/program/recordCount/recordGet/
  recordHas/textConcat/textLength/unaryFail. integerPrimitive retains select/op field
  and five concrete branches negate/product/modulo/quotient/remainder, NOT defaults.
- unify-lowering-control.cue: ordinary root11 cases and/booleanIs/bounds/compare/fail/
  hasEndpoint/if/kindIs/letBounds/or/textCompare. No version/alias/shared leaf.
- unify-difference-policy.cue: original BOTH-alternative scalar string preference at
  cases.integerPrimitive.cases.difference.value, plus that node's op:text.
- unify-divide-policy.cue: same at cases.integerPrimitive.cases.divide.value.
- unify-same-policy.cue: original BOTH-alternative scalar string preference at
  cases.same.value, plus its op:text. Keep original preferred/fallback literals exact.
- Existing native-policy-package.cue is package-only companion for marked modules.

SEVEN raw CUE union: helpers/two ordinary bodies/three marked modules/package.
FIVE independent TWO-file native invocations: helpers+values, helpers+control,
package+each preference. No marked+definition/let/multiple-Or document is admitted.
33 root cases total (21+11+same), with the two other preference modules contributing
nested branch leaves, not duplicate root cases. Binary composer recurses shared
object namespaces at cases.integerPrimitive.cases; no op/field/leaf collisions.
No unresolved default/constraint/identity crosses materialization or composition.

Unchanged compose-policy.jq accepts exactly TWO objects; shared nonobjects collide
EVEN equal, arrays atomic. Use FOUR binary stages for five modules; repeat each
export and composition, verify all120 module orders with BOTH left/right folds.
This tests two associations, not all14 possible trees. Bound all source/module/
intermediate/final encoded sizes by existing64KiB cap before publication. No numeric
transform/overwrite/CUE meet/residual refinement or expected-answer construction.

## Production versus independent validation

New native-unify-lowering-policy package/check. Inputs ONLY seven raw CUE cut,
unchanged eight-source exporter, pinned jq and unchanged composer. Native
construction excludes Go/schema/full bundle/compiler/executor/oracles/generated
Rust/goldens/observations/expected source. Mandatory tools absolute/readable/
executable before cleared env/PATH unusable/owned HOME; failures publish nothing.

jsonExports exact canonical unify-lowering copies artifact/SKIPS Go. Go bridge
unify-lowering-fragments.cue keeps #UnifyLowering:#LoweringPolicy and requires ALL33
root case slots, aliases.choose plus ALL SEVEN named integerPrimitive branch slots
(including difference/divide). Select operation/field typing remains schema-owned.
Missing helpers is Unknown/setup, NOT semantic incompleteness. Missing either body/
any preference required slot is Incomplete/no JSON. Bridge is Go-only, not constructor.

Raw override witnesses load ALL SIX raw policy owners+bridge/schema/selector before
context: unify-lowering-permissive, integer-kernel-alternate, division-kernel-alternate.
Do not replace unresolved originals by selected JSON or silently freeze defaults.
Raw literal alternatives stay live; late unequal concrete defaults conflict.
Keep schema and typed compiler checks separate: malformed DATA may export natively
but must fail consumer without Rust. Include a schema-accepted/unbound-get DATA
variant rejected by actual typed policy validator. No error-triggered fallback.

## Independent evidence and interfaces

Evidence owner ONLY spec-export/tests/unify_policy.rs: complete independently
hand-authored trees for values/control/three preferences/full policy, not native/Go/
scratch captures. Use Cargo native CLI + existing pinned cue/jq discovery before
clear-env; NO new compiler/artifact provider ABI or full-compiler dependency in
Rust tests. Nix verifier owns actual canonical-artifact/consumer source effects.

- Compare native API/CLI/raw Go/schema/composer and production output collectively;
  exact33 root cases, nested five+two branch ownership, alias/version singleton,
  repeated/file-reversed/helper-renamed sources and all120 orders/BOTH folds.
- Retain existing composer collision/count/shape/exact-i64 assertions; test actual
  deep namespace assembly and duplicate equal nested leaves fail, not overwrite.
- Missing helpers setup distinct from body/preference incompleteness; every33 root
  slot, seven nested branches and alias.choose required. No wrong-schema-source
  failure counted as semantic evidence. Strict no partial JSON/diagnostic/recovery.
- All three raw overrides succeed, preference-source variants retain BOTH literals
  and produce independently predicted JSON; materialized late override conflicts.
  Whole ordinary body STILL exact Limit(evaluation steps)/no JSON. Mixed source
  definitions/marks and multiple marked modules stay Unsupported. No guard changes.
- Six raw owners (helpers/values/control/three preferences) independently affect
  whole canonical DATA, then actual unification compiler to COMPLETE predicted Rust.
  Retain all unification goldens and real compiled permissive/difference-as-product/
  divide-as-quotient controls. Emitted-source effects do not imply execution unless
  retained compiled controls actually run. Invalid opcode/shape schema+typed rejects
  and typed-only lexical failure must emit no Rust. Go/schema verifiers never seed.

Source inventories: raw7, exporter8, native core8/9, scanner7/8, experimental6 and
other raw cuts intact. ALL249 JSON/all12 builder artifacts/generated Rust/older
native bytes unchanged; all twelve older non-bundle constructor IDs unchanged
(bundle/legacy/joined/executor/verifier legitimately change). Fresh recursive
DERIVATION and --requisites --include-outputs retained graphs exclude current Go
CUE/output/source/full bundle/legacy+joined/current executor/BOTH oracle outputs-drvs/
provider environments from native construction. Runtime references alone insufficient.
Original core feature path remains native, scanner source-feature remains declared
Go-backed verification only. Preserve six fresh compilers and exact3+3 provenance.

Both full Rust suites/default strict all-target Clippy debug/release, exporter84+
new groups/four profiles, research135/four profiles/focused strict Clippy; BOTH
facade feature matrices/debug-release/fresh generation and separate derivative
rebuilds. SEVENTEEN checks/FOURTEEN packages actual build AND rebuild. Pinned preflight/
whitespace/relative links, two immutable final reviews, created workers/workspaces
retired BEFORE PR, protected exact-head rebase auto-merge/required CI through MERGED.
Only validated architectures claimed; approximately<=1500 changed lines incl data/docs.

## Ownership and implementation gate

Build owner ONLY flake.nix, unify-lowering-fragments.cue and SIX new raw policy files
named above. Evidence ONLY new tests/unify_policy.rs. Coordinator docs/integration/
source/baseline/full graph/validation/CI. No shared Rust/Cargo/composer/schema/provider
edits. At most two stable-main isolated workers; immutable saved handoffs, no saved
amendment/rebase onto mutable coordinator; new children for fixes/dependency overlays.
Format only owned files; preserve licenses/notices. No generated Rust changes.

Implementation GO: BOTH corrected focused source/refactoring/consumer and independent
semantic/evidence reviews approve interfaces/distinct ownership. First source review
found proposed #UnifySide/#UnifyEndpoint/#UnifyRelation collide with actual schema
definitions. BEFORE implementation names changed to collision-free
#UnifyLoweringSide/Endpoint/Conditional/Relation/L/R. Actual-schema canonical and all
three raw override probes match original-owner counterparts; original-owner-assisted
feasibility is NOT replacement-bridge completeness. Both reviewers explicitly
approve this correction; coordinator GO now permits ONLY this contract. Final
required-slot bridge proofs, immutable reviews and release gates remain mandatory.
Pause if partition/helper rewrite cannot preserve actual canonical/raw override
contracts without new semantics or raised guards. Accepted claim ONLY fourth native
policy DATA; not control/schema/4th compiler closure/full self-hosting/default-umbrella
completion/general CUE/performance/strict minimality or Go-free six-seed graph.

## Implemented evidence and local validation

Raw/build handoff `3db9b6655d4722bdbb646e6ead25a4ef74d3909c` gained actual consumer
verification in `9576d25aaa6fbf7e9c2e5707a0d3298d6d38bc81`. Initial pinned sandbox
preflight caught11 missing include_str test inputs; targeted test-only fileset child
`dc301bbf46bceb909640646011ae01db1e49e382` fixes that WITHOUT changing constructor
cuts. Independent tests `00db691aefea6fda3917722aa849f24e3b689c7c` gained idiomatic
strict-Clippy iteration/fixture factoring in `c81d63e20650e8fba8c4fa2146a3d8bd550211f0`.
Expected trees and required assertions stay intact; two Rust body-mutation checks
move collectively to stronger Nix actual-consumer complete-source effects. Saved
handoffs stay immutable; fixes/dependency overlays use new children/owned paths.

Four new groups independently pin33 roots,21/11/same ownership, nested5+2 branches,
alias/version, all120 orders/BOTH folds, deep equal-leaf collision rejection,
helper/literal-path renaming, required41 slots, missing-owner error classes,
BOTH-literal priority swaps/raw3 overrides/concrete conflicts, whole ordinary exact
Limit(no JSON), excluded mixed/multiple-mark profiles and recovery. Actual Nix
consumer matches all unification/override goldens and six COMPLETE predicted Rust
source effects. Schema+typed invalid opcode/shape and schema-accepted unbound-get
DATA correctly produce no Rust. Retained compiled adversaries execute separately.

All88 exporter tests/four profiles/strict all-target Clippy, both full Rust suites/
default all-target strict Clippy debug-release, research135/four profiles/focused
Clippy pass. Both genuine façade matrices/fresh modules and separately rebuilt
feature derivatives pass; scanner source-feature input graph remains Go-backed
verification ONLY. Pinned preflight retry passes13/13(seven skips); SEVENTEEN actual
checks/FOURTEEN packages build AND rebuild. Six fresh compilers/3+3 provenance/
profiles/drift/source/value retained. Source projection from #231 matches EXACTLY
after unquoted helper-reference rename/version1/21+11+3pref partition and outside-
quoted whitespace only. All249 JSON/all12 builders/generated Rust/old native bytes
unchanged; TWELVE older non-bundle constructor IDs unchanged.

Raw seven-source cut: `/nix/store/a49w4h70wswkmxfdds2hwly5w44j8nv6-source`;
canonical artifact14473 bytes. Exporter8/core8-9/scanner7-8/experimental6/other raw
cuts unchanged. Fresh recursive/full retained inventories match: four old native
1407/3392, AST1412/3410, Meta+scanner+unification DATA1411/3407, core1424/3437,
scanner compiler1423/3434, joined seed1450/3504. Native excludes current Go CUE/
source/full bundle/legacy+joined/current executor/BOTH oracle outputs-drvs/provider
environments. Original core feature graph stays native; scanner feature graph
POSITIVELY contains Go/source/bundle as declared verification. Four native DATA
inputs, THREE isolated compiler paths; TWO other policy constructors/joined seed
still Go-dependent. Only x86_64-linux validated; #175 OPEN. Final immutable release
reviews and required exact-head protected CI/confirmed MERGED remain release gates.
