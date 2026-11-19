# Native AST integer lowering-policy input

## Selection and review gate

Milestone [#220](https://github.com/kolohelios/cuestacean/issues/220).
Continue the separately approved [tagged fragment](tagged-default-admission.md)
after #218/#219: make that SAME fragment a production input of the AST compiler's
default policy. This is source/construction integration, not a new default class,
full AST policy/schema admission, or a narrowed definition of #175. Original #175
stays open/held. No exporter grammar, semantics, guards or error behavior change.

This contract requires two focused source/consumer and construction/evidence
reviews, agreed distinct ownership and explicitly recorded implementation GO
before implementation edits. Those gates were completed below. If any future
relocation/composition requires broader default-context semantics or erasing
unselected alternatives, stop.

## One canonical raw owner, not a source-rewriting selector

Move the unchanged integer alternative from `ast-lowering-fragments.cue` into new
`bootstrap/ast-integer-policy.cue`, in ordinary root data:

```cue
package bootstrap

{cases: {
    integer: *{op: "concat", parts: [{op: "text", value: "Value::from("}, {op: "signed", field: "value"}, {op: "text", value: "i64)"}]} | {op: "text", value: _}
}}
```

The literal expression and BOTH alternatives are unchanged. Add a package-only
`bootstrap/native-policy-package.cue`. These are the artifact's entire two-file
raw cut. Native construction invokes the existing two-path CLI on these files
DIRECTLY: no grep/cut, extracted definition selector, preprocessing that drops
unsupported declarations, expected/captured JSON, or generated Rust as inputs.
The standalone input satisfies the exact existing one-group guard at field path
`cases.integer`; no definitions, lets, aliases or other choices are admitted.

Leave `integer: _` as a REQUIRED provider slot in `#AstLowering.cases`. It is not
another default implementation. Removing the provider must fail export as
Incomplete rather than silently publishing a policy without its integer case.
No duplicated raw alternative remains in that definition or in expected files.
Other compiler cases and lexical Copy/Owned declarations remain unchanged.

## Explicit production consumer boundary

Add independent package `native-ast-integer-policy`, producing exactly
`ast-integer-policy.json` from the minimal production exporter plus the two raw
sources, twice with byte comparison. Fresh cleared environment, unusable
PATH and owned scratch HOME; mandatory absolute readable/executable exporter.
Errors fail the build, never trigger Go export. JSON remains build-only.

For the canonical `ast-lowering` fixture ONLY, `jsonExports` adds that artifact
as an explicit JSON root input of the EXISTING Go CUE schema/fragments/selector
invocation. Go CUE still constructs/validates the rest of the compiler policy;
there is no native full-policy or Go-free compiler-seed claim. This is ordinary
CUE root conjunction, not a hardcoded Rust lowering case or JSON subtree patch.
The policy compiler consumes the resulting fully materialized policy exactly as
before. The selected default integer constructor is valid under the existing
`#LoweringExpression` schema; output/case membership and generated Rust must stay
byte-identical. Do not assert general contextual congruence of projected JSON.

The artifact is already a materialized DEFAULT policy input. It is NOT an
unresolved preference that callers may later force to text. Fixtures such as
`tests/valid/ast-lowering-forced` therefore load the canonical RAW owner (both
alternatives) as an explicitly selected input BEFORE their contextual override.
They MUST NOT load the concrete default artifact. This is manifest membership
chosen before execution, not error-triggered fallback. Preserve the compiled
constant99 control and every original assertion. Native forced-text and tag-only
Incomplete remain separately tested against the raw two-file native input.

A raw-source Go oracle for the default policy loads schema, unchanged remaining
fragments, canonical raw owner and root selector; production loads the same inputs
except the independently native JSON replaces the raw owner. Compare complete
canonical policy bytes, not only the integer subtree. Also compare the full 249
outputs and all generated Rust against merged #219. No extra exported fixture or
committed JSON artifact is required.

## Independence and acceptance evidence

Create one separate `native-ast-integer-policy` check; retain all ten old checks
and seven old packages (now ELEVEN checks/EIGHT packages). Its native construction
closure roots are the exporter and artifact DERIVATIONS. Reject current pinned
Go CUE/source, current JSON bundle, downstream executor and BOTH current oracle
outputs/derivations, plus oracle provider environments, in that recursive native
construction graph. Retain complete root/registration/path inventories and inspect
`nix-store --query --requisites --include-outputs`, not just runtime references.
The verifier may depend on Go CUE/executor; it must never feed construction.
The exporter source cut stays exactly eight files; existing raw policy cut six.

Before publication require:

- Independent hand-authored whole fragment JSON for default concat, forced text/x
  and tag-only Incomplete/no JSON. Read the canonical source directly; never
  reconstruct the artifact from expected/captured data.
- Source relocation default and forced full-policy exports must match #219.
  Raw full-policy text-only must remain Incomplete (eval succeeds, export fails).
  Missing provider must fail with no JSON. Default concrete artifact combined
  with forced-text context must fail, documenting why override fixtures use raw
  input instead. Setup failures are not semantic observations.
- Mandatory absolute readable/executable native/oracle processes; fresh cleared
  environments, unusable PATH, owned HOME, repeated/reversed/renamed raw inputs.
  Structural/limit behavior and all 59 exporter groups stay intact; no modified
  assertions, stack overrides, raised guards or suppressions.
- The separate verifier compares native fragment bytes with independently
  authored complete answers and raw Go CUE; compares assembled full policy with
  the raw-source oracle and production `ast-lowering.json`; feeds it into actual
  AST code generation and compares unchanged independent builder Rust goldens.
- A separately authored preferred-source variant retaining BOTH alternatives
  must change the native artifact, assembled compiler policy and emitted builder
  Rust in the predicted way. This is source/effect evidence, not permission for
  field/opcode-specific native semantics or a full-policy bootstrap claim.
- Native forced-text production-binary probe supplies its value, while tag-only
  remains Incomplete. Do not count a default concrete JSON artifact as evidence
  of contextual override support after materialization.
- No change to all twelve builder outputs, all 249 bundle JSON files, committed
  generated Rust, independent old answers or current oracle identities. Existing
  exporter/atom/family/experimental-policy constructors stay unchanged because
  their production input cut is unchanged; full JSON/consumer identities may change.

Two immutable implementation/evidence cross-reviews, both full Rust suites/strict
all-target Clippy debug/release, retained135 research groups/four profiles,
pinned preflight, ELEVEN actual checks/EIGHT packages build AND rebuild, six fresh
artifact-free compilers, feature isolation, deterministic generation/drift,
source/value evidence and fresh full graph inspection remain mandatory. No
Go-free/minimal-source seed, full self-hosting or performance claim. Linux x86_64
only. Retire workers/workspaces before PR; exact-head protected rebase auto-merge
and required CI watched through confirmed MERGED.

## Agreed ownership and preliminary probes

Build owner: ONLY `infra/tooling/flake.nix`, `bootstrap/ast-lowering-fragments.cue`,
and new `bootstrap/{ast-integer-policy,native-policy-package}.cue`. Evidence owner:
ONLY `bootstrap/spec-export/tests/tagged_defaults.rs` (raw owner relocation, no
weakened outcome), and new `bootstrap/spec-export/tests/integer_policy.rs`.
Coordinator owns docs, integration, baseline/artifact/graph proofs and final CI.
No exporter or executor implementation changes, new modules or compiler binaries.
Stable merged #219 workspaces; immutable saved handoffs, no amendment/rebase onto
mutable coordinator parents; at most two workers, owned-file formatting only.

Read-only scratch probes on pinned #219 inputs checked the proposed relocation:
raw default and forced full policies are byte-identical before/after relocation;
production native-case composition is identical under the existing pinned JSON
canonicalization (the native JSON changes only object ordering before that step).
Missing provider fails; raw text-only export fails while eval succeeds. These
support a review; they are NOT implementation or source-closure attestation and
not an approval to broaden semantics. These probe outcomes were then checked by two independent focused contract
reviews. Both approved the exact same-profile source/consumer boundary, direct
raw construction, explicit default-versus-override manifest membership, ownership
and evidence obligations. The marked constructor's text/signed/text parts satisfy
the current #LoweringExpression schema without a further choice/refinement;
Go retains schema/definition authority. This is the exact consumer justification,
not a general contextual-congruence claim.

Explicit coordinator implementation GO is now recorded for this contract and
agreed ownership only. The build and evidence owners may implement after these
two approvals; immutable cross-reviews and all production-validation/CI gates are
still required. Original #175 remains unchanged/open/held.

## Implemented construction and local validation

Immutable build handoff `c8d4d1ce7d70e95271cc5d9b23ad93f34cb01edf` and evidence
`ae23d273d50590caae8c6dd0675f3937af0710fc` were integrated by agreed owned paths.
Focused review caught two new native-context witnesses using Go shorthand; the
immutable child `2bda0f7c33412efc0c87b30b450f0c4fd179942a` uses explicit nested
braces instead. No assertion, outcome, grammar or guard changed. Five new groups
compare whole fragment expectations, complete raw/native policy composition,
raw text/x/99 controls, missing-provider/raw tag-only Incomplete, and conflict
when a concrete default is incorrectly given a late text override.

Local Linux x86_64 validation passes all 64 exporter groups in four profiles,
both complete Rust suites/default strict all-target Clippy debug/release, all
retained135 research groups/four profiles and focused strict Clippy. Pinned
preflight passes13/13 (seven skips); ELEVEN actual checks/EIGHT packages build
AND rebuild, including source/effect and downstream compiler evidence. Six fresh
artifact-free compilers, feature isolation, generation/drift and source/value
checks remain. Other architectures are omitted, not passed.

The production exporter source cut is the unchanged eight files; the canonical
artifact raw cut is exactly two files at
`/nix/store/rfb7zcl7xjds771vy39npq6rybz2nvy7-source`; the existing experimental
policy cut remains six. Direct comparison confirms the exact original
BOTH-alternative expression was relocated unchanged. Native exporter/atom/family/
experimental-policy and both oracle constructor identities remain unchanged.
The new artifact constructor is `d4baldff…`; full JSON becomes `pf3ppbp4…` while
all249 outputs, all12 builders and existing native artifacts are byte-identical
to #219. No exporter/executor implementation or committed generated Rust changes.

Independent recursive derivation/full-build queries match all FOUR native verifier
inventories at1407/3392 each; the seed inventory now matches1437/3469. Both current
oracle outputs/derivations, current downstream executor and provider environments
are absent from those native/seed construction graphs; native graphs also exclude
pinned Go CUE/source and current full JSON. The complete default AST policy and
seed construction STILL depend on Go CUE; only this integer input is independently
native. Final immutable cross-review and exact-head protected CI/confirmed merge
remain required. #175 remains open/held with its original obligations.
