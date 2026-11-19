# Native AST/lowering compiler construction (#226)

## Selected dependency and scope

After #224, `ast.rs::generate` and `lowering.rs::generate` embed only independently
native AST/meta policies. Their other shared implementation dependency is module.rs.
The current six-bin seed still builds the complete executor library/full JSON bundle.
Cut that packaging dependency, not a new CUE semantic class or compiler algorithm.

Read-only assessment at #225 compiles both existing CLI targets/all-target tests from
exactly eight selected source files and ONLY ast-lowering.json/meta-lowering.json.
No generated Rust, other policies, executor lib.rs or source parser are present.
Scratch success is feasibility, not accepted Nix construction evidence.

Add internal `bootstrap/native-codegen-core` Cargo façade. Library name
bootstrap_executor serves the unchanged CLI imports, not a drop-in executor API.
Re-export the EXACT existing three modules via #[path], not copies/translations:
executor/src/{ast,lowering,module}.rs. Point TWO Cargo bin targets at the original
executor/src/bin/{bootstrap-ast-codegen,bootstrap-lower-codegen}.rs. Original shared
Rust algorithms, CLI code/diagnostics, Cargo manifest/dependencies, resource guards,
CUE sources/composer/exporter and generated goldens remain unchanged.

Explicit production source allowlist (eight files):

- native-codegen-core/{Cargo.toml,Cargo.lock,src/lib.rs} (three new façade files).
- executor/src/{ast,lowering,module}.rs (three unmodified shared modules).
- executor/src/bin/{bootstrap-ast-codegen,bootstrap-lower-codegen}.rs (two unchanged CLIs).

Paths above are under bootstrap/. Lock exact existing dependency versions, adding
only the façade package identity. Keep Apache-2.0 notices. No source scraping,
algorithm duplication, fallback, stub, lint suppression or opcode/filename privilege.
Do not call eight files mathematically minimal; it is an exact allowlisted cut.
Existing cfg(test) defensive shared-unit source is not executed during construction;
separate fixtures/goldens/observations are not construction inputs.

## Feature and ABI contract

Façade default features EMPTY: artifact-free interpreted AST/meta policy execution.
Real source feature activates the existing eight pure JSON AST builder functions,
NOT a CUE source parser/evaluator. Real compiled-lowering feature activates ONLY
fresh generated AST/meta drivers, using the same self-alias as the executor.
Those two features are declared because shared modules genuinely implement them;
all four combinations must compile/test/strict-Clippy in debug/release. No empty
implementation of an advertised feature or unexpected_cfgs suppression.

Do not alter executor feature meaning or its six-target/full-Rust isolation suite.
Façade scope does not expose other four compiler frontends or evaluator APIs.
CLI usage/optional explicit custom-policy argument, failure exit2/empty stdout and
shared kernel/decoder/typed/context limits remain exactly the existing interfaces.
Neither binary may spawn an external fallback process. Checks use cleared runtime
environments, absolute readable/executable tools, unusable PATH and owned HOME;
provider setup failure never counts as semantic rejection.

## Nix construction and actual production consumer

New helpers in infra/tooling/flake.nix:

- nativeCoreSources(includeTests): explicit eight-file fileset; test variant adds
  ONLY native-codegen-core/tests/compiler.rs. No committed generated Rust/JSON.
- nativeCorePolicies(pkgs): exact directory with ONLY the canonical native AST/meta
  policy artifacts, copied without Go/schema/executor/answer inputs.
- nativeCodegenCore(pkgs): fresh Rust build from eight-file cut/locked libraries,
  CUESTACEAN_BOOTSTRAP_JSON=nativeCorePolicies, empty default features/no generated
  modules. No provider env; constructor check phase does not run test/expected code.
- legacyCodegenSeed(pkgs): old artifact-cleared constructor, builds ONLY remaining
  four existing binaries (bootstrap-codegen/scan/pratt/unify-codegen).
