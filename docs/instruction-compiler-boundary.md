# Instruction compiler/runtime source boundary

## Selected dependency after #256 (#258)

The remaining `bootstrap-codegen` compiler builds the full executor because its
46-line compiler imports root-owned Rules/Rule/Instruction/parse_rules/Error.
Separate that real typed compiler boundary before considering a sixth façade.
This slice does NOT construct the sixth binary or finish instruction-policy DATA.
FIVE complete native policies/FIVE independently native compiler binaries remain;
full instruction policy/compiler, joined seed and PROGRAM/schema still need Go.
#175 stays OPEN/HELD. No native language/default/schema/admission/guard change.

Read bootstrap/lowering.md, bootstrap/module.md and the prior Pratt/unification
boundary contracts. This is an architecture slice, not more routine source syntax.

## Exact source and API contract

Kernel owner ONLY executor/src/lib.rs, executor/src/codegen.rs and NEW
executor/src/codegen/compiler.rs. No runtime.rs/lowering.rs/module.rs/CUE/Cargo/
CLI/façade/source-allowlist changes. Coordinator owns this contract/checkpoint;
evidence owner ONLY NEW executor/tests/instruction_compiler_boundary.rs.

Move ORIGINAL Rules, Rule, Instruction and parse_rules from lib.rs, and ORIGINAL
compiler generate/generate_with_policy/emit from codegen.rs into compiler.rs.
Preserve every variant/serde attribute, BTreeMap storage, original-text decoding,
format-version ordering, typed projection, selected Driver/Frame calls, embedded
policy paths, cfg branches and error timing. No copied decoder or instruction
match, generic JSON bypass, re-rendering algorithm, fallback or new budget.
Rules/Rule/Instruction/parse_rules are crate-private ONLY. Rules.format_version
stays PRIVATE; ONLY Rules.rules and Rule.result become crate-private fields for
the real runtime. Do not expose public IR convenience APIs.

The new core owns a REAL three-variant compiler Error: Json(document, source),
Version(document, version), Lowering(source), with original diagnostic spelling
and concrete serde_json/lowering source types. It imports ONLY real serde/Snafu,
lowering and module APIs, plus genuine compiled_lowering under compiled-lowering.
NO root Error/parse_rules/runtime/Pratt/unifier/frame/value stubs or provider ABI.
Root Error's existing variants remain authoritative for its other real runtime
consumers; do not drag their Pratt/unifier dependencies into the compiler core.

codegen.rs becomes the small public compiler façade (`pub mod codegen` is an
explicit ADDITIVE module exposure); `codegen::compiler` exposes Error/generate/
generate_with_policy. Existing root generate/generate_with_policy and existing
CLI still return EXACT root Error. An exhaustive owned From<compiler::Error>
conversion unwraps the three real sources into the corresponding ORIGINAL root
variants, preserving document/version, concrete source and Display/source chain;
no boxed wrapper or catch-all classification. Root parse_rules is a thin adapter
for real runtime callers. Explicitly gate this runtime-only adapter AND its
Instruction/Rule/Rules imports with cfg(feature = "semantics"), alongside the
semantics-only BTreeMap import: after delegation their ONLY consumers are the
existing semantics entry points. Core parse/typed compilation stay unconditional.
This reflects actual consumers, not lint suppression or feature-meaning changes.
Remove/move other now-unused imports genuinely; never suppress lints.

Only documented imports/module declarations/visibility, the three real compiler
error definitions and bijective adapter, and thin legacy API forwarding may differ
from original source. Original execution/binary/slice helpers and every runtime
instruction arm stay byte-exact after pinned formatting. Preserve Apache notices.
Original public root APIs, Error variants/sources, feature meaning, CLI syntax,
optional trusted policy override, exit2/EMPTY stdout and recovery are unchanged.

## Independent boundary evidence

