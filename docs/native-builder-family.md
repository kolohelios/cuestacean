# Native export of the complete AST-builder family

## Reviewed contract

Milestone [#171](https://github.com/kolohelios/cuestacean/issues/171), after #169:
extend the independent native export cut to the existing 12 builder specifications,
then make those artifacts the production JSON bundle's provider. Two focused design
reviews preceded implementation. This changes the build dependency graph, not the native evaluator's
language/limits, production source syntax or builder/compiler semantics.

The authoritative family is `builderFixtures` plus `tests/valid/ast-literal` in
`infra/tooling/flake.nix`: eight ordinary builders (program, atom, integer, text,
binary, ternary, unary, record-field), three alternates (program-v2, escaped,
duplicate argument use), and the literal used with the forced lowering policy.
`ast-lowering` and `ast-lowering-forced` are compiler policies, **not** members.
Use exact manifest membership, never an `ast-*` wildcard.

## Independent artifact and provider graph

Add `native-ast-builders`, receiving the unchanged exporter and a fileset of
exactly `ast-schema.cue` plus those 12 unmodified programs. Each export runs twice
in separate processes with empty environment/unusable PATH, compares bytes, and
publishes build-only JSON under its existing fixture-relative name. No Go CUE,
policy JSON, expected answers, generated Rust or compiler is an input.

The production `jsonExports` loop copies these already-checked native artifacts
and skips Go CUE for these positive fixtures. All other exports and negative CUE
schema tests retain their existing paths. Repeat-export evidence belongs to the
native producer, not to copying its output twice. Native failure is a failed
build, never a reason to retry with Go CUE.

```text
minimal exporter sources -> exporter -> native builder-family artifacts
raw builder schema + 12 programs -----------^
                                         |
other specs -> Go CUE ------------------> production JSON bundle -> six seeds
                                         |
                                         v
                                  generated/compiled Rust

native family -> separate Go CUE + downstream compiler verification
```

Keep the original `native-ast-atom` constructor and two-file proof intact; do not
silently widen its input closure. The family gets its own full build-closure
proof and separate verification check. Nothing in the native family depends on
`jsonExports`, its own verifier, or the downstream compiler: that would introduce
a cycle or invalidate independence.

## Evidence and scope

- Before changes, pinned native-vs-Go-CUE exports of all 12 matched byte-for-byte
  under the existing `jq -S .` artifact convention, within unchanged budgets.
- Add independent handwritten expected specification JSON for every member,
  including parameter/field order, nested construction, version 2 as builder data,
  escaped UTF-8 metadata, repeated argument use, and literal 7 versus forced output.
  Compare native, expected and pinned upstream; do not derive expectations from
  generated artifacts or the Go CUE result. Binding names remain DSL data.
- Add lowered-budget failures and direct native CLI coverage with no Go CUE on
  PATH. Do not weaken existing structural/copy/output limits or tests.
- Inspect full derivation roots for exporter/family (not runtime references only).
  Retain paths/registration/roots and reject pinned Go CUE/source/JSON-export
  dependencies. The identity denylist is supporting evidence, not a universal
  foreign-code detector; inspect allowlisted sources and the whole graph too.
- Separately compare fresh pinned upstream exports with every native artifact,
  regenerate each builder Rust module using its existing compiler/policy and
  compare goldens. The literal uses its existing forced-policy check. Existing
  fresh-seed compilation and AST execution tests retain independent expectations.
- Compare the **entire** production JSON output tree before/after routing; keep
  generated Rust unchanged. Run both Rust suites/Clippy, all existing Nix checks,
  fresh six artifact-free compiler binaries, profiles/drift and source/value checks.

## Validation and preserved boundaries

All 12 source exports matched pinned upstream before routing; the complete
production JSON output tree remains byte-identical after routing. Three new tests
cover the family against handwritten expectations/upstream, cleared-environment
CLI processes, source order, repeat bytes and lowered budgets; all 12 native crate
tests pass in debug/release with strict Clippy. Independent full source/value and
fresh-seed generation/profile checks pass. Full Rust/Clippy passes; pinned
preflight passes 13/13 (7 inapplicable checks skipped), with all eight Nix checks
explicitly rebuilt in actual sandboxes and fresh six compiler binaries retained.

At #171, the inspected family graph had 1407 build derivations/3392 store paths; its raw
source contains exactly 13 files. Pinned Go CUE/source/production JSON-export
identities are absent. That step preserved the original exporter/atom derivation
identities without parser/evaluator or guard changes. Later exporter changes such
as [#173 lexical bindings](native-let-bindings.md) retain this raw family boundary
and re-attest the new exporter/artifact derivations. Verification keeps paths,
reference registration and roots separate from the native artifact. Only
x86_64-linux is validated; incompatible aarch64 systems are omitted, not tested.

```sh
nix build --no-link ./infra/tooling#native-ast-builders
nix build --no-link ./infra/tooling#checks.x86_64-linux.native-builder-family
```

At #171 all AST *specification exports* became independently native, not the
compiler policies. Subsequently [#222](native-ast-lowering-policy.md) independently
constructs the complete canonical `ast-lowering.json` DATA from separately admitted
modules; other lowering policies still require Go CUE. This does not imply native
schema evaluation, general references/defaults/imports, a wider source profile,
self-hosting or a performance claim. Subsequent policy-closure/interface work needs its own
bounded selection; the native bootstrap policy remains explicitly native.
