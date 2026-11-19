# Native construction of the complete default AST policy data

## Selection and authority

Milestone [#222](https://github.com/kolohelios/cuestacean/issues/222).
After #220/#221, remove Go CUE from construction of the canonical
`ast-lowering.json` input, the first COMPLETE lowering-policy data artifact among
the six compiler policies. The user approved #175 becoming a long-term compatibility
umbrella rather than the critical delivery gate. Its unresolved obligations remain
open and its prototype held; this construction milestone does not satisfy them.

No native exporter/executor grammar, semantic class, error, compiler wire format,
resource cap or Rust implementation changes. This is NOT native evaluation of the
full #LoweringPolicy schema or a mixed marked/let CUE document. Two focused
source/composition and independence/evidence reviews, agreed ownership and explicit
GO must precede implementation. Stop if this requires broader default semantics.

## Canonical source modules and a separate Go validation bridge

Relocate the remaining Copy/Owned lets and cases from
`bootstrap/ast-lowering-fragments.cue` to new ordinary root-data
`bootstrap/ast-lowering-body.cue`. Keep every represented instruction and lexical
binding unchanged. Add explicit `formatVersion: 1`, formerly supplied by the Go
schema's singleton. The body provides six cases: program, parameter, argument,
string, object, field. It does NOT provide integer or an incomplete integer slot.
The whole body plus package-only `native-policy-package.cue` is already in the
ordinary native subset, including the reviewed immutable lexical-let semantics.
No marked syntax, unresolved names, unsupported declarations or latent defaults
are hidden by extraction. Direct native CLI inputs, not grep/preprocessing.

Keep the existing `ast-integer-policy.cue` BOTH-alternative raw owner and native
integer artifact UNCHANGED. That module alone uses the reviewed one-group tagged
profile. Each module is fully materialized independently before composition.

`ast-lowering-fragments.cue` becomes a Go-only bridge: #AstLowering remains
#LoweringPolicy with required slots for ALL seven cases. It no longer duplicates
the represented instructions. Its explicit schema validation and slot requirements
belong in the raw Go oracle/override path, not the native construction closure.
Missing body or integer provider must fail there as Incomplete/no JSON. This is
an honest source/validation boundary, not claiming native support for that schema
or silently dropping unsupported definitions from a still-loaded document.

Raw Go default/override comparisons load schema, bridge, body, raw integer owner
and selector/context. They retain all constraints before preference selection.
Canonical production default instead consumes the independently constructed policy
DATA; override fixtures continue on the explicitly selected raw-source path.
Selected JSON is not a residual preference or late-refinement interface.

## Disjoint materialized-module composition

Add a small generic build-only `bootstrap/compose-policy.jq` used by production
and tests. It accepts exactly TWO JSON object modules. Recurse only when a shared
key has object values on BOTH sides; otherwise shared keys are ERROR, even when
values are identical. Arrays/scalars are atomic leaves, never concatenate, prefer,
meet or overwrite them. Nonobject roots, missing/extra modules and shape collisions
fail. Output uses existing pinned `jq -S` canonicalization.

For current modules, the only shared object namespace is root `cases`; every leaf
path is disjoint. Body has formatVersion and six complete cases; integer has its
one complete case. Composition therefore merely copies independently concrete
fields. No unresolved constraint, default, binding or closure crosses the module
boundary; no post-materialization CUE refinement is admitted. This is NOT residual
JSON merging, CUE conjunction/solving, or a contextual-equivalence shortcut.
Both argument orders must give identical bytes. Do not inject expected answers.

Native CLI calls retain their existing PER-INVOCATION limits (as in the 12-builder
family). Repeat each body export with unchanged shared work per call; no resetting
inside a source/branch/path. Do not claim one shared4096-step allowance across two
independently constructed modules and a separate assembly step. Each native input
and output remains bounded; composition has exactly two bounded inputs, native
container-depth bounds and an explicit combined64KiB encoded-output check before
publication. It does not raise the compiler's existing typed-policy/node/VM caps.
Pinned jq is used only for structural identity/assignment/sorting, no numeric
arithmetic; current policy numbers are formatVersion1. Preserve exact integer
controls and do not claim arbitrary numeric-transform compatibility.

## Producer, consumer and scope of validation

Add `native-ast-lowering-policy`: receives the unchanged minimal native exporter,
raw body/package files, unchanged native integer artifact, pinned jq and the
composition script. Export body twice under cleared environment/unusable PATH/
owned HOME, compare bytes; compose twice with swapped module order, compare bytes;
check output bounds; publish build-only `ast-lowering.json`. No Go CUE/schema,
full JSON bundle, executor/compiler, oracle, expected/captured data or generated
Rust is a constructor input. Source-derived native intermediate JSON is allowed;
committed/captured answers are not. Failures never retry through another provider.

`jsonExports` copies this artifact and SKIPS Go CUE for canonical `ast-lowering`,
using exact fixture membership like the existing native builder family. All other
fixtures retain their own paths; AST override fixtures explicitly load the body
and RAW integer source before context. This is membership, not fallback.

The existing Rust lowering-policy decoder/compiler remains the typed consumer,
with all existing validation and limits before emitting code. Go #LoweringPolicy
validation is retained independently in the verifier. The native artifact claims
complete canonical DATA construction, not evaluation of that schema language or
validation of arbitrary user-authored policies. Invalid-source controls must fail
independent schema/typed consumer checks, never publish successful compiled code.

Add one `native-ast-lowering-policy` check and package: retain ELEVEN old checks/
EIGHT old packages, now TWELVE/NINE. Its native graph roots cover exporter,
unchanged integer artifact and complete native policy DERIVATIONS. Inspect actual
recursive derivation/full graphs and retain roots/registration/paths. Exclude Go
CUE output/derivation/source, current full JSON and executor outputs/derivations,
BOTH current oracle outputs/derivations and provider environments. The separate
verifier may depend on Go CUE, bundle and executor, never feed construction.
The exporter cut remains eight files, integer raw cut two, experimental-policy
raw cut six. New construction owns the package/body two-file raw cut plus the
composition script; union of CUE owners is THREE files, not a single supported
mixed marked/let native invocation.

Six complete compiler seeds still consume other Go-built policies/the full bundle;
no Go-free AST-seed/minimal-seed-source, all-six closure, self-hosting or performance
claim. This removes one complete policy-data dependency on Go, not all boundaries.

## Evidence and ownership before implementation

Build owner ONLY: `infra/tooling/flake.nix`, `bootstrap/ast-lowering-fragments.cue`,
new `bootstrap/ast-lowering-body.cue`, new `bootstrap/compose-policy.jq`.
Evidence owner ONLY: `bootstrap/spec-export/tests/integer_policy.rs` (add mandatory
body to Go witnesses, preserve old assertions), new
`bootstrap/spec-export/tests/ast_policy.rs`. Coordinator owns docs, baseline/full
graph inspection and integration/CI. No exporter/executor src or dependency edits.
Stable merged #221 workspaces, immutable handoffs, no saved amendment/rebase onto
mutable coordinator; at most two workers, format owned files only.

Required evidence:

- Complete hand-authored expected body and full policy trees, not native/oracle
  captures. Native library/CLI, raw Go and structural composition agree; repeated,
  renamed/reversed inputs and fresh cleared mandatory absolute providers.
- Whole raw-Go policy, native policy and production artifact bytes agree with
  merged #221; all249 bundle outputs/all12 builders/generated Rust unchanged.
  Keep constant99/forced text/tag-only outcomes and all64 exporter assertions.
- Missing body/integer slots fail without JSON. Late concrete-default override
  fails; raw override remains live. Mixed marked/let source remains excluded.
- Production composition script rejects duplicate leaves (including equal values),
  array/scalar/shape conflicts, nonobject roots and wrong module counts. Swapping
  disjoint input order preserves bytes; exact integer identity controls remain.
- Source/effect controls independently change body source and integer source,
  preserve their intended lexical/default semantics, construct native modules and
  the complete policy, then drive the actual AST compiler to independently
  expected emitted Rust. No claim of unperformed generated-program execution.
- Go schema/typed compiler validation remains a separate mandatory check; malformed
  opcode/shape controls fail there with no emitted Rust. No schema-free arbitrary
  policy-acceptance claim, and no invalid-input Go fallback.
- Both focused contract approvals + explicit GO before edits; immutable final
  implementation/evidence cross-reviews; full Rust suites/default strict all-target
  Clippy debug/release, all retained135 research/four profiles, exporter/four profiles,
  pinned preflight, TWELVE actual checks/NINE packages build AND rebuild, fresh six
  artifact-free compilers/isolation/drift/source/value and full graph evidence.
  Exact-head protected rebase auto-merge and required CI through confirmed MERGED;
  retire workers/workspaces before PR. Target coherent <=1500 changed lines.

Read-only pinned probes on #221 inputs already export the ordinary native body
within unchanged limits and compose it with the existing native integer artifact
to byte-identical complete `ast-lowering.json`; a duplicate module is rejected.
These observations justify assessment, not construction attestation. Two focused
source/composition and independence/evidence reviews subsequently approved this
exact contract and agreed ownership. They explicitly require exact-two-module
validation (not present in the early scratch helper), output-size checks, all
seven Go bridge slots and mandatory body inputs in every raw Go witness.

Explicit coordinator implementation GO is now recorded for this contract only.
No new semantic class or schema-evaluation claim is authorized; immutable final
cross-reviews and all validation/CI gates remain required.

## Implemented cut and independent local evidence

Build handoff `d36cf4531f4a038bfefeeabb47168b80a504c200` and independent tests
`64c0ff4f9e85b309ada9c8cc22f119831c82677f` were integrated by owned path. A
read-only review found diagnostic unwraps in the new test harness; immutable child
`4b7647c975ae69e49f0569421d799d60f0f3ddae` uses meaningful expectations and
checks provider status before consuming stdout. Unused jq-provider environment
variables were removed during integration; pinned jq remains a mandatory absolute,
readable/executable process discovered from the test toolchain PATH. No semantic
assertion, exporter/executor implementation, old expected result or hard cap changes.

Six new groups independently author the COMPLETE body/policy trees, retain every
old assertion, compare native library/CLI, raw Go and the PRODUCTION composition
script, reject collisions/count/shape mistakes, preserve exact i64 identity and
prove that mixed marked/let native input is STILL Unsupported. The separate
sandbox checks required module presence, full schema validation, actual builder
code generation/goldens, BOTH modules' source-to-independently-expected Rust effects,
and malformed opcode/shape failures in Go schema and typed native compiler with
no emitted Rust. These are emitted-source controls, not unperformed runtime claims.

Local x86_64-linux validation passes all70 exporter groups/four profiles, both full
Rust suites/default strict all-target Clippy debug/release and retained135 research
groups/four profiles with focused strict Clippy. Pinned preflight passes13/13
(seven skips); TWELVE actual checks/NINE packages build AND rebuild, retaining six
fresh artifact-free compiler binaries/isolation/drift/source/value evidence. Other
architectures are omitted, not passed.

The body source cut is exactly package+body at
`/nix/store/4cqb994izyjijbxkml3vm9x58rqx1ysf-source`; composition source is
`bootstrap/compose-policy.jq`, separately attested. Original eight-file exporter,
two-file integer and six-file experimental-policy cuts/constructor identities
remain unchanged, as do both oracle constructors. Direct source comparison proves
unchanged Copy/Owned lets and represented instructions, apart from the documented
wrapper/version/required-slot boundary. New full native-policy constructor is
`ly6bhplk…`; full bundle becomes `hl49dys2…` while ALL249 JSON outputs, all12
builder artifacts and committed generated Rust remain byte-identical to #221.

Independent recursive queries match retained full inventories: four existing
native cuts1407 derivations/3392 paths each; complete native AST-policy cut
1412/3410; shared seed1438/3473. Both current oracle outputs/derivations, current
executor and provider environments are absent from construction; native graphs
also exclude Go CUE/source/current full JSON. The new complete policy artifact
really has no Go construction dependency. Schema validation and five other policy
constructors still use Go; the shared six-compiler seed is NOT Go-free or minimally
source-attested. #175 remains an open compatibility umbrella, not a delivery gate.
Final immutable cross-review and exact-head protected CI/confirmed merge are still
required release gates.
