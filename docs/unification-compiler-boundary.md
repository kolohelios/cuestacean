# Unification compiler/runtime source boundary (#240)

## Actual next dependency

After #238, the unification compiler consumes native policy DATA but still builds
through the legacy full library. Its unification.rs mixes typed decoding/validation/
projection/lowering with runtime Value/root Error/execution. A narrow constructor
cannot include that file unchanged. Binary instruction-policy extraction separately
couples a comprehension with shared R defaults; do not silently widen admission.

Select a bounded ARCHITECTURE PR: extract the shared compiler-only core, preserving
all behavior and public paths. Do NOT claim a fourth constructor or change production
seed integration in this slice. FIVE complete native policy DATA inputs/THREE
isolated compilers remain; instruction policy/PROGRAM/schema work/joined seed still
need Go. #175 remains OPEN. Constructor isolation is a separately assessed follow-up.

Read-only scratch extraction of the actual ~650-line compiler portion compiles
all targets with unchanged lowering/module/CLI and native unify/meta policies, no
runtime module/root Error/stub. Initial probe found Side::validate_arity interleaved
in runtime; adding that unchanged validator closes compilation. This is feasibility,
not source-derived production generation, independent expectations, construction
closure proof, public package or integrated fourth binary. No source scraping in
production; new file becomes authoritative shared source.

## Exact production interfaces / ownership

Kernel owner ONLY executor/src/unification.rs and NEW
executor/src/unification/compiler.rs. No other production Rust/Cargo/Nix/CUE/CLI/
façade/schema/guard/feature/dependency/runtime.rs files may change.

- New compiler.rs owns original typed IR/serde/Snafu Error and discriminants,
  structural limits, parse, all validation (including Side::validate_arity),
  serialization-only unaryFail projection, generate/generate_with_policy and lower.
  Move original algorithms/text unchanged, except imports/explicit narrow visibility
  necessary for the existing parent execution/tests. No Value/root crate::Error or
  crate::runtime references in this source, no fake types/adapters/source suppression.
- Parent unification.rs loads private mod compiler via normal module path. Re-export
  EXACT existing public compiler API: Error, Endpoint, Kind, Relation, Failure,
  generate, generate_with_policy. Runtime primitives/interpreters stay in parent,
  their signatures/order/algorithms unchanged, using original crate::runtime::Value
  and crate::Error. No parallel copied IR or compiler implementation.
- Private IR types Side/Function/IntegerOperation/Expr/End/Condition/Program and
  ONLY necessary validation/parse/projection helpers become pub(super), not public
  API or pub(crate) by convenience. Program execution-needed fields only. The
  finite original serde tags/deny_unknown_fields/duplicate checks remain EXACT.
  Parent imports explicit names; no broad pub glob exposing implementation details.
- Existing embedded tests stay in parent, assertions/expected values unchanged.
  Adjust ONLY import/qualified helper paths after moves; do not hide tests behind
  new feature gates. Public CLI/Error paths remain bootstrap_executor::unification
  and existing root runtime Error. Debug source-module identities may naturally
  differ, but serialized wire, Display/source chains, validation/error precedence,
  metadata/source/values must not. Preserve function/type public reachability.

Production defaults and all feature behavior unchanged: default uses compiled
unify lowering, no-default/explicit policy reference uses existing interpreter.
Do not add a feature/cfg to suppress runtime dependencies or change rustPackage.
Keep one validation before selected backend, no fallback, lowered guards, altered
scope/arity/laziness, new instruction/wire/metadata aliases or source emitter.
Move helper code by exact source transformation reviewed independently; no rewrite
or expected-output construction disguised as refactor. Preserve Apache/notices.

## Independent boundary evidence

Evidence owner ONLY NEW executor/tests/unification_compiler_boundary.rs. Target
roughly60-100 lines: compile the SAME compiler.rs as a module in a test-crate root
that exposes only REAL existing lowering and, under the SAME genuine existing
compiled-lowering feature, the real compiled_unify_lowering alias. There is NO root
runtime module, root Error, Value or fake interface. Feature-dependent alias is not
cfg suppression; SAME tests execute default/no-default debug/release. This full-
crate test graph remains Go-backed verification, NOT a new native constructor.

