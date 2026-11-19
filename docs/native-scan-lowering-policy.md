# Native scanner-lowering policy DATA (#228)

## Concrete boundary after #226

Make canonical scan-lowering.json the THIRD complete native policy DATA input.
AST/meta and the two corresponding compiler construction paths stay unchanged.
Three other policy constructors/joined six-seed path still need Go CUE. This is
NOT native scanner schema evaluation or a third Go-free compiler constructor.
#175 remains OPEN; no new source/default/schema/solver/VM/algorithm/guard admission.

Read-only probe partitions actual source, uses the existing exporter and composer,
and matches complete baseline scanner policy bytes. The previous whole ordinary
body fails4096 work; do NOT increase limits or optimize charges to fit it. Scratch
success is feasibility, not construction/independent semantic evidence.

## Canonical raw ownership and precise refactoring

Existing scan-lowering-fragments.cue becomes ONLY the Go #ScanLowering bridge over
#LoweringPolicy requiring ALL15 original case slots. Canonical instruction data
has exactly one owner per case. New sources, under bootstrap/:

- scan-lowering-helpers.cue: five finite concrete named definitions #ScanStep,
  #ScanThen, #ScanElse, #ScanEmpty, #ScanNot, corresponding to the original five
  local let bodies. Rename only references; no helper-body or instruction changes.
  #ScanNot references #ScanEmpty. No cycles, marks, imports, patterns or schema.
- scan-lowering-flow.cue: ordinary brace-wrapped root, explicit formatVersion1,
  eight cases program/repeat/while/if/advanceChar/break/setFlag/reject, referencing
  shared definitions. No advance slot. No copied helper or expected-output seed.
- scan-lowering-predicates.cue: ordinary brace-wrapped root, six cases
  and/or/byteIn/charIn/flagIs/hasByte. No version leaf/case overlap; #ScanNot reused.
- scan-advance-policy.cue: ordinary root cases.advance with the UNCHANGED op:text,
  value:*"cursor.step()?;\ncursor.advance()?;\n" | string default expression.
- Existing native-policy-package.cue remains package-only companion for advance.

This intentionally changes helper binding syntax, not claiming unchanged source
bytes or general let/definition contextual equivalence. Original lets lived inside
#ScanLowering and inherited definition closure. New package definitions hold the
same finite concrete DATA, already supported by ordinary native definition lookup;
Go schema/raw construction and whole exported tree must match original. Closedness/
identity with arbitrary added constraints is NOT generalized from this relocation.
No declarations are silently dropped from a still-loaded unsupported source file.

THREE independent native invocations, each exactly TWO raw files:
helpers+flow, helpers+predicates, package+advance. Whole five-CUE raw union is not
one admitted marked/definition document. No unresolved default, binding/constraint
crosses assembly. All current syntax/evaluation/output bounds remain per invocation,
with no per-case reset or claim of one4096 ledger across independent programs.

Binary compose-policy.jq stays unchanged: exactly TWO materialized object modules;
recurse shared objects only, reject shared leaves EVEN equal, arrays atomic.
Compose three disjoint cases modules in two stages; verify repeats, all six module
orders and both associations. Check each module/intermediate/final encoded size
<=64KiB before publication. No numeric transformation, CUE meet/residual/default
refinement or fallback. JSON remains build-only, never captured/committed seeds.

## Production versus validation

New native-scan-lowering-policy package/check. Constructor inputs only explicit
five-CUE raw cut, unchanged minimal exporter, pinned jq and unchanged composer.
No Go/schema/bundle/executor/compiler/oracle/golden/observation/generated Rust input.
Mandatory tools absolute/readable/executable before clear-env execution, owned HOME,
unusable PATH, failure publishes no artifact. Direct raw files, no grep/extraction
or expected-answer construction. Source-derived native intermediates allowed.

jsonExports exact canonical scan-lowering membership copies native artifact/SKIPS
Go. Raw override fixtures (scan-lowering-unicode) explicitly load helpers+flow+
predicates+raw advance+bridge before selector/context. Concrete default JSON is
NOT a late-refinement interface. Keep BOTH alternatives/string type; raw source
variants/refinement work, late unequal concrete-default constraint conflicts.

