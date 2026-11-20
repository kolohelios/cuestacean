# Native scanner compiler construction (#230)

## Selected dependency after #228

Scanner policy DATA is native, but bootstrap-scan-codegen still builds with the
legacy full-library/full-bundle constructor. Isolate this EXISTING binary using
unchanged scanner/lowering/module implementations and original CLI. Native AST/meta
core remains unchanged at eight production/nine test source files, two policies,
two binaries and its independently native feature-generation path.

Read-only scratch compile of the seven-source/two-policy scanner façade/all targets
passes without generated Rust/full executor/other policies. This is feasibility,
not independent Nix construction or semantic proof. No source/default/schema/solver/
VM/compiler algorithm/API/guard changes; no new logical compiler binary or fallback.
#175 remains OPEN. Three other policy constructors/joined seed remain Go-dependent.

## Exact source and feature boundary

Add bootstrap/native-scan-codegen, internal library named bootstrap_executor for
UNCHANGED CLI imports, not a drop-in executor API. Explicit SEVEN-source allowlist:

- native-scan-codegen/{Cargo.toml,Cargo.lock,src/lib.rs}: three new façade files.
- executor/src/{scanner,lowering,module}.rs: three UNCHANGED shared files via #[path].
- executor/src/bin/bootstrap-scan-codegen.rs: original UNCHANGED CLI Cargo target.

Keep locked versions; lock differs ONLY by root package identity. Preserve notices.
No copies/translations/source scraping, stubs, cfg/lint suppression, privileged
opcodes/filenames or separate golden/expected/captured inputs to construction.
Existing cfg(test) defensive source is not executed by constructor doCheck=false.
This is an exact allowlist, not mathematical source-minimality.

Default EMPTY features: scanner/meta policies interpreted, no generated files.
compiled-lowering genuinely activates fresh scan/meta drivers and shared self-alias.
source genuinely activates ALL27 existing generated lexical scanner wrappers, not
a CUE parser/evaluator/schema implementation. Keep four combinations meaningful,
executed/debug-release/strict all-target Clippy, without changing executor features.

IMPORTANT asymmetry: the 27 scanner PROGRAM specifications still need Go CUE.
Separate scanner feature-test generation may consume full Go-exported specification
JSON to produce fresh scanner Rust; compiled drivers come from native policies.
That entire test derivative is a VERIFIER, intentionally Go-dependent. It never
feeds the scanner constructor. Do NOT claim its full source-feature input graph is
Go-free. Existing AST/meta native-core feature path remains Go-free/unchanged.
No committed generated Rust may seed EITHER feature test build.

## Nix interfaces and production integration

Build owner changes only new façade Cargo/lib files and infra/tooling/flake.nix:

- nativeScannerSources(pkgs,includeTests): seven explicit files; test variant adds
  ONLY native-scan-codegen/tests/compiler.rs, EIGHT-source test cut.
- nativeScannerPolicies(pkgs): exact TWO-file directory containing independently
  native scan-lowering.json and meta-lowering.json. No full JSON/schema/compiler.
- nativeScanCodegen(pkgs): independent Rust constructor from seven-source cut and
  locked libraries; CUESTACEAN_BOOTSTRAP_JSON=nativeScannerPolicies; empty features,
  no generated Rust, tests/providers/final-executor/Go dependency. Do not inherit
  rustPackage and its full source/environment. Public package native-scan-codegen.
- legacyCodegenSeed now builds ONLY bootstrap-codegen/pratt-codegen/unify-codegen.
- codegenSeed joins existing native-core TWO + new native scanner ONE + legacy THREE.
  Assert SIX exact names and resolved origin for EACH. Production regeneration
  must actually invoke the new native scanner binary, not final-executor binaries.
- nativeScannerTests: separate eight-source test derivative. Generate all27 required
  scanners with nativeScanCodegen and explicitly Go-exported PROGRAM inputs; generate
  scan/meta drivers with native core compiler/native policies. Compile/execute all
  four scanner façade features/debug-release/strict all-target Clippy. These Go-backed
  test inputs are NOT a constructor dependency. No new runtime provider ABI.