Use an actual lowering::Driver/Context to independently pin complete unary/binary/
ternary metadata and default function/serialization-only inputValue/unaryFail facts,
not capture source or substitute serde JSON from old implementation. Check invalid
arity/dead-arm/scope/original-wire duplicates/aliases reject BEFORE an invalid policy
can run, and meaningful custom-policy effect plus success after failure. Real default
and reference entry points remain exercised by existing suites. Public enum serde/
Error paths and execution/offset/failure/interpreter tests stay meaningful.

No test-only production ABI, new tool provider, copied compiler/stub, weakened
assertions or lint/cfg/stack/recursion suppression. If the private source needs
runtime/root Error, broadened public API or suppressed warnings to compile, STOP and
resolve the interface before proceeding.

## Integration and release gates

Coordinator docs/integration/projection/full validation/CI. Two focused independent
source/API/visibility and semantics/evidence reviews plus explicit GO BEFORE edits;
immutable owned-path handoffs/new children for fixes/overlays, two stable-main isolated
workers max. Integrate ONLY owned paths, format ONLY owned files.

Strict source projection must account for EVERY moved compiler token and EVERY
remaining runtime token, allowing only documented imports/pub(super)/test helper
paths/module/re-exports/comments/whitespace. No algorithm/source literal changes.
All249 JSON/12 builders/old native DATA/generated Rust unchanged. ALL17 existing
PUBLIC package ctor drv identities expected unchanged (including exports; only
legacy/joined/executor/verifiers may change). Exporter9/core8-9/scanner7-8/other raw
cuts unchanged. No new public package/check or construction-count assertion.

All existing full Rust/default all-target strict Clippy debug-release, exporter109/
four profiles,135 research groups/four profiles; NEW boundary test executes all four
profiles with strict Clippy. BOTH genuine façades/fresh feature modules and separate
drv^* rebuilds. TWENTY checks/SEVENTEEN packages actually build AND rebuild. Fresh
FOURTEEN recursive/full retained inventories and forbidden Go/source/full bundle/
executor/legacyjoined/BOTHoracle out-drv/provider-env exclusions unchanged for native
constructors; original core feature Go-free, scanner source-feature Go-backed
verification ONLY. Compiler-boundary integration test itself is not native closure.

Pinned Nix/Shaka/full whitespace/links, immutable cross/final exact-tree reviews,
retire workers BEFORE PR, protected exact-head rebase auto-merge/required CI watched
through confirmed MERGED. Linux x86_64 only. Budget approximately1500 changed lines,
mostly ~650-line move (count BOTH additions/deletions), not permission to combine
full fourth-constructor implementation. Pause/reassess if materially larger; never
compress/delete evidence or expand API/guards to satisfy the estimate.

Implementation GO: BOTH independent focused reviews approve the single authoritative
compiler core, seven explicit public re-exports, minimal pub(super) parent sharing,
interleaved Side validator move, runtime-free same-source test root/real feature
aliases, unchanged execution/guards and bounded move budget. Coordinator GO permits
ONLY this architecture split; immutable cross/final reviews and all gates remain.

## Completed local evidence

Kernel e89e7906 and tests-only19d1f648 retain immutable parents. Exact source
projection accounts for ALL1943 original lines once, except shared module doc twice;
both projected files are byte-identical after pinned rustfmt and ONLY documented
imports/module/re-exports/pub(super)/test-helper paths. No algorithm/literal change.
Four new boundary groups execute EACH default/no-default debug/release with strict
Clippy; full Rust/default all-target Clippy both profiles/exporter109 four profiles/
research135 four profiles pass. Both genuine façade matrices/fresh modules and
separate drv^* rebuilds pass.20 checks/17 packages actually build AND rebuild;
preflight13/13 (7 skipped; cached Nix discovery0 not execution),1368-file whitespace/
602 relative links pass. All249 JSON/12builders/old DATA/generated Rust unchanged;
ALL17 public PACKAGE constructor drv identities unchanged INCLUDING exports.
Fourteen fresh recursive/full retained inventories MATCH; original native source
cuts unchanged, core feature Go-free/scanner source-feature Go-backed verification
ONLY. Same-source boundary test has no runtime/root Error/Value stubs but remains
full-library verification, NOT a fourth constructor or Go-free construction proof.