Compile SAME compiler.rs in an integration-test root exposing ONLY real lowering,
module and actual compiled_lowering under its genuine feature. No root Error or
runtime/helper stubs. Full-library linkage is Go-backed VERIFICATION, not proof of
a new native constructor. If source separation needs broader dependencies or
behavior changes, stop and revise the contract before implementation.

Four focused groups, without compressing proof to meet a guessed line count:

1. Complete independently authored typed-wire/projection inventory for ALL45
   Instruction variants/fields/tags, booleans/escaped selectors and BTreeMap order.
   No integer-literal instruction exists: numeric checks concern formatVersion
   u32 or wrong-typed fields, not runtime i64 validation. Keep ONE authoritative
   decoder. Duplicate map keys retain LAST value, but an ill-typed overwritten
   earlier value still fails; duplicate struct fields reject. Preserve this exact
   behavior, never invent stricter rule-map rejection or erase original text.
2. Eager typed rejection before invalid policy, including ignored/unused rules,
   unknown op/extra/duplicate struct fields, wrong scalar types/ranges, malformed
   documents and version precedence. Assert exact Json/Version/Lowering phases
   and original root Error/source adaptation, not just shared-engine agreement.
3. COMPLETE handwritten tiny source for default/reference and a distinct custom
   assembly/traversal policy, escaping/canonical order and fresh recovery. Expected
   source/metadata comes from contracts, never emitted/native/Go/golden capture.
   Retain existing full driver/evaluator/left-twice/reverse/adversarial goldens and
   actual compiled execution. No custom policy may bypass typed rule validation.
4. Real Cargo CLI default/custom input, usage/read/JSON/version/policy failures,
   exit2/EMPTY stdout, clear environment and fresh recovery. Mandatory tools must
   be absolute regular/readable/executable BEFORE env-clear, owned HOME/unusable
   PATH. CLI reads an explicit policy file BEFORE calling generate_with_policy;
   a missing policy read beats malformed rules, unlike API validation precedence.
   Setup failures are not semantic negatives; no substitute provider API.

Run new FOUR groups EACH default/no-default debug/release with strict focused
Clippy. Keep all existing assertions/expected outcomes unchanged. Lowering policy
512-node/32-level, dispatch4096/depth64/frames128/output1MiB, module64/16/4096/1MiB
and original typed serde recursion/duplicate behavior remain unchanged. This is
not comprehensive instruction-input hardening or general Rust name resolution.

## Ownership, source proof and release gates

Two independent focused source/API/error and semantics/evidence contract reviews
plus explicit coordinator GO BEFORE implementation. At most TWO isolated stable-
main workers, agreed owned paths, immutable handoffs and NEW-child fixes/overlays;
integrate ONLY owned paths. No amendment/rebase onto mutable coordinator. Format
only owned files. Retire workers/panes/workspaces BEFORE PR.

Coordinator independently projects ALL original lib.rs/codegen.rs source into
moved core OR retained runtime/adapters, accounting once for every original token,
with ONLY the transformations above. Demonstrate the core compiles with real
lowering/module but NO root Error/runtime stubs. New errors are actual compiler
results, not replacement stand-ins for runtime dependencies. Original CLI and all
five native façade source cuts stay exact. No construction expectations/seeds from
committed generated Rust, goldens, observed/native/Go answers. JSON stays build-only.

Target coherent <=~1500 total changed lines INCLUDING both sides of source moves,
all evidence/docs/generated code. Initial estimate ~1100–1350, not a proof cap.
Pause for ambiguity/unresolved validation/excess; never compress/delete proof or
weaken assertions. Independent cross-reviews and TWO final immutable exact-tree
reviews after integration, pinned Shaka/full preflight/whole-tree whitespace/links.

