# Pratt compiler/runtime source boundary (#244)

## Actual next dependency after #242

Pratt policy DATA is native; bootstrap-pratt-codegen still builds via the legacy
full library. pratt.rs mixes typed IR/validation/metadata/lowering with parser Frame
and four runtime item frames. Select a bounded ARCHITECTURE slice, not the fifth
compiler constructor. Five complete native DATA inputs/four isolated compiler
binaries remain; instruction policy/two binaries/joined seed/PROGRAM/schema still
need Go. #175 stays OPEN, held prototype unchanged.

One real shared dependency needs explicit treatment: Condition::{Item,Slice}KindIs
uses unification::Kind. Move that EXACT11-variant serde enum to ONE authoritative
unification/kind.rs and re-export it from the current unification compiler API.
Do NOT duplicate it or require future Pratt construction to embed all unification
program/compiler/policy machinery just for a data type. The existing native unifier
source allowlist must consequently grow7→8 (test8→9), otherwise its real sandbox
constructor breaks. This mechanical source-boundary change is part of this slice,
not a new policy, public enum, interpreter, provider or constructor.

## Exact interfaces / ownership

Kernel/build owner ONLY:
- executor/src/pratt.rs and NEW executor/src/pratt/compiler.rs;
- executor/src/unification/compiler.rs and NEW unification/kind.rs;
- infra/tooling/flake.nix, ONLY nativeUnifierSources added kind.rs and corresponding
  nativeUnifierVerification source-count assertions8/9. No other Nix algorithm/
  feature/seed/provenance/public package/check edits.

New Pratt compiler.rs owns ORIGINAL serde/Snafu Error/typed Program/Target/Number/
Condition/Statement, limits/parse/family validation/definite_return, generate,
generate_with_policy/project_body/lower. No parser Frame/root Error/runtime module,
frame/Value/stub dependency. Retain genuine crate::unification::Kind. Move text,
algorithms, serde fields/tags/defaults/errors/validation ordering exactly; only
imports/module declarations/explicit re-exports/narrow visibility/helper test paths.

Parent retains ORIGINAL Number::value/Condition::test, Flow/extended_action/execute/
interpret and four collection interpreter APIs/ItemFrame/macro/primitives/VM. Keep
ALL embedded target/return tests, assertions unchanged except moved-helper paths.
One explicitly reviewed typed-integration exception: annotate existing interpret_list
binding `let program: Program = parse(input)...` with its SAME original inferred
Program type. This gives the preserved unconditional pub(crate) alias a genuine
no-source consumer, without cfg/lint suppression, wrapper or algorithm/API change.
Re-export public Error/generate/generate_with_policy; preserve pub(crate) Program/
parse paths used by parser, not new public IR. Private Target/Number/Condition/
Statement and Program.target/body ONLY pub(super) for parent execution. Preserve
private Program.format_version. Preserve public runtime APIs/error source chains.
Helper definite_return/project_body/lower may be pub(super) only where parent tests/
same-source test require. No broad public/pub(crate) IR convenience or copied core.

Shared kind.rs contains exact current Kind derives/camelCase/eleven variants,
imports/notice only. Current unification/compiler.rs loads this real source with
normal explicit module path and pub use, keeping unification::Kind and façade
unification::Kind identical API paths. No new enum/adapter/stub/renaming/serde work.
No unification parent/runtime.rs/façade Cargo/lib/CLI/CUE/guards changes. Preserve
Apache/applicable notices. Existing source/compiled-lowering features remain real
and unchanged; no cfg/lint/stack/recursion suppression or fallback.

## Independent boundary evidence

