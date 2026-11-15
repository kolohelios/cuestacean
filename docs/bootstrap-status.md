# Bootstrap checkpoint and continuation guide

Checkpoint: merged [PR #53](https://github.com/kolohelios/cuestacean/pull/53),
commit `aa947f2a` on `main`. Treat this as a restart map; component contracts
and current tests remain authoritative as implementation advances.

## Direction and current boundary

Cuestacean targets Rust with executable parser, semantics, solver and generator
machinery progressively expressed in a CUE-hosted DSL. Merely using CUE to
configure a permanently handwritten implementation is not the end goal.

Current implementation: a deliberately small, checked i64 expression subset,
not full CUE or bootstrap closure. Go CUE still exports specifications. All five
DSL compilers and reference interpreters remain handwritten Rust. Performance
advantages remain a hypothesis, not a measured result.

### Executable CUE-owned machinery

| Boundary | Specifications and contract | Rust implementation/verification |
| --- | --- | --- |
| Evaluation rules | `bootstrap/evaluation.cue`, expression/rule schemas | `src/lib.rs`, `src/codegen.rs`; interpreted and generated rule execution |
| Token/trivia scanning | `bootstrap/*.cue`, [scanner contract](../bootstrap/scanner.md) | `src/scanner.rs`; generated scanners and reference interpreter |
| AST construction | `bootstrap/ast-*.cue`, [builder contract](../bootstrap/ast.md) | `src/ast.rs`; typed builders, no native parser JSON-map assembly |
| Pratt expression loop | `bootstrap/pratt.cue`, [control contract](../bootstrap/pratt.md) | `src/pratt.rs`, checked native frame callbacks in `src/parser.rs` |
| Primary dispatch | `bootstrap/primary.cue`, [primary contract](../bootstrap/primary.md) | Same control compiler; groups, negation, bounds, atoms, integers |
| Integer/bounds unification | `bootstrap/unify.cue`, [unification contract](../bootstrap/unify.md) | `src/unification.rs`; native `runtime::unify` delegates generated policy |

Paths beginning `src/` above are under `bootstrap/executor/`.
Generated Rust goldens are under `bootstrap/generated/`.

Native boundaries still include grammar validation/lookup, checked cursor and
frame operations, value representation/projection, finite comparison, arithmetic
primitives, JSON handling, concrete output, resource guards, and compiler code.
Do not describe the full parser/compiler/solver as self-hosted.

### Supported source and important semantics

See the [source profile](../bootstrap/source/README.md) for exact limits.

- Signed decimal integers; `int`; inclusive literal bounds `>=` / `<=`;
  parentheses; `+`, `-`, `*`, `&`; unary expression negation;
  `div`, `mod`, `quo`, `rem`.
- Binary operators are left-associative by default. `div/mod/quo/rem` share
  multiplication's precedence 20; addition/subtraction use 10, unification 5.
  Grammar data controls these choices. `/` is recognized for custom grammar
  configuration but remains unregistered by default: no rational arithmetic.
- CUE word scanning requires complete keywords and preserves a following minus.
  Partial/extended words do not advance. `7quo-3` is valid; `quo2` is not a token.
- Unary prefix selection uses CUE lookahead. Digit-adjacent minus stays on the
  signed-decimal path: `-9223372036854775808` must never be parsed as negating
  an out-of-range positive literal. Other prefixes recurse at CUE minimum 101,
  above the maximum configurable binary precedence 100.
- `-2` is an integer AST leaf; `-(2)` is a unary AST node. Optional negation
  constructor metadata can disable expression prefixes without disabling
  signed literals. `--1`, spaced signs and newline-after-sign are supported.
- `div/mod` are Euclidean; `quo/rem` truncate toward zero. Do not substitute
  floor division for Euclidean behavior with negative divisors. Quotient
  i64::MIN / -1 overflows; both remainders are zero. Avoid `abs(i64::MIN)`.
- Arithmetic requires concrete operands and checks intermediates, with no
  backward constraint propagation. Evaluate children left-first before primitive
  operand/zero checks. Child errors propagate before enclosing arithmetic.
- `int` and bounds may survive intermediate evaluation, but scalar output must
  be concrete. Singleton bounds remain incomplete until given a candidate.
  Missing endpoints are infinite, not clipped to i64 extrema.
- CUE may export integer zero as `-0`; native i64 output is canonical `0`.
  Differential checks normalize only that exact scalar spelling. Keep every
  nonzero integer byte-exact; do not use floating-point/jq normalization.
- Newline-after-operand rejection remains conservative, not full CUE comma
  insertion. Comments, declarations, references, structs/lists, strings, other
  scalar types, disjunctions/defaults and general constraint solving are absent.

### Resource and phase invariants

- Source parser: 64 AST nodes, 32 recursive expression levels, 4096 steps per
  expression frame shared with primary dispatch. Grouping adds depth but no AST
  node; unary construction adds one node. Both selected drivers propagate into
  groups, RHS expressions and unary recursion.
- Primary state is private. Reject reentry, unfinished results, out-of-order
  stages and expression mutation while primary dispatch is active.
- Scanner: checked initial UTF-8 boundary, ASCII-only advancement, checked
  lookahead up to 64 bytes; fuel `128 * (remaining bytes + 1)` saturating.
  Limits: 64 statements, 16 nesting levels, 128 predicate bytes.
- AST builders: typed/scoped parameters and bounded expressions; repeated owned
  arguments clone only before their final use, which moves.
- Unification control: finite expressions, no loops/fuel; 512 JSON nodes,
  64 JSON levels counting the root. Validate lexical locals, decode original
  text to reject duplicate fields, and preserve simultaneous initializer scope.

## Recent merged milestones

| PR | Milestone |
| --- | --- |
| [#41](https://github.com/kolohelios/cuestacean/pull/41) | CUE AST construction |
| [#43](https://github.com/kolohelios/cuestacean/pull/43) | CUE Pratt loop and binding-power policy |
| [#45](https://github.com/kolohelios/cuestacean/pull/45) | CUE primary dispatch/grouping |
| [#47](https://github.com/kolohelios/cuestacean/pull/47) | CUE integer/bounds unification control |
| [#49](https://github.com/kolohelios/cuestacean/pull/49) | Checked subtraction end to end |
| [#51](https://github.com/kolohelios/cuestacean/pull/51) | CUE-controlled unary negation |
| [#53](https://github.com/kolohelios/cuestacean/pull/53) | Integer division/remainder and word operators |

## Build and validation runbook

From the repository root (there is no root flake):

```sh
nix develop ./infra/tooling --command shaka preflight
nix develop ./infra/tooling --command shaka whitespace check
nix develop ./infra/tooling --command cargo test --manifest-path bootstrap/executor/Cargo.toml
nix build ./infra/tooling#exports --out-link result-exports
nix run ./infra/tooling#source -- bootstrap/source/valid/nested.cue
```

`infra/tooling/project.cue` owns generated Shaka orchestration and GitHub Actions;
do not hand-edit the generated justfile/workflow. `cuestacean validate` is the
required GitHub check. Preflight includes sandbox tests, strict Clippy, feature
profiles, deterministic exports/goldens and pinned upstream comparisons.

`CUESTACEAN_BOOTSTRAP_JSON` comes from the devshell/package. **Re-enter the shell
after CUE edits** to obtain fresh exports. Additional `jj` workspaces need an
explicit source root so relative flake filesets include `bootstrap/`:

```sh
nix develop "path:$PWD?dir=infra/tooling" --command cargo test --manifest-path bootstrap/executor/Cargo.toml
```

The ordinary nested-flake shorthand can resolve filesets to missing
`/nix/bootstrap` outside the main Git checkout. Never quietly use another
workspace's stale exports for changed specifications.

### Artifact and feature policy

- CUE-generated JSON is build-only. Handwritten JSON fixtures remain committed.
- Generated Rust stays committed for now, explicitly reaffirmed during compiler
  development. It is a review/drift baseline, not a required seed input.
- Nix deletes staged JSON/Rust, builds five handwritten compiler seeds without
  default features, regenerates everything, then compiles/tests the final package.
- Default `source` enables `semantics`. Semantic evaluation/generated unification
  can build without the parser; compiler-only seeds exclude both dependencies,
  with no native unification fallback or stub. The sandbox checks these profiles.
- See [exact regeneration loops](../bootstrap/generated/README.md). After adding
  a fixture/compiler, update the corresponding Nix fixture list, seed generation
  and drift loop; do not just update a golden and call the bootstrap validated.

### Test maintenance traps

- Source `valid/` fixtures compare outputs upstream; `conflicts/` and
  `incomplete/` require both tools to fail; `rejected/` may be valid upstream
  outside the admitted subset. Keep these meanings explicit.
- Conflict CLI tests retain integer-conflict assertions and explicitly map the
  four zero-divisor fixtures to exact division-zero diagnostics. Extend expected
  errors deliberately when adding other supported semantic failures.
- When expanding rule maps, update the handwritten reordered-rule fixture so
  its order-invariance test still compares equivalent maps.
- Unknown-kind fixtures now use `unknownTestKind`; avoid using a prospective
  feature name as the permanent unknown case.
- When reclassifying newly supported syntax, move old negative cases to positive
  coverage and explain any intentional diagnostic changes. Unary whitespace
  moved the dangling-sign/newline missing-operand diagnostic to EOF byte 2.
- Keep independent AST/value/error expectations: sharing a runtime primitive or
  reading the same CUE policy twice does not independently establish correctness.

## Resume after compaction

1. Read `AGENTS.md`, this checkpoint, relevant component contracts and current
   source/tests. Inspect live workspace state rather than trusting old revisions.
2. Fetch and start from `main@origin` after the latest maintainer merge. Use
   `jj workspace update-stale` when a shared operation has rewritten a worker.
3. No next implementation was started after #53. Previous workers/panes and
   additional workspaces were retired; only the default workspace remains.
4. Select a meaningful vertical slice and define interfaces before delegation.
   A candidate is CUE-owned comment/trivia handling toward file syntax, with
   explicit Unicode-cursor and newline-policy decisions. Structural values and
   references need a design first: current `Value` is Copy and APIs return i64.
   Do not assume another arithmetic feature is already selected.
5. Use at most two isolated Herdr workers, clear file ownership and a coordinator
   for integration/fresh generation. Read back reviews, fix findings, validate
   locally, open with `--no-auto-merge`, watch CI and clean up created workers.
   See [tooling notes](../infra/tooling/README.md) and `AGENTS.md`.

The approximate 1,500-line PR budget includes goldens. Leave merging to the
maintainer. Keep estimates and claims about compatibility/self-hosting cautious;
references, cycles, disjunctions/defaults and propagation remain major work.