- nativeScannerVerification: public check native-scan-codegen. Depends on scanner
  tests/full executor/goldens/Go as verification only. Compare all scanner goldens,
  Unicode alternate compiler control and all prior source/effect/typed failure gates.
  Retain roots/registration/full paths for SCANNER CONSTRUCTOR ONLY, not verifier.

Do not modify native-core source/manifests/source cuts/feature behavior. Constructor
policy DATA/builder/exporter/oracle/native-core/full JSON IDs and bytes must remain
unchanged (no CUE changes). Legacy/joined/executor/verifier identities may change.
Binary bytes legitimately differ; output/API/provenance equality is the criterion.
Old fifteen checks/twelve packages + new check/package = SIXTEEN/THIRTEEN.
No root-flake or generated orchestration edits; project.cue owns any such changes.

## Independent evidence

Evidence owner ONLY new native-scan-codegen/tests/compiler.rs. Independently authored
complete source/results/errors, not captures or copied shared-library expectations:

- Default/compiled scanner emission: full small Rust outputs; branch merging/inversion,
  repeat/break labels/flow facts, byte/character pattern ordering/escaping, Unicode
  advancement and cursor errors, custom-policy effects/validation precedence/recovery.
- Typed invalid/dead-arm/scoped-break/duplicate JSON, malformed/usage/custom-policy
  CLI failures require exit2/empty stdout/meaningful stderr, then success. Discover
  Cargo CLI absolute path, check readable/executable before clear-env, owned HOME,
  unusable PATH; setup failure is never semantic rejection. No new provider env.
- Real compiled scan/meta Driver/Context executions and shared cursor/interpreter
  results, not compile-only stubs. Real source feature executes ALL27 wrappers with
  independently expected offsets/flags/errors, including non-ASCII/EOF boundaries.
- Nix all-scanner formatted golden comparison/determinism/retained compiled Unicode
  advancement control plus full six compiler/profiles/drift/source/value checks.
  Expected files are comparison-only; fresh feature modules come from generation.
- Source seven/eight and policy TWO exact inventories; four shared Rust files exact,
  lock-only root rename. Existing exporter8/native-core8-9/experimental6/raw cuts intact.
- Fresh recursive derivation AND --requisites --include-outputs retained graph proof:
  scanner CONSTRUCTOR excludes current Go CUE/source/full bundle/legacy/joined seeds/
  final executor/BOTH oracle outputs-drvs/provider environments. Separate test/verifier
  may depend on Go/full bundle, never reverse the edge or call it construction-free.
- ALL249 JSON/all12 builders/generated Rust/older native bytes and all twelve older
  constructor IDs unchanged. Three compiler construction paths are independently
  native; three policy constructors/joined six-seed path still need Go CUE.

Both full Rust suites/default strict all-target Clippy debug/release, exporter84/
four profiles, retained135 research/four profiles/strict focused Clippy, original
native-core genuine feature matrix plus separate derivative rebuild. SIXTEEN actual
checks/THIRTEEN packages build AND rebuild; separately rebuild scanner feature-test
derivative. Pinned preflight/whitespace/relative links; exact head protected rebase
auto-merge/required CI through confirmed MERGED. Only validated architectures claimed.
Two immutable final cross-reviews; retire created workers/workspaces before PR.
Coherent slice approximately<=1500 changed lines including lock/tests/docs.

## Ownership and GO gate

Build owner ONLY native-scan-codegen/{Cargo.toml,Cargo.lock,src/lib.rs} and flake.nix.
Evidence owner ONLY native-scan-codegen/tests/compiler.rs. Coordinator docs/source/
artifact/full graph proofs/integration/validation/CI. At most two stable-main isolated
workers. Immutable saved handoffs, no amendment/rebase onto mutable coordinator;
new children for fixes/dependency overlays, integrate by owned path. Format owned files.