Actually capture ONCE/build AND rebuild SAME explicit roots: currently27 checks/
24 NONDEFAULT packages,51 names/50 unique; confirm actual counts. All24 prior
NONDEFAULT package constructor IDs INCLUDING exports must remain unchanged; full
executor/legacy seed/verifier IDs may change. All249 JSON/12 builders/old native
DATA/generated Rust/five façade source cuts/six names-origins remain unchanged.
Full executor/default debug+release strict all-target Clippy, exporter142/research135
FOUR profiles, all THREE four-group compiler-boundary suites EACH FOUR profiles.
All FOUR genuine façade matrices and separately rebuilt exact feature roots matched
to fresh discovery.21 fresh recursive/full --requisites --include-outputs retained
inventories/current Go CLI+source outputs/drvs/full bundle/executor/legacy joined/
BOTH oracle package outputs+drvs/provider-env exclusions for native construction.
Core/unifier/Pratt feature generation Go-free; scanner source-feature positively
Go-backed VERIFICATION ONLY. Real sandboxes, no pending/canceled/setup run as pass.

Protected exact-head CI-gated rebase auto-merge; watch required cuestacean validate
through confirmed MERGED, compare exact merged tree, retire workers and return to
clean fetched main. Linuxx86_64 only; no sixth constructor, minimality, performance,
full CUE/self-hosting or new default compatibility claim.

**Implementation GO:** BOTH independent source/API/error/source-cut and semantics/
evidence reviews approve immutable clarified contract b71ae10c, including genuine
semantics-only root adapter/import gating, PRIVATE format_version and ONLY two
crate-private fields, ALL45 variants/actual numeric domain, preserved overwritten
map-value typing and API-versus-CLI precedence. Explicit coordinator GO authorizes
ONLY the three kernel Rust paths and one new evidence test path above. No Nix/
Cargo/CLI/CUE/runtime/kernel algorithm change follows. Source projection, actual
four-profile evidence, all release gates and two final whole-tree reviews remain
mandatory. Pause rather than widen scope, suppress failures or exceed the budget.

## Local validation receipt

Kernel038b6900 (ONLY three Rust paths) and evidence451ac084 (ONLY one test path)
stayed immutable; dependency overlays used new children, not handoff amendments.
Independent cross-reviews approve both. No CLI/Cargo/Nix/CUE/runtime/kernel change.
The four focused groups pass EACH default/no-default debug/release with strict
focused Clippy: ALL45 variants/projections/real backend, preserved duplicate typing,
API/CLI phase precedence, concrete serde/lowering source chains, complete handwritten
default/reference/custom source and actual cleared-env CLI exit2/EMPTY/recovery.

Independent ORIGINAL-source projection BYTE-MATCHES all three final files:
old lib657/codegen46 rows accounted; retained root466/core263/façade21. Rules/Rule/
Instruction/parse and compiler bodies move once; root Error/Program/runtime suffix
stay exact. Only reviewed imports/modules/narrow visibility, real Error3/owned
bijection, runtime-only cfg/forwarding and duplicated API docs differ. No algorithm
or admission/guard/profile/provider change. Same-source tests use real modules,
not runtime/Error stubs; full-library verification remains Go-backed.

Full executor/default debug+release tests/strict all-target Clippy, exporter142 and
research135 FOUR profiles, and ALL THREE four-group boundary suites EACH FOUR
profiles pass. Pinned preflight13/13 (seven skips), whole-tree whitespace1408 files
and629 relative links pass.27 checks+24 NONDEFAULT packages give51 named attrs/
50 unique derivations captured ONCE, actually built AND rebuilt with SAME explicit
drv roots in real sandboxes. All FOUR separately rebuilt feature roots MATCH fresh
final discovery;21 refreshed recursive/full retained inventories/current exclusions
pass. Core/unifier/Pratt feature generation stays Go-free; scanner source-feature
is positively Go-backed VERIFICATION ONLY.

ALL24 prior NONDEFAULT package IDs INCLUDING exports remain unchanged, together
with all249 JSON/12 builders/old DATA/generated Rust/five façade source cuts and
six compiler names/origins. FIVE complete native policies/FIVE native binaries
remain; no sixth constructor, general default/schema or self-hosting claim follows.
