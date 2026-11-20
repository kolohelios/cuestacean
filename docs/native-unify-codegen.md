# Native unification compiler construction (#242)

## Selected boundary after #240

The single authoritative unification compiler core now exists independently of
runtime Value/root Error. Its binary still builds via the legacy full library and
Go-backed full policy bundle. Isolate this EXISTING binary, not another compiler
or a new source-language feature. Five complete native policy DATA inputs remain;
success adds the FOURTH isolated compiler path, leaving two legacy binaries and
the joined seed/PROGRAM/schema/instruction-policy work Go-dependent. #175 stays OPEN.

## Source, Cargo and feature contract

Add internal bootstrap/native-unify-codegen, library named bootstrap_executor for
the UNCHANGED CLI. Exact SEVEN-source constructor allowlist:

- native-unify-codegen/{Cargo.toml,Cargo.lock,src/lib.rs} (three new files).
- executor/src/{unification/compiler,lowering,module}.rs (three unchanged shared
  sources, loaded via #[path]; compiler core exposed under unification).
- executor/src/bin/bootstrap-unify-codegen.rs (unchanged original CLI Cargo target).

No parent unification/runtime/root Error/Value/executor lib/generated files, copies,
source extraction, stubs, cfg/lint suppression, extra public IR or changed shared
sources. Lock differs from existing narrow façade ONLY by root package identity;
same locked dependencies/deny unwrap_used/Apache notices. This is an explicit cut,
not mathematical source minimality. Constructor doCheck=false does not run shared
defensive tests; expected/captured outputs are never construction inputs.

Default features EMPTY, interpreted native unify/meta policies only. Sole opt-in
compiled-lowering genuinely activates freshly generated unify/meta drivers plus
existing self-alias. Execute BOTH feature settings debug/release (four profiles),
strict all-target Clippy. Do NOT invent a no-op source feature: unlike the scanner,
this compiler-only core has no lexical wrappers or runtime execution API. Existing
core/scanner four-way feature matrices remain unchanged. Verification tests actual
compiled unify/meta Driver/Context and complete compiler outputs, not compile-only
fixtures. No generated module is copied from committed Rust into feature builds.

## Nix / production interfaces

Build owner ONLY new façade Cargo/lock/lib plus infra/tooling/flake.nix:

- nativeUnifierSources(pkgs,includeTests): seven explicit sources; test derivative
  adds ONLY native-unify-codegen/tests/compiler.rs (EIGHT-source cut).
- nativeUnifierPolicies(pkgs): exact TWO native JSON files, unify-lowering.json and
  meta-lowering.json. No full bundle/schema/Go compiler or additional policy.
- nativeUnifyCodegen(pkgs): standalone buildRustPackage, NOT rustPackage inheritance;
  seven sources/locked dependencies/empty features/nativeUnifierPolicies, no tests,
  generated seeds/full executor/Go/provider environment. Public native-unify-codegen.
- legacyCodegenSeed builds ONLY bootstrap-codegen and bootstrap-pratt-codegen.
  codegenSeed joins core TWO + scanner ONE + unifier ONE + legacy TWO. Preserve SIX
  exact binary names and resolved-origin assertion for EACH, including unifier.
  Production regeneration must actually use this unifier, not final executor.
- nativeUnifierTests: eight sources, fresh unify/meta drivers generated using the
  native core lower compiler and two native policies in cleared environment. Owned
  HOME/unusable PATH; validate absolute/readable/executable providers beforehand.
  Four real profiles+strict all-target Clippy; graph independently Go-CUE-free.
- nativeUnifierVerification: public check, depends on feature tests/retained native
  unification-policy verifier/full executor and all relevant old goldens, strictly
  verification only. Generate ALL unifyFixtures deterministically and compare
  formatted complete Rust, plus scoped/right controls and permissive-bool,
  difference-as-product/divide-as-quotient compiler overrides. Replay ALL SIX raw
  source-owner effects through the NEW native compiler against their independently
  predicted complete Rust: retain each materialized variant policy/tiny control in
  the old policy VERIFIER output, never feed it into constructor. Retain compiled
  execution controls in full executor; emitted-source comparison alone is not
  semantic execution. Copy constructor roots/registration/paths for graph auditing.

All shared Rust/CLI/CUE/API/features/guards/wire/validation/laziness/public paths
unchanged. No root flake or generated orchestration edits; project.cue is authority.
All17 previous PUBLIC package constructor IDs INCLUDING exports, all249 JSON,
12builders/old native DATA/committed generated Rust remain unchanged. Legacy/joined/
executor/verifier IDs may change; binary bytes need not equal old full-library
build. Old source cuts unchanged: exporter9/core8-9/scanner7-8 and all policy raw cuts.
21 checks/18 packages after adding exactly one public check/package.

## Independent evidence and release gates

Evidence owner ONLY native-unify-codegen/tests/compiler.rs. Independently authored
full Rust expectations for small unary/binary/ternary/default programs and meaningful
custom policy facts, arity/inputValue/unaryFail, integer/endpoint/layout/escaping;
not captured answers or a second read of the same policy. Validate dead arms/locals/
all six serialization-only aliases/duplicate original wire/invalid arity before
bad policy; preserve errors and recovery. Pin existing structural bounds without
claiming the original control guard streams/counts overwritten duplicate values. Existing #240 same-source metadata test
remains unchanged; new Cargo tests cover the real façade and real CLI.

Cargo-discovered absolute CLI/readable+executable check BEFORE env_clear; owned HOME,
unusable PATH. Usage/read/typed/custom-policy negatives: exit2/EMPTY stdout/exact
meaningful diagnostics, then success. No provider ABI. Real compiled Driver/Context
controls for BOTH modules when enabled; keep same core tests under both settings.
No runtime Value stubs or fake execution wrapper. Runtime correctness remains the
unchanged executor/retained generated compiled controls, not a façade runtime API.

Fresh recursive derivation AND --requisites --include-outputs inventories for all
retained14 graphs plus NEW compiler (15 total); compare outputs/drvs/current Go
CLI+source+source drv/full JSON bundle/executor/legacy+joined/BOTH oracle out+drvs/
provider env exclusions for native constructors. Also prove native unifier feature
generation Go-free; original core feature stays native, scanner source-feature is
positively Go-backed VERIFICATION ONLY. No verifier dependency feeds constructor.
Inspect exact selected store source inventories and BOTH policy files.

Two independent focused contract reviews and explicit coordinator GO BEFORE edits.
Stable-main isolated workers, distinct ownership; immutable handoffs/new child fixes
and dependency overlays, integrate ONLY owned paths. Coordinator owns docs/integration/
source/artifact/graph proof/full validation/CI. At most two active workers; retire
created workers/workspaces BEFORE PR. Immutable cross/final exact-tree reviews.

Full executor/default all-target strict Clippy debug/release, exporter109/four,
research135/four, unchanged boundary four groups/four, all THREE real narrow façade
matrices/fresh modules and separate explicit drv^* feature rebuilds. Actually build
AND rebuild ALL21 checks/18 packages; pinned preflight/whitespace/relative links.
Protected exact-head rebase auto-merge, watch required CI through confirmed MERGED.
Linux x86_64 only; no performance/minimality/conformance/self-hosting/default claim.
Bounded vertical slice normally <=1500 additions+deletions including lock/tests/docs.
Pause for changed shared source/API, unproved native feature graph or larger scope;
never weaken evidence, lower guards or compress tests to satisfy an estimate.

Implementation GO: BOTH independent focused reviews approve the exact source/Cargo/
Nix and independent semantic/CLI/feature-evidence contracts. Four-profile scratch
compilation executes28 shared tests per profile with strict all-target Clippy and
fresh native drivers, no shared-source edits/stubs/suppression. Feasibility is not
sandbox closure. Reviewer correction requires all six raw source-owner effects
through the NEW façade, retaining variants only in verifier output. Coordinator GO
permits ONLY the interfaces/ownership above; immutable cross/final reviews and ALL
validation/CI/merge gates remain. Pause rather than silently expand scope.

## Implemented evidence

Immutable build5186d1b4, tests c257110b, test correction2ad6e0f5 and build children
bd21f788/b78bb4f0 were integrated ONLY by owned paths. Initial boundary fixture
counted514, not512: three shorter conditions reduce six scalar nodes; exact512/513
assertions remain. Cross-review also caught a copied no-op source feature: removed,
not admitted. Strict feature shell is locally scoped/fail-fast. Earlier failures/
superseded runs are NOT passing evidence. Shared Rust/CLI/CUE/old façades unchanged.

Actual new feature profiles run28 shared tests EACH plus6 interpreted/7 compiled
integration groups EACH debug/release, strict all-target Clippy four profiles. Both
fresh compiled drivers execute real Driver/Context; Cargo rejects unknown source
feature (setup evidence, not semantics). All control goldens/three alternate compiler
policies/SIX owner-source effects deterministically match independent complete Rust;
existing actual full-executor compiled execution controls remain green.

Full Rust/default all-target strict Clippy debug/release, exporter109/four,
research135/four and boundary4/four pass. ALL21 checks/18 packages actually build AND
rebuild. Pinned preflight13/13 (7 skips;22 discovery entries, not check executions),
1373-file whitespace/607 relative links pass. All17 previous public PACKAGE drv IDs
INCLUDING exports/249JSON/12builders/old DATA/generated Rust unchanged. Six actual
compiler origins are core2/scanner1/unifier1/legacy2. Three genuine façade matrices
use fresh modules; separate explicit drv^* feature rebuilds are required above.

New constructor h1bkq13a has exact7-source js5yl1lg cut; feature c04245pr has exact8
fx77kqb7 cut; TWO native policies c5prial9. Four shared files compare byte-exact and
lock differs ONLY by package root name. Fifteen fresh recursive/full inventories
MATCH retained; unifier1423 derivations/3434 full paths, joined1456/3521. Current Go
CLI/source/source drv/full bundle/executor/legacyjoined/BOTHoracle out-drvs/provider
envs excluded from native construction; interpreted unifier also excludes sibling
compilers. Unifier feature inventory MATCHES retained and independently excludes
Go; original core feature remains native, scanner source-feature positively Go-backed
VERIFICATION ONLY. Five native DATA/four compiler binaries, not full seed closure.

## Subsequent shared source boundary (#244)

The [Pratt compiler boundary](pratt-compiler-boundary.md) moves the exact existing
Kind enum into authoritative unification/kind.rs, re-exported at the same public
paths. Current native unifier source/test cuts are therefore EIGHT/NINE, adding
ONLY that real file to the original seven/eight above. Policies, Cargo/features,
CLI/VM/compiler algorithms and all six binary origins remain unchanged. Its public
constructor identity legitimately changes; the seventeen other package identities
stay unchanged. Fresh native construction/feature/rebuild/graph proofs are required,
not permission to reuse the historical seven-source result as current evidence.