- codegenSeed(pkgs): explicit union/symlinkJoin of native two + legacy four, exactly
  SIX names, no duplicate target or fallback. Existing production regeneration
  consumes this joined seed, so actual AST/meta compiler invocations use native two.
- nativeCoreTests(pkgs): separate test derivative, nine-source test cut plus fresh
  generated files produced ONLY with nativeCodegenCore/native builder/policy inputs.
  Never copy committed Rust into a feature build. Generate eight required builders
  and both AST/meta drivers; all four façade profiles/debug-release/strict Clippy.
- Public package native-codegen-core and check native-codegen-core. Separate verifier
  may use Go/full-bundle/executor/goldens to compare all12 builder inputs/all6 drivers;
  none feed nativeCodegenCore/nativeCorePolicies/native construction feature inputs.

No root flake/orchestration hand edits. Any orchestration edit belongs to project.cue.
The shared joined six-compiler seed remains Go-dependent through the four legacy
policies. Joining does NOT authorize a six-seed Go-free claim. Constructor identity
changes are expected; all249 bundle JSON/all12 builder artifacts/generated Rust and
old native-artifact/exporter/oracle constructor identities must remain unchanged
in this build-only slice. Binary bytes may legitimately differ due to the smaller library/package;
emitted source/output/API equivalence and actual provenance are the requirements.

## Independent evidence and release gates

Evidence owner writes only new tests/compiler.rs, deriving expectations independently
from contracts/raw DSL/shared public APIs, not copying captured outputs. Required:

- Default interpreted and compiled AST generation: complete hand-authored small Rust
  output; ownership clone/final-use, exact i64/string escaping and custom-policy
  behavior, typed invalid binding/use errors and no partial CLI Rust.
- Fresh compiled AST/meta drivers execute through real shared Driver/Context APIs,
  not compile-only stubs. Source feature executes all eight real builders with
  independently expected JSON, without claiming a source-language evaluator.
- Lowering metadata/module typed rejection and complete generated-source controls;
  retained independent goldens and full executor tests are separate broader evidence.
- Mandatory CLI binary discovery from Cargo target env before clearing runtime env;
  both CLI custom-policy/default paths, malformed input/arguments and recovery.
- Nix independently compares all12 builder inputs: eleven ordinary builder goldens,
  plus literal7 against independently authored Rust (and forced99 against its real
  retained golden). Compare all6 driver goldens (meta self-regeneration included), determinism and
  existing typed/forced/previous/source-effect controls. Golden comparison is
  verification only; feature modules must come from native generation.
- Verify joined seed SIX exact names, native two resolve to nativeCodegenCore and
  other four to legacyCodegenSeed, with no final-executor/committed-artifact shortcut.
- Actual sandbox build AND rebuild all relevant checks/packages, retaining baseline
  thirteen checks/ten packages plus new check/package = FOURTEEN/ELEVEN. Both full
  Rust suites/default all-target strict Clippy debug/release, exporter80/four profiles,
  retained135 research/four profiles, feature isolation/drift/source/value all remain.
- Source inventory eight/nine and policy directory TWO exact files. Fresh recursive
  derivation and --requisites --include-outputs inventories of native constructor
  must exclude current Go CUE/output/source/full JSON, legacy four/joined seed,
  executor, BOTH oracle outputs/drvs and provider environments. Keep older native
  cuts and shared seed inventories correct. Runtime references alone are insufficient.
- ALL249 JSON/all12 builders/generated Rust and old native artifacts byte-identical
  to #225. Two final immutable cross-reviews, pinned Shaka preflight/whitespace/links,
  protected exact-head rebase auto-merge/required CI through confirmed MERGED.
  Retire created workers/workspaces before PR; approximately<=1500 changed lines.

