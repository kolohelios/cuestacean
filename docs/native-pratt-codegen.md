# Native Pratt compiler construction (#246)

## Selected dependency after #244

The authoritative compiler-only Pratt core and shared Kind source now exist; the
existing bootstrap-pratt-codegen still builds via the full legacy executor/bundle.
Isolate this EXISTING binary, no new compiler algorithm or source language feature.
Success adds the FIFTH native compiler binary; FIVE complete native policy DATA
inputs remain. Only bootstrap-codegen and the joined seed/full instruction policy/
PROGRAM/schema work remain Go-dependent. #175 OPEN/held prototype untouched.

## Exact source / Cargo / feature interfaces

New internal bootstrap/native-pratt-codegen, library bootstrap_executor for the
UNCHANGED CLI. Exact EIGHT-source constructor allowlist:

- native-pratt-codegen/{Cargo.toml,Cargo.lock,src/lib.rs} (three new façade files).
- executor/src/{pratt/compiler,unification/kind,lowering,module}.rs (four UNCHANGED
  shared sources loaded via #[path], Kind under unification, core under pratt).
- executor/src/bin/bootstrap-pratt-codegen.rs (original unchanged Cargo binary).

No parent pratt/unification compiler/runtime/parser/root Error/Value/frame source,
full library, unifier programs, copies, duplicate Kind, stubs, source scraping,
public IR/wrapper/test ABI, cfg/lint/stack/recursion suppression or fallback.
Same locked dependencies/deny unwrap_used; lock differs from existing narrow façade
ONLY root package identity. Preserve Apache/applicable notices. Exact allowlist,
not mathematical source minimality; constructor doCheck=false never runs expected
answers. Committed generated Rust/JSON/goldens/oracles are NEVER construction seeds.

Default EMPTY features; sole opt-in compiled-lowering genuinely activates fresh
pratt/meta drivers plus self-alias. NO source feature (this pure core has no runtime
parser or lexical wrappers). Both settings debug/release =FOUR real profiles with
strict all-target Clippy. Same independent groups in both settings, plus actual
compiled Driver/Context execution when enabled. No shared source/API/Cargo/feature/
CUE/CLI/guard changes. Existing THREE façade matrices remain unchanged.

## Nix / production integration

Build owner ONLY new façade Cargo/lock/lib and infra/tooling/flake.nix:

- nativePrattSources(pkgs,includeTests): exact8 sources, test variant adds ONLY
  native-pratt-codegen/tests/compiler.rs (NINE-source cut).
- nativePrattPolicies(pkgs): ONLY independently native pratt-lowering.json and
  meta-lowering.json; no full bundle/schema/other policy/native unifier program.
- nativePrattCodegen(pkgs): standalone buildRustPackage, NOT rustPackage inheritance;
  source8/locked dependencies/no-default-features/nativePrattPolicies/no tests or
  generated/full-executor/Go/provider dependency. Public native-pratt-codegen.
- legacyCodegenSeed now builds ONLY bootstrap-codegen. codegenSeed joins existing
  core TWO + scanner ONE + unifier ONE + Pratt ONE + legacy ONE. Assert SIX exact
  names and resolved origin for EACH. Production regeneration actually invokes
  the new native Pratt binary, not a full executor substitute.
- nativePrattTests: source9 derivative, fresh pratt/meta drivers via existing native
  core lower compiler and ONLY nativePrattPolicies; cleared environment, owned HOME,
  unusable PATH, absolute/regular-file/readable/executable checks BEFORE invocation.
  Strict locally scoped set-euo-pipefail; four profiles/all-target Clippy. Prove
  independently Go-CUE-free feature generation. Never copy committed generated Rust.
- nativePrattVerification: new public check. Verification-only deps on feature
  tests/retained Pratt policy verifier/full executor/old goldens. ALL prattFixtures
  (five target families/primary/return/fuel/postfix/scoped alternatives), repeats,
  formatted full-source comparisons, existing no-increment alternate golden and
  actual compiled overflow/right-nesting controls retained in executor.
- Retain materialized tiny.json/three owner variant policies and malformed policies
  in OLD Pratt policy VERIFIER output, never constructor inputs. NEW native binary
  must replay ALL THREE raw-owner effects vs existing independently predicted
  complete Rust (body/add/tables), no-increment override, and typed opcode/shape/
  unbound policy errors with exit2/empty stdout/stable diagnostic. Source comparison
  alone is not semantic execution. Retain constructor AND feature roots/paths/
  registration; independent recursive/full/provider-environment audits.

ALL18 previous NON-DEFAULT PUBLIC package constructor drv identities INCLUDING
exports/native unifier unchanged (default aliases executor and may change); all249 JSON/12builders/old DATA/generated Rust unchanged. Old
source cuts exporter9/core8-9/scanner7-8/unifier8-9/all raw cuts unchanged. Legacy/
joined/executor/verifier IDs may change. Exactly ONE added public package/check:
22 distinct checks/19 non-default packages (default check alias adds discovery).
No root flake/generated orchestration edits; project.cue remains authority.

## Independent consumer / semantic evidence

Evidence owner ONLY native-pratt-codegen/tests/compiler.rs. Independently authored
complete tiny Rust for ALL FIVE wrapper targets, canonical actions/selector/typed
numeric calls, actual early/tail return/if/while layout, exact quoted/meta data.
Check genuine custom policies observing target/tail/body/Kind and changing source,
validation-before-ignored-policy, recovery. Existing #244 complete metadata/Kind/
source preservation boundary groups remain unchanged, not copied as new proof.

Typed dead-family arms/nested conditions/definite-return reachability/original-text
duplicates/internal action/selectorAction/parseMinimum and forged position/tail/
malformed target/u8/i64 reject before invalid compiler policy. Pin original depth0/
32 and512 structural guard if adding boundary controls; count independent fixtures
explicitly, not by captured source or changing guard. Do not claim streaming
original-control duplicate accounting. Expected layout is comparison-only.

Real Cargo CLI absolute regular/readable/executable setup checked BEFORE env_clear;
owned HOME/unusable PATH. Usage/read/typed/invalid custom-policy negatives exit2,
EMPTY stdout/stable exact diagnostics (read OS suffix not over-specified), then
success/default/custom output. Setup failure is not a semantic negative. No new
provider ABI. Compiled feature tests call BOTH real fresh Driver/Context instances,
including success → error → success for EACH driver independently; no runtime/frame/
Value simulation.
Independent runtime/source semantics remain unchanged full executor/genuine compiled
alternate controls, not a new façade runtime API.

## Reviews / release gates

Two independent focused construction/API/Nix and evidence/feature/semantics reviews
plus explicit coordinator GO BEFORE implementation. Distinct stable-main isolated
workers, max2; immutable owned-path handoffs/new-child fixes/dependency overlays,
never amend/rebase saved handoffs onto mutable coordinator. Integrate ONLY owned
paths; format owned files only (skip_children=true for path modules). Coordinator
owns docs/integration/full artifact/source/graph/validation/CI. Retire created
workers/workspaces BEFORE PR; immutable cross/final exact-tree reviews afterward.

Full executor/default strict all-target Clippy debug/release, exporter109/four,
research135/four, BOTH unification+Pratt boundary4groups EACH four; ALL FOUR genuine
façade matrices/fresh modules and separate explicit drv^* feature rebuilds. Actually
build AND rebuild ALL22 checks/19 packages. Pinned preflight/full whitespace/relative
links. Preserve all18 old public package IDs/bytes/six binaries/provenance and exact
source inventories by exact sorted relative MEMBERSHIP, not counts alone; lock-only
root rename/four shared files/original CLI byte-exact.

SIXTEEN fresh recursive derivation AND full --requisites --include-outputs retained
inventories. Native constructors exclude current Go CLI/source/source drv/full
bundle/executor/legacy+joined/BOTHoracle output+drvs/provider environments. New
interpreted Pratt constructor additionally excludes all THREE sibling constructors
outputs/drvs. Native core/unifier/Pratt feature generation Go-free (new Pratt retained
feature inventory MATCHES); scanner source-feature positively Go-backed VERIFICATION
ONLY. No verifier edge feeds construction. Source cut is exact, not proven minimal.

Protected exact-head rebase auto-merge/watch required CI through confirmed MERGED.
Linuxx86_64 only; no general CUE/default/schema/self-hosting/performance/minimality
claim. Target coherent <=1500 changed lines including lock/tests/docs. Pause for
shared source/API/feature/guard/ABI changes, unresolved validation or larger scope;
never delete/compress independent evidence to hit an estimate.

Implementation GO: BOTH independent focused reviews approve the source/Cargo/Nix/
closure/provenance and independent feature/CLI/semantic-evidence interfaces above.
Coordinator GO permits ONLY those owned paths. Retain variants/malformed policies
inside their individual loops (never overwritten/captured construction seeds),
exact relative source MEMBERSHIP, regular provider file checks, all18 NON-DEFAULT
package identities and EACH fresh driver's success/error/recovery independently.
These reviews establish static feasibility only, not scratch/sandbox/native closure
or release validation. Immutable cross/final reviews and all gates remain mandatory.

## Completed local evidence

Immutable build ac812294 and test-only3c7fb803 were integrated ONLY by owned paths;
validation dependency overlays are separate children. No shared Rust/CUE/CLI/
existing façade changes or evidence fixes. Source8/test9 membership and all four
shared modules/original CLI byte-identical; lock differs ONLY by root package name.
Native policy directory contains ONLY unchanged pratt/meta DATA. All18 previous
NON-DEFAULT package constructor drv IDs INCLUDING exports/native unifier unchanged;
all249 JSON/12builders/old DATA/generated Rust unchanged. Actual six origins resolve
core2/scanner1/unifier1/Pratt1/legacy1. No full executor substituted for native Pratt.

New feature derivative executes28 shared units EACH plus7 empty/8 compiled
independent groups EACH debug/release, strict all-target Clippy4. Both fresh drivers
independently execute real Context success/error/success; CLI setup/read/typed/
custom negatives and all five complete wrapper/layout/private-metadata predictions
pass. New native verifier matches ALL fixtures/no-increment/THREE owner-effect
complete independent Rust predictions and exact typed malformed-policy diagnostics.
Actual compiled no-increment overflow/right-nesting/full executor controls remain.

Full Rust/default all-target strict Clippy debug/release/exporter109 four/
research135 four/both old boundary4groups EACH four pass. ALL22 distinct checks/
19 NON-DEFAULT packages actually BUILT AND REBUILT; all FOUR genuine façade matrices/
fresh modules plus separate explicit drv^* rebuilds pass. Pinned preflight13/13
(7 skips;23 discovery entries, not check executions),1382-file whitespace/613 relative
links pass. Discovery/cache counts are not substituted for actual build execution.

Native constructor pidzmsg8 has source8 rbv8ia79; feature4nn5fjc1 uses source9
q3v1lcdk; native two-policy directory0hrvx6hk. Sixteen fresh recursive/full inventories
MATCH retained: Pratt1424derivations/3437full paths, joined1459/3529. Current Go
CLI/source/source drv/full bundle/executor/legacyjoined/BOTHoracle outputs-drvs/
provider env exclusions pass; Pratt constructor also excludes ALL THREE siblings.
Pratt feature inventory MATCHES retained/1430derivations3454full paths and is
independently native, as are original core/unifier features; scanner source-feature
positively Go-backed VERIFICATION ONLY. FIVE native compiler binaries/FIVE DATA,
not full seed/PROGRAM/schema/instruction-policy/default closure. Only Linuxx86_64
validated; immutable final exact-tree review/retirement/protected CI/confirmed merge
gates remain mandatory.
