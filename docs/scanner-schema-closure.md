# Scanner specification closure

## Purpose and review gate

Follow [the closure inventory](bootstrap-closure.md) and the completed native AST
builder family. Select the next scanner closure WITHOUT another guessed default
algorithm: isolate the unchanged scanner definitions and make existing Go-CUE
scanner construction use their exact raw closure. Milestone
[#194](https://github.com/kolohelios/cuestacean/issues/194). Two focused contract
reviews preceded implementation, confirming exact extraction and native rejection. #175 remains held; no source evaluator, native
exporter language, semantic policy, generated Rust or resource-guard changes.

This is production SOURCE-CLOSURE progress plus independent characterization,
not native scanner construction, scanner compiler closure, CUE self-hosting or a
performance claim. Go CUE remains an explicit constructor for scanner artifacts.
This narrows each scanner's explicit CUE invocation, NOT the entire `exports`
derivation: the combined bundle still receives all its other schemas/programs.
The two-file characterization is not an independent native artifact or a sealed
two-file Nix build-closure attestation.

## Exact schema extraction

Move the contiguous block of #Scan* definitions from bootstrap/schema.cue into
new bootstrap/scanner-schema.cue, package bootstrap. Preserve every definition,
constraint, comment and recursive reference unchanged (including optional ahead,
uint/ranges, required list prefixes, character regex, bool, flat empty-list default
else). No duplicate old definitions or deliberate schema weakening. Existing
expression/control/grammar definitions stay in schema.cue; ast-schema stays intact.

The scanner schema has no import or dependency on unrelated expression/control
schemas. Production scanner exports must select scanner-schema.cue instead of
schema.cue, plus exactly their existing explicit scanner-fragment closure and one
program. Do NOT hardcode an opcode/filename answer or strip constraints to make
native export appear feasible. Inventory scanner programs by actual #ScanProgram
embedding as an independent check of the existing explicit scanner fixture list.

Non-scanner exports keep their existing source closures. All negative fixtures
requiring scanner definitions must explicitly load the new schema; missing
references are never accepted as a successful negative test. Review every raw
CUE invocation and fileset rather than relying on ambient package directory
loading. Source filenames are build orchestration, never evaluator semantics.

## First selected raw target and native boundary

Choose unmodified bootstrap/group-open.cue with scanner-schema.cue as the first
exact two-file scanner target. It must export the same complete handwritten JSON
through pinned Go CUE, in either file order and under renamed copies. Go CUE
schema-only export is the empty object (definitions are not program fields),
not a scanner program; program-only must fail missing schema, not silently
construct a fixture answer. Existing scanner fragments remain separate inputs
where their programs actually reference fragment definitions.

Characterize (do NOT implement) the current native export rejection of this
closure and the specific remaining schema/profile requirements. The first target
being operationally simple does not let the native exporter ignore unsupported
optional/regex/bool/bounds/default syntax in the full scanner schema. Definitions
must not be stripped or rewritten to evade its admission boundary.

Independent schema tests must pin positive/negative constraints with fresh CUE
processes and handwritten expected JSON/classifications:

- Missing else supplies [], explicit else retains statements, recursive nested
  statements instantiate independently; no claim of general default lifetime.
- ahead omitted/0/64 versus 65/negative; bytes 0/127 versus128 and empty required
  prefix; bool flag values versus wrong scalar type; repeat 0/64 versus65.
- One Unicode scalar versus multiple characters/empty string; nonempty rejection
  reason and closedness/unknown fields; malformed semantic cases produce no JSON.
- Export order/repeat/rename determinism; existing invalid fixture evidence remains
  intact. Go CUE source pin, not display version alone, is authoritative.

Production scanner JSON bytes and ALL full-bundle outputs must remain identical
to merged main, as must committed generated Rust and all independent source/value
answers. Capture old build-only exports for drift comparison, never seed the new
producer from them. A changed full JSON derivation is expected and must be
identified honestly: raw source ownership changes, not output content.

## Ownership and construction boundaries

After both reviews and explicit GO: extraction owner ONLY schema.cue, new
scanner-schema.cue and infra/tooling/flake.nix; evidence owner ONLY new
bootstrap/spec-export/tests/scanner_boundary.rs. Coordinator docs, integration,
raw-source/whole-bundle comparison and final validation. Additional files need
explicit ownership agreement first. Stable merged-main workspaces; exact immutable
commits, no mutable-parent rebases; at most two workers. No suppression.

The FOUR native constructors (exporter, AST atom, AST family, demand-policy family)
and four policy hashes must remain unchanged, with exact 8/6 source cuts intact.
The complete Go-CUE JSON bundle .drv and downstream seed/verifiers legitimately
change with this extraction. Do not claim the five old .drv identities remain
unchanged or that the whole scanner/seed graph is Go-CUE-free. Re-attest native
full graphs excluding CURRENT full JSON, executor, CUE/source and observer, and
seed graph excluding observer. Existing observer binary/provider scope unchanged.

The spec-export TEST fileset gains scanner-schema.cue and group-open.cue only as
raw characterization inputs; production eight-file native source cut is unchanged.
Expected JSON stays handwritten in tests and generated JSON build-only. Native
constructor inputs may not include expected answers, old outputs or test oracle.
No hidden conversion of scanner programs or accidental production fallback.

## Validation

Focused exporter tests/debug+release and strict Clippy, both full Rust suites/
default all-target Clippy debug+release; preserve all existing four-profile graph/
oracle tests. Require scoped Go observer tests, pinned full preflight and all TEN
actual Nix checks and native/export/oracle package builds/rebuilds. Keep six fresh
artifact-free compilers, feature isolation, deterministic generation/drift, twelve
native AST builders and independent source/value evidence. Inspect recursive build
and derivation graphs, not merely runtime refs. Linux x86_64 only; pinned CUE
6d609d768f1686f9a3a2a20197cacdbb70e5c79d (v0.16.1), Nixpkgs
ac588ef1c914865dad517b337ea0eb9694a4021c. Stay near 1500 changed lines.

### Completed evidence

- Immutable extraction `5e56c2ddd5c64e3e6a46eb88d33e3611da4fb114` and independent
  evidence `e4d687afce1420e779ee64bbf2b4e02aaa36d0b5`, both from merged #192.
  Read-only implementation cross-reviews approved both; workspaces retired.
- All 15 definitions/comments match the old contiguous block byte-for-byte apart
  from the package header/trailing separator. Independently checked inventory:
  46 scanner exports (39 direct embeddings, seven fragment-root programs), plus
  18 direct scanner-negative embeddings covered by explicit schema loading.
- All 249 full-bundle JSON files compare byte-identical with old exports. Full
  JSON constructor changes from `9ygahizw…` to `rsgf7sm4…`; no committed generated
  Rust or independent expected answers change. No baseline enters construction.
- All 36 exporter groups, including seven new boundary groups, pass debug/release
  with strict all-target Clippy. Both full Rust suites/default all-target Clippy
  pass debug/release; 76 retained graph/oracle groups and 14 solver groups pass
  default/no-default debug/release with strict scoped Clippy.
- Pinned preflight passes 13/13 (seven inapplicable skips); all ten Linux checks
  and six selected packages build AND rebuild. Observer Go tests run in its
  offline pinned sandbox. Whole-repository whitespace checks 1287 text files.
- Four native constructor identities, observer identity and all four policy-file
  hashes are unchanged; exact eight-file exporter/six-file policy cuts remain.
  Independent recursive queries match verifier path inventories: native 1407
  derivations/3392 full build paths; seed 1436/3466. Current full JSON, executor,
  pinned CUE/source and observer remain outside native construction; observer
  remains outside the seed graph and provider environment. These checks do not
  establish a Go-CUE-free compiler seed or minimal seed source attestation.

Native scanner construction remains unimplemented; #175 remains held.