Go schema/required-slot and typed lowering-policy/compiler validation remain in
separate verifier only; malformed source DATA may materialize but must fail there
without emitted Rust. Missing helpers is unknown/setup failure, NOT Incomplete or
valid negative-schema evidence. Missing flow/predicates/advance provider required
slots is Incomplete/no JSON. No error-triggered alternate provider or schema skip.
Existing scanner compiler native validation/flow/pattern kernels/guards/CLI stay.
All standard scanner goldens, compiled Unicode-advance adversary, full generated
driver drift and real source/value/profile checks remain. Do not claim generated
source effects were executed unless actually compiled/run by retained controls.

## Independent tests and closure proof

Evidence owner adds only spec-export/tests/scan_policy.rs. Its runtime interfaces
are Cargo's native CLI path and existing pinned PATH discovery for cue/jq before
clearing subprocess environments. NO new compiler/artifact environment-provider
ABI or full-compiler dependency in specTests. The separate Nix verifier owns actual
canonical-artifact comparison and scanner-compiler source effects/typed no-Rust
rejection. Requirements below are collective; do not duplicate consumer execution
inside Rust tests. Derive complete expected
flow/predicate/advance/full trees independently from DSL contract/raw instructions,
not native/oracle captures or expected files read by construction. Required:

- Full expected trees, actual production composer, native library/CLI, pinned raw
  Go/schema/canonical production output, repeats/file reversal/renaming and all
  assembly orders/associations; exact output. Existing collision/count/shape/i64
  composer assertions remain, no weaker replacement.
- All15 case coverage/eight+six+one ownership, version singleton, shared helpers
  correctly reused, no silent omitted slot/provider. Unknown helper setup failure
  remains distinct from missing body/default provider incompleteness.
- Canonical default and live raw Unicode advance override; native source preference
  variant retains string alternative and yields independently predicted value;
  raw late-vs-concrete default distinction. Do not widen marked/definition admission.
- Complete source-effect controls for shared helper, flow, predicate and advance
  owners drive actual scanner compiler to predicted emitted Rust, plus independent
  schema/typed no-Rust rejection. At least one native source/body/helper change
  must demonstrate instructions are not cached or answered by filename.
- Actual scoped missing/tool failure/no partial JSON/Rust/recovery; no setup failure
  accepted as semantic result. Old exporter80/four-profile assertions retained.

Source cut exactly FIVE raw CUE files, exporter eight, native core eight/nine,
experimental policies six; other existing raw cuts and artifacts unchanged. Fresh
recursive derivation AND --requisites --include-outputs retained inventories must
exclude current Go CUE/output/source/full bundle/current downstream executor,
legacy/joined seeds, BOTH current oracle outputs/drvs and provider environments
from native construction. Verifier/consumer may use those, never feed construction.
All older native data/exporter/oracle/native-core constructor IDs should remain
unchanged; bundle/executor/legacy/joined/verifier IDs may legitimately change.
ALL249 JSON/all12 builder artifacts/old native outputs/generated Rust byte-identical
with #227. Preserve six fresh artifact-free compilers/provenance/isolation/drift.

Both full Rust suites/default strict all-target Clippy debug/release; exporter and
retained135 research/four profiles; genuine native-core four features/debug-release
from fresh native-generated modules; pinned preflight/whitespace/relative links;
FIFTEEN checks/TWELVE packages actual build AND rebuild. Two immutable final cross-
reviews, retire created workers/workspaces, protected exact-head rebase auto-merge
and required CI through confirmed MERGED. Only validated architectures claimed.
Approximately<=1500 changed lines including moved data/tests/docs, no new goldens.

## Ownership and implementation gate

Build owner ONLY infra/tooling/flake.nix, scan-lowering-fragments.cue and FOUR new
raw scanner-policy files named above. Evidence owner ONLY new tests/scan_policy.rs.
Coordinator owns docs, source/projection/baseline/closure evidence, integration/CI.
Unchanged exporter/executor/core Rust/manifests/composer/CUE schemas/provider ABI.
At most two stable-main isolated workers; immutable saved handoffs, no saved
amendment/rebase onto mutable coordinator. New children for fixes/overlays; format
owned files only. Production files manually owned, not generated source extraction.