Evidence owner ONLY NEW executor/tests/pratt_compiler_boundary.rs. Compile SAME
Pratt compiler.rs in a test root exposing ONLY real lowering, the SAME actual Kind
source at unification path (#[path] module, not a duplicate enum/mock), and actual
compiled_pratt_lowering under genuine compiled-lowering feature. NO root parser,
runtime, Error, Value/frame stubs or source-provider ABI. Full-library test graph
still Go-backed VERIFICATION, NOT independently native construction proof.

Four focused groups, approximately120–180 lines if feasible without compressing
assertions: complete independently handwritten metadata for default parser and
list/record/conjunction/slice targets; omitted else/default target, canonical action/
selectorAction and true early/tail return positions through nested if/while; real
lowering Driver/Context checks, not captured output. Exercise actual Kind wire tags
for collection/slice metadata from authoritative source.

Typed rejection BEFORE invalid policy: cross-family dead branches/nested conditions,
statements after definite return even in loops/dead arms, original-wire duplicates,
internal action/selectorAction/parseMinimum/position/tail metadata not accepted on
wire, malformed target/u8/i64. Real default/reference complete tiny Rust and custom
policy observing canonical target/tail/body with failure→success recovery. Existing
return-step budget/laziness/parser/runtime/source assertions remain unchanged.
Same four groups execute default/no-default debug/release with strict focused Clippy.
Do not claim original control structural guard counts overwritten duplicate values
or streams policy input; preserve depth0/32 and512 original guard exactly.

## Integration and release evidence

Coordinator owns docs/integration/strict source projection/artifact/graph/full local
validation/CI. Two independent focused source/API/Nix/Kind and semantics/evidence
reviews plus explicit GO BEFORE implementation. Stable-main isolated workspaces,
at most two workers; immutable handoffs/new-child fixes/overlays, integrate ONLY
owned paths. No amendment/rebase onto mutable coordinator or concurrent formatting
of others' files. Retire created workers/workspaces BEFORE PR.

Strict original-source projection accounts for EVERY original Pratt token in new
compiler OR retained runtime/tests (only shared module doc may duplicate). Separately
account for unchanged authoritative unifier compiler and moved Kind enum. Both
source files/Kind byte-exact after pinned rustfmt plus documented visibility/import/
module/test helper changes and that ONE inferred-to-explicit Program annotation;
no rewriting source algorithms or Rust DATA strings.

Preserve all249 JSON/12builders/old native DATA/generated Rust. ALL17 other prior
PUBLIC package constructor IDs INCLUDING exports stay unchanged; native-unify-
codegen legitimately changes due source8/test9 and exact original Kind dependency.
Native unifier policy directory still ONLY TWO unchanged files; native unifier
compiler is rebuilt and used in actual six-binary pipeline core2/scan1/unify1/legacy2.
Old core8-9/scanner7-8/exporter9/raw cuts unchanged. No new package/check/count claim.

All full executor/default all-target strict Clippy debug/release, exporter109/four,
research135/four, old unification boundary4/four plus NEW Pratt boundary groups/four;
ALL THREE genuine façades/fresh feature modules and separate explicit drv^* rebuilds.
Actually build AND rebuild ALL21 checks/18 packages. Fresh FIFTEEN recursive/full
--requisites --include-outputs retained inventories; current Go CLI/source/source
drv/full bundle/exec/legacyjoined/BOTHoracle outputs-drvs/provider env exclusions
for native construction. New unifier8/9 source/Kind/policy TWO inventories/feature
Go-free proof; core feature native/scanner source-feature positive Go VERIFICATION
ONLY. Source separation test itself remains full-crate Go-backed verification.

Pinned Nix/Shaka/full whitespace/relative links; immutable cross/final exact-tree
reviews, protected exact-head rebase auto-merge/required CI watched through confirmed
MERGED. Linuxx86_64 only; no minimality/performance/full CUE/self-hosting/default
admission claim. Target coherent <=1500 changed lines counting BOTH sides of moves,
new evidence and docs. Pause/reassess for widened API/shared implementation/CUE/ABI/
guards, unresolved validation or larger scope; never delete/compress evidence.

Implementation GO: BOTH independent focused reviews approve the exact source/API/
Kind/Nix and independent metadata/validation/backend evidence interfaces above.
Coordinator GO permits ONLY those owned paths and unchanged behavior. Use explicit
source-relative Kind module resolution, same-source authoritative Kind rather than
whole unifier machinery, Json-versus-Program negative assertions/original text for
duplicates and numeric errors. Evidence line estimate is not a cap. Strict source
projection, actual current native-unifier rebuilds, all package/artifact/graph/full
validation and immutable cross/final release reviews remain mandatory. No fifth
constructor or Go-free boundary-test proof follows; pause for widened scope.

## Completed local evidence

Kernel53a69ce5 and tests7a33803a stayed immutable; fix165a5223 is a new child,
integrated ONLY by owned paths. Four original-source projections BYTE-MATCH after
pinned rustfmt and documented transforms: ALL1177 Pratt rows once except shared
module docs twice; ALL649 unifier rows once, exact Kind27–41 moved once. Only one
same-inferred-type Program annotation joins import/module/re-export/visibility/
helper-path/blank changes; algorithms/Rust DATA/retained runtime/assertions unchanged.
Nix changes ONLY the exact Kind source entry and source-count assertions8/9.

Initial no-source strict Clippy exposed an unused Program alias. Both focused
reviews approved that single genuine typed consumer, preserving unconditional
crate-private paths without cfg/lint suppression/wrapper. Initial failed/canceled
runs are NOT passing evidence. All four new groups now pass EACH of four profiles
with strict focused Clippy; complete five-target metadata/all11Kind tags ×two
condition families/original-wire errors/default-reference Rust/custom facts pass.

Full executor/default all-target strict Clippy debug/release/exporter109 four/
research135 four/old+new boundary4groups EACH four pass. ALL21 checks/18 packages
actually build AND rebuild; all THREE genuine façade matrices/fresh modules plus
separate explicit drv^* feature rebuilds pass. Pinned preflight13/13 (7 skips;22
checks discovered incl default alias),1377-file whitespace/610 relative links pass.
All249 JSON/12builders/old DATA/generated Rust unchanged; ALL17 OTHER prior public
PACKAGE drv IDs INCLUDING exports unchanged. Actual native unifier changes only
for source8/test9; exact original Kind/source bytes, unchanged two native policies/
lock/Cargo/CLI and six compiler origins core2/scanner1/unifier1/legacy2 verified.

Constructor caxqjc3w uses source8 j2qzbp7r; feature j274jjv7 uses source9 fa61cpbd.
Fifteen fresh recursive/full inventories MATCH retained; native unifier1423drvs/
3434full paths, joined1456/3521. Current Go CLI/source/source drv/full bundle/exec/
legacyjoined/BOTHoracle outputs-drvs/provider env exclusions pass; unifier also
excludes sibling compilers. Core/unifier feature generation independently native,
unifier feature retained inventory MATCHES; scanner source-feature positively
Go-backed VERIFICATION ONLY. Five native policy DATA/four binaries remain, no new
constructor or native schema/default admission. Full-library boundary tests remain
Go-backed verification. Only Linuxx86_64 validated; final exact-tree/CI/merge gates
above remain mandatory.

Subsequent [#246 native Pratt construction](native-pratt-codegen.md) uses this exact
core/Kind in an independent eight-source/two-policy façade; that separate milestone
integrates the existing fifth binary, not a capability claimed by #244 itself.