Coordinator owns this contract/status/closure/runbook docs, integration/graph/source/
artifact proofs/CI. Build owner ONLY new façade Cargo.toml/Cargo.lock/src/lib.rs and
infra/tooling/flake.nix. Evidence owner ONLY new façade tests/compiler.rs. Stable
#225 workspaces, immutable saved handoffs; no saved amendment/rebase onto mutable
coordinator parents. New children for fixes/dependency overlays; at most two workers.

**Implementation GO:** two independent focused source/Cargo/Nix and consumer/
evidence/feature contract reviews found no blockers and agreed the eight/nine-file
cut, two policy inputs, genuine four feature combinations, unchanged shared APIs,
mandatory Cargo CLI paths and distinct ownership. Explicit coordinator GO now
permits ONLY this contract, including the eleven-goldens/literal7/forced99 distinction.
Final immutable cross-reviews and all validation/release gates remain required. Scope does not alter default/scanner/schema admission,
#175 obligations or Rust semantic kernels. If the source cut requires shared-module
algorithm/API changes, paused review is required rather than broadening this slice.

Accepted claim ONLY after those gates: two existing compiler binaries constructed
from the eight-source allowlist and two independently native policy DATA inputs,
with a recursively proven Go-CUE-free construction path, and used in production
regeneration. Not full six-seed closure, strict source minimality, native schema
validation, full self-hosting, general CUE conformance or performance.

## Implemented evidence and local validation

Build handoff `375442ad` and provider-hardening child
`93df06ca529d2ee8dea2043c1387d00b2f6c0a94` were integrated ONLY by their four owned
paths. Independent tests `7c2b508b9e00d1701cbe40804df4e59b4f00ffa4` and hardening
child `9816119298b925f8bf5000e4e2042cc7e8323ce7` were integrated ONLY by test path.
Saved handoffs were not amended/rebased; dependency overlays used new children.
Both immutable implementation/evidence cross-reviews approve provider prechecks,
strict typed CLI failures/recovery and unchanged independent expected source/JSON.

All four façade feature combinations execute29 shared unit tests plus5/6/6/7
independent integration groups respectively, debug/release, with strict all-target
Clippy. Fresh eight builders/two drivers are generated by native binaries/data;
no committed Rust enters feature construction. The full feature-test derivative
was separately sandbox-rebuilt. Both original full Rust suites/default strict
all-target Clippy debug/release, exporter80/four profiles, research135/four profiles
and strict focused Clippy pass. Pinned preflight13/13(seven skips) initially ran16
uncached flake checks; FOURTEEN enumerated checks/ELEVEN packages build AND rebuild.
Six fresh compilers/full executor profiles/drift/source/value checks remain intact.

Native compiler constructor is
`/nix/store/036aad0d25ck1lqrkiy23fcwx7vby1q2-cuestacean-native-codegen-core-0.1.0.drv`.
Exact eight-source cut: `/nix/store/vffqybv46n182g133ib16h688dhbcdnd-source`;
separate nine-source test cut: `/nix/store/1gfbni43x024p39mmvr52v3q07q7wd93-source`.
Shared five Rust files compare unchanged; Cargo.lock differs ONLY in root package
name. Native policy directory has exactly TWO files. All ten older artifact/exporter/
oracle constructor identities and bytes remain unchanged; ALL249 JSON/all12 builder
artifacts/generated Rust match #225. Joined SIX names resolve to native two/legacy
four exactly, not final executor binaries. Goldens/forced/previous controls pass.

Fresh recursive/full retained inventories match: four old native cuts1407/3392,
AST policy1412/3410, Meta policy1411/3407, **native compiler1424/3437**, joined seed
1445/3490. Both current oracle outputs/drvs, executor/provider environments are
excluded. Native graphs additionally exclude current Go CUE/source/full bundle,
legacy-four/joined-seed outputs/drvs. Fresh feature-test construction inputs also
pass those exclusions; its expected-code tests remain verification, not construction.
Only x86_64-linux validated. #175 remains OPEN; final immutable release reviews and
required exact-head protected CI/confirmed merge remain release gates.