Implementation GO: BOTH independent focused source/refactoring/consumer and
semantic-evidence reviews approve this finite closed-helper/DATA boundary and
exact ownership. Both additionally agree the no-new-provider interface clarification:
Rust native/raw-Go/composer evidence, separate Nix actual consumer/source effects.
Explicit coordinator GO now permits ONLY this contract. Final immutable cross-
reviews and all validation/release gates remain required. Pause if helper refactoring does not preserve actual
canonical/override contracts or a partition needs new semantics/raised limits.
Accepted claim only third native canonical policy DATA construction; NOT native
scanner schema, scanner compiler seed closure, all-six closure, full self-hosting,
general CUE compatibility, default-umbrella completion or performance.

## Implemented evidence and local validation

Build `64f6ff4a77876a30c84792915de486e8040124c2` and typed-separation child
`4bcf148097ad54c525e548564eeb0ed10bf3b555` were integrated only by owned paths.
Independent tests `2df62ab5a8499678eb541f72393c618efca49716` gained meaningful strict
expect diagnostics in `4b597086ceb5902ff4bde8632cbfd2c131a44546` and whole-body limit
coverage in `f987bd35d4ad2bf301592ad32e21b295ed217975`. Dependency overlays were new
children, not saved handoff amendment/rebase. Cross-reviews approve unchanged
expected trees, source ownership, missing-provider classifications and typed checks.

Four independent test groups pin complete15-case trees, 8/6/1 ownership, all six
orders/both associations, provider/file renaming/reversal, required-slot failures,
raw Unicode preference versus concrete conflict, source mutations and recovery.
Combining ordinary owners STILL yields exact Limit("evaluation steps")/no JSON;
independent invocations do not raise/reset that guard. Separate Nix verifier matches
all scanner goldens/compiled Unicode control and four complete predicted Rust
source effects. Opcode/shape fail schema AND typed validation without Rust; an
unbound-get DATA variant passes native construction/Go schema but correctly fails
the typed compiler with unknown-binding/no Rust. This is not native full schema.

All84 exporter tests/four profiles, both full Rust suites/default all-target strict
Clippy debug/release, retained135 research/four profiles/strict focused Clippy pass.
Native-core genuine four-feature/debug-release tests use fresh native modules;
their derivative was separately sandbox-rebuilt. Pinned preflight13/13(seven skips)
initially runs16 flake checks. FIFTEEN enumerated checks/TWELVE packages build AND
rebuild; six fresh compilers/provenance/profiles/drift/source/value remain intact.

Source projection from #227 matches after ONLY helper-reference renaming,
version1/8+6+1 partition and whitespace OUTSIDE quoted strings. ALL249 JSON/all12
builder artifacts/generated Rust/old native output bytes match #227; all TEN older
exporter/native-artifact/oracle/core constructor IDs are unchanged. Only full bundle/
legacy/joined/executor/verifier IDs change. Raw five-source cut is
`/nix/store/gil0szdk01dn6msib898rswg2jhwwcal-source`; exporter/core eight, core tests
nine and experimental-policy six-file cuts remain unchanged. The native artifact
is13547 canonical JSON bytes. Composer/schema/exporter/shared Rust/CLI stay unchanged.

Fresh recursive/full retained inventories match: four original native1407/3392,
AST policy1412/3410, Meta and scanner policy1411/3407 each, native core1424/3437,
joined seed1446/3493. Current BOTH oracle outputs/drvs/executor/provider environments
are absent; native cuts also exclude current Go CUE/source/full bundle/legacy and
joined seed outputs/drvs. Three policies are native DATA, only TWO compiler
constructors isolated; scanner schema/compiler closure stays open. Only x86_64-linux
validated; #175 remains OPEN. Final immutable release reviews, required exact-head
protected CI/confirmed merge remain release gates.