Implementation GO: BOTH focused source/Cargo/Nix and independent feature/consumer/
semantic-evidence reviews found no blockers and agreed seven/eight-source cuts,
two native policy inputs, genuine features, Go-backed scanner feature VERIFICATION
only, unchanged original native core and Cargo-CLI-only provider interfaces.
Explicit coordinator GO now permits ONLY this contract/distinct ownership.
Final immutable cross-reviews and all validation/release gates remain required. Pause if shared Rust/API/schema changes become needed.
Accepted after all gates: third existing compiler binary constructed independently
without Go CUE and used in actual production. NOT scanner PROGRAM/schema export,
Go-free source-feature verification, whole six-seed closure, general CUE support,
strict source minimality, broader defaults, self-hosting or performance.

## Implemented evidence and local validation

Build `cb8d67bcedd0d1d8d97a760b5018b7467e47c992` and one-file comment-only child
`ad7c526f09c5f88f5da589ce8be93dbacb264fe4` were integrated ONLY by four owned paths.
Independent test `d44f3b677beb` gained fixture corrections in `317f8d7e`/`2be5ec25`
and all27 EOF coverage in `db1796e8916b41ea2d5ac1a36f09421909a77394`, by test path.
Saved handoffs stayed immutable; overlays/fixes were new children. Strict failures
exposed wrong fixture expectations: whitespace stops at byte7, record-open only
advances/flagfalse, string-close sets flagtrue; raw CUE programs confirm them.
CLI diagnostic is exactly invalid lowering policy JSON. No assertion weakening or
shared implementation changes. Read-only semantic/source cross-reviews approve.

Actual four scanner feature profiles/debug-release execute42 shared units plus
4/5/5/6 independent integration groups, strict all-target Clippy. All27 wrappers
execute normal/shifted/invalid UTF-8 offsets,12 exact EOF rejects and15 exact EOF
successes; branch/flow/pattern/fuel/custom-policy/compiled-context/CLI controls pass.
Feature derivative was separately sandbox-rebuilt; all scannerFixture goldens,
Unicode alternate and retained policy source/effect/typed/full executor gates pass.

Both original full Rust suites/default strict all-target Clippy debug/release,
exporter84/four profiles, retained135 research/four profiles/focused strict Clippy
pass. Original native-core genuine feature path and separate sandbox rebuild pass.
Pinned preflight13/13(seven skips) initially runs17 flake checks; SIXTEEN enumerated
checks/THIRTEEN packages build AND rebuild. Six fresh compilers/provenance/full
profiles/drift/source/value remain. All249 JSON/all12 builder artifacts/generated
Rust/older native bytes and ALL TWELVE previous constructor IDs are unchanged.

Scanner constructor:
`/nix/store/b5b0p4jh7kz257kqs2qaj8axzqddin4d-cuestacean-native-scan-codegen-0.1.0.drv`.
Source seven: `/nix/store/0zf65nhn3v33grlhcfxhwi985mcwcv12-source`; test eight:
`/nix/store/xmhyzmmyn6sv5nrfzz7afrkfac6f75zw-source`; four shared Rust files compare
exact and lock differs ONLY by root package name. Policy directory contains ONLY
meta/scanner JSON. Existing core8/9/exporter8/experimental6 and raw cuts unchanged.
SIX joined paths resolve exactly native-core TWO + scanner ONE + legacy THREE.

Fresh recursive/full retained inventories match: four original native1407/3392,
AST1412/3410, Meta+scanner DATA1411/3407 each, original core1424/3437, NEW scanner
compiler1423/3434, joined seed1449/3501. Current BOTH oracle outputs/drvs/executor/
provider environments absent; native cuts additionally exclude Go CUE/source/full
bundle/legacy+joined outputs/drvs. Scanner constructor also excludes original core
compiler outputs/drv. Original core feature graph remains Go-free; scanner source-
feature graph POSITIVELY contains Go CUE/source/full bundle, intentionally verifier
only, with no constructor feedback. Only x86_64-linux validated. #175 stays OPEN;
final immutable release reviews and exact-head protected CI/confirmed merge remain.
