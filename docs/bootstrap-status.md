# Bootstrap checkpoint and continuation guide

Checkpoint: merged [PR #53](https://github.com/kolohelios/cuestacean/pull/53),
commit `aa947f2a` on `main`. Treat this as a restart map; component contracts
and current tests remain authoritative as implementation advances.

## Direction and current boundary

Cuestacean targets Rust with executable parser, semantics, solver and generator
machinery progressively expressed in a CUE-hosted DSL. Merely using CUE to
configure a permanently handwritten implementation is not the end goal.

Current implementation: a deliberately small null/top/boolean/checked-i64/string expression
subset ([string syntax](../bootstrap/strings.md), [ownership](string-values.md)),
not full CUE or bootstrap closure. Go CUE still exports specifications. Six
bootstrap compilers still require native decoders, kernels and policy-compilation
machinery; increasingly their executable lowering policies live in CUE. Performance
advantages remain a hypothesis, not a measured result.

### Executable CUE-owned machinery

| Boundary | Specifications and contract | Rust implementation/verification |
| --- | --- | --- |
| Lowering-driver compiler | `bootstrap/meta-lowering{,-fragments}.cue`, [meta contract](../bootstrap/meta-lowering.md) | Self-regenerating full source emitter; native decoder, metadata projection and checked VM/kernels |
| Scanner compiler lowering | `bootstrap/scan-lowering{,-fragments}.cue`, [backend contract](../bootstrap/scan-lowering.md) | Native flow/label facts and bounded patterns; CUE source/branch layout; compiled driver |
| AST-builder compiler lowering | `bootstrap/ast-lowering{,-fragments}.cue`, [backend contract](../bootstrap/ast-lowering.md) | Native checked binding/use-count facts; CUE layout/clone/move selection; compiled driver |
| Inclusive bound construction | `bootstrap/{lower,upper}-bound.cue`, [bound control](../bootstrap/bound-control.md) | Unary CUE threshold/domain/layout; checked left-only endpoints, no native constructor fallback |
| Concrete-output admission | `bootstrap/{json,integer}-output.cue`, [output control](../bootstrap/output-control.md) | Unary CUE domain/incomplete diagnostics; checked native extraction/JSON encoding |
| Checked identifier styles | [identifier rendering](../bootstrap/identifier-rendering.md), shared kernel/meta policy | CUE-selected snake/initial-capital rendering; 431-value unification backend under unchanged 512 guard |
| Unification compiler lowering | `bootstrap/unify-lowering{,-fragments}.cue`, [backend contract](../bootstrap/unify-lowering.md) | Shared select/concat kernel and compiled driver; native typed/scope validation |
| Pratt/primary compiler lowering | `bootstrap/pratt-lowering{,-fragments}.cue`, [backend contract](../bootstrap/pratt-lowering.md) | Shared lowering compiler/kernel; compiled driver; native typed bridge |
| Evaluator module assembly | `bootstrap/lowering-fragments.cue`, [module contract](../bootstrap/module.md) | `src/module.rs` checked cursor/output kernel; compiled/reference control |
| Instruction-to-Rust lowering | `bootstrap/lowering{,-fragments}.cue`, [lowering contract](../bootstrap/lowering.md) | `src/lowering.rs` compiler/reference engine/shared kernel; generated Rust driver; native checked primitives |
| Evaluation rules | `bootstrap/evaluation.cue`, expression/rule schemas | `src/lib.rs`, `src/codegen.rs`; interpreted and generated rule execution |
| Token/trivia scanning | `bootstrap/*.cue`, [scanner contract](../bootstrap/scanner.md) | `src/scanner.rs`; generated scanners and reference interpreter |
| AST construction | `bootstrap/ast-*.cue`, [builder contract](../bootstrap/ast.md) | `src/ast.rs`; typed builders, no native parser JSON-map assembly |
| Pratt expression loop | `bootstrap/pratt.cue`, [control contract](../bootstrap/pratt.md) | `src/pratt.rs`, checked native frame callbacks in `src/parser.rs` |
| Primary dispatch | `bootstrap/primary.cue`, [primary contract](../bootstrap/primary.md) | Same control compiler; groups, negation, bounds, atoms, integers |
| Division/remainder policy | `bootstrap/{divide,modulo,quotient,remainder}-control.cue`, [division control](../bootstrap/division-control.md) | CUE domain/zero/mode/unit-divisor selection; native checked division math |
| Checked integer dispatch | `bootstrap/{negate,difference,product}-control.cue`, [integer control](../bootstrap/integer-control.md) | Same unary/binary compiler; CUE domain/operation policy over borrowed checked kernels |
| Unary logical not | `bootstrap/logical-not.cue`, [unary control](../bootstrap/logical-not.md) | Checked unary arity/subset; same compiler and generated one-operand policy |
| Logical value selection | `bootstrap/{logic-fragments,logical-and,logical-or}.cue`, [logical control](../bootstrap/logic.md) | Same finite binary compiler; native boolean inspection; CUE concreteness/result policy |
| Scalar ordering | `bootstrap/{ordering-fragments,less,less-equal,greater,greater-equal}.cue`, [ordering control](../bootstrap/ordering.md) | Same finite binary compiler; native finite/text comparisons; generated domain/result policy |
| Scalar equality | `bootstrap/{equal,not-equal,equality-fragments}.cue`, [equality control](../bootstrap/equality.md) | Same finite binary compiler; native content equality; generated concreteness/result policy |
| Scalar addition | `bootstrap/sum.cue`, [binary control](../bootstrap/sum.md) | Same finite control compiler; native checked integer/text kernels; generated policy |
| Null/top boundaries | `bootstrap/unify.cue`, [lattice values](../bootstrap/lattice-values.md) | Native scalar representation/output; CUE identity, domain and concreteness policy |
| Scalar unification | `bootstrap/unify.cue`, [unification contract](../bootstrap/unify.md) | `src/unification.rs`; native `runtime::unify` delegates generated policy |

Paths beginning `src/` above are under `bootstrap/executor/`.
Generated Rust goldens are under `bootstrap/generated/`.

All arithmetic dispatch now uses CUE control, including division/remainder
zero checks and unit-divisor selection. Checked integer numeric computations
and defensive kernel representation/zero checks remain native.

Native boundaries still include grammar validation/lookup, checked cursor and
frame operations, lowering compiler/rendering and cursor kernels, value representation/projection, finite comparison, arithmetic
primitives, JSON handling/encoding, checked output extraction, resource guards, and compiler code.
Concrete-output admission and incomplete diagnostic selection now live in CUE.
Do not describe the full parser/compiler/solver as self-hosted.

### Supported source and important semantics

See the [source profile](../bootstrap/source/README.md) for exact limits.

- Concrete `null` and abstract top `_`, using existing CUE atom construction.
  Top is unification identity before domain guards; null agrees only with null
  or top. Null is concrete for equality and logical selection, never numeric.
- Single-line quoted UTF-8 strings with seven common escapes, four-digit
  `\uXXXX` and valid UTF-16 surrogate pairs ([contract](../bootstrap/unicode-escapes.md)),
  plus eight-digit scalar `\UXXXXXXXX` ([wide contract](../bootstrap/wide-unicode.md)); abstract `string`;
  signed decimal and prefixed 0b/0o/0x/0X integers ([contract](../bootstrap/radix-literals.md)); `true`/`false`; `bool`/`int`; inclusive bounds `>=` / `<=` with literal or
  [parenthesized computed thresholds](../bootstrap/computed-bounds.md);
  parentheses; `+`, `-`, `*`, `&`, `&&`, `||`; unary `!` and expression negation;
  `div`, `mod`, `quo`, `rem`; concrete scalar `==` / `!=`; integer/text `<` / `<=` / `>` / `>=`; CUE-owned UTF-8 `//` comment trivia.
- Binary operators are left-associative by default. `div/mod/quo/rem` share
  multiplication's precedence 20; addition/subtraction use 10, unification 5.
  Default `||` uses 6, `&&` 7, comparisons 8 (previously 7). This preserves all
  earlier relative precedences while inserting the two logical levels.
  Grammar data controls these choices. `/` is recognized for custom grammar
  configuration but remains unregistered by default: no rational arithmetic.
- CUE word scanning requires complete keywords and preserves a following minus.
  Partial/extended words do not advance. `7quo-3` is valid; `quo2` is not a token.
- Unary prefix selection uses CUE lookahead. Digit-adjacent minus stays on the
  signed-integer path: `-9223372036854775808` must never be parsed as negating
  an out-of-range positive literal. Other prefixes recurse at CUE minimum 101,
  above the maximum configurable binary precedence 100.
- `!` uses CUE prefix/primary control and the unary AST builder, at minimum 101.
  It accepts concrete booleans only. Unary control has a real one-value signature;
  unused right references and binary operations reject, without a dummy operand.
- `-2` is an integer AST leaf; `-(2)` is a unary AST node. Optional negation
  constructor metadata can disable expression prefixes without disabling
  signed literals. `--1`, spaced signs and newline-after-sign are supported.
- `div/mod` are Euclidean; `quo/rem` truncate toward zero. Do not substitute
  floor division for Euclidean behavior with negative divisors. Quotient
  i64::MIN / -1 overflows; both remainders are zero. Avoid `abs(i64::MIN)`.
- CUE-owned `+` dispatch adds concrete integers or concatenates two concrete
  strings. Other arithmetic remains integer-only; no implicit coercion is added.
  The Arithmetic diagnostic is now `arithmetic requires compatible concrete operands`.
- Logical children evaluate left-first, including RHS errors before result
  selection. Both values must be concrete; CUE can ignore a concrete RHS's type
  when the left determines the boolean (`false && 1` / `true || "x"`). This is
  not lazy child evaluation. Abstract/bound operands and selected nonbooleans fail.
- Arithmetic requires concrete operands and checks intermediates, with no
  backward constraint propagation. Evaluate children left-first before primitive
  operand/zero checks. Child errors propagate before enclosing arithmetic.
- Boolean identity/equality and mixed-domain rejection live in CUE unification
  control, before numeric identity. Never let `int & true` return `true`.
  Scalar `==`/`!=` use CUE concreteness guards before native `same`; concrete
  mixed types compare unequal, while abstract types/bounds reject. Ordering
  requires matching concrete integers or text, with lexicographic UTF-8 comparison.
- `bool`, `int` and bounds may survive intermediate evaluation, but scalar output must
  be concrete. Singleton bounds remain incomplete until given a candidate.
  Missing endpoints are infinite, not clipped to i64 extrema.
- CUE may export integer zero as `-0`; native i64 output is canonical `0`.
  Differential checks normalize only that exact scalar spelling. Keep every
  nonzero integer byte-exact; do not use floating-point/jq normalization.
- Newline-after-operand and newline-after-word-operator rejection remain
  CUE-owned and conservative, not full CUE comma insertion. Only LF ends `//` comments; CR alone remains comment content.
  NUL/BOM inside comments are rejected by CUE. Block comments, declarations,
  references, structs/lists, general string syntax, further scalar types, disjunctions/defaults
  and general constraint solving are absent.
- JSON scalar APIs (`execute_json`, `generated::execute_json`) preserve null,
  strings, bools and exact i64s. Top output is `non-concrete value`. Legacy `execute` rejects concrete strings/booleans.
  JSON AST `string`/`stringType` now also come from the bounded CUE source-string profile.
  The public default generated evaluator is gated by `semantics`; fresh seeds
  omit it, with no stubs. Values are Clone, not Copy; shared text uses `Arc<str>`.
  Structural/reference graph ownership still needs a separate design.

### Resource and phase invariants

- Source parser: 64 AST nodes, 32 recursive expression levels, 4096 steps per
  expression frame shared with primary dispatch. Grouping adds depth but no AST
  node; unary construction adds one node. Both selected drivers propagate into
  groups, RHS expressions and unary recursion.
- Primary state is private. Reject reentry, unfinished results, out-of-order
  stages and expression mutation while primary dispatch is active.
- Scanner: checked initial UTF-8 boundary, ASCII-only `advance`, explicit whole-
  scalar `advanceChar`, scoped `break` and Unicode `charIn`, checked lookahead
  up to 64 bytes; fuel `128 * (remaining bytes + 1)` saturating.
  Limits: 64 statements, 16 nesting levels, 128 predicate bytes. Pure `and`/`or`
  trees have separate whole-program bounds of 128 predicate nodes/16 levels;
  CUE owns normal/De Morgan emission ([contract](../bootstrap/scanner-predicates.md)).
  Literal repeat counts 0..64 share the same structural/fuel limits; CUE emits
  counted/once/zero forms ([contract](../bootstrap/scanner-repeat.md)).
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
| [#56](https://github.com/kolohelios/cuestacean/pull/56) | CUE-owned Unicode line comments and word-newline policy |
| [#58](https://github.com/kolohelios/cuestacean/pull/58) | Boolean domain, CUE typed unification and JSON scalar APIs |
| [#60](https://github.com/kolohelios/cuestacean/pull/60) | Interpreted CUE instruction-to-Rust lowering |
| [#62](https://github.com/kolohelios/cuestacean/pull/62) | Compiled production lowering with artifact-free interpreter seeds |
| [#64](https://github.com/kolohelios/cuestacean/pull/64) | CUE evaluator assembly/traversal and reverse compiled evidence |
| [#66](https://github.com/kolohelios/cuestacean/pull/66) | Shared CUE backend for Pratt/primary compiler emission |
| [#68](https://github.com/kolohelios/cuestacean/pull/68) | Shared CUE backend for unification compiler emission |
| [#70](https://github.com/kolohelios/cuestacean/pull/70) | Shared CUE backend for AST builders and ownership selection |
| [#72](https://github.com/kolohelios/cuestacean/pull/72) | Shared CUE backend for scanner source and branch layout |
| [#74](https://github.com/kolohelios/cuestacean/pull/74) | CUE-defined self-regenerating lowering-expression emitter |
| [#76](https://github.com/kolohelios/cuestacean/pull/76) | CUE-defined complete lowering-driver/module-statement emission |
| [#78](https://github.com/kolohelios/cuestacean/pull/78) | Owned JSON string scalars and CUE string-domain unification |
| [#80](https://github.com/kolohelios/cuestacean/pull/80) | CUE-controlled single-line source strings and abstract `string` |
| [#82](https://github.com/kolohelios/cuestacean/pull/82) | CUE-owned scalar addition dispatch and text concatenation |
| [#84](https://github.com/kolohelios/cuestacean/pull/84) | CUE-owned concrete scalar equality and inequality |
| [#86](https://github.com/kolohelios/cuestacean/pull/86) | CUE-owned complete integer/text ordering family |
| [#88](https://github.com/kolohelios/cuestacean/pull/88) | CUE-owned logical selection and export-schema resource fixes |
| [#90](https://github.com/kolohelios/cuestacean/pull/90) | CUE-owned unary logical not and checked one-operand control |
| [#92](https://github.com/kolohelios/cuestacean/pull/92) | CUE-owned null/top lattice boundaries |
| [#94](https://github.com/kolohelios/cuestacean/pull/94) | CUE-owned negation/subtraction/multiplication dispatch |
| [#96](https://github.com/kolohelios/cuestacean/pull/96) | CUE-owned division/remainder domain/zero/mode/exception policy |
| [#98](https://github.com/kolohelios/cuestacean/pull/98) | Checked identifier styles and compiler policy headroom |
| [#100](https://github.com/kolohelios/cuestacean/pull/100) | Unary CUE concrete-output admission and diagnostics |
| [#102](https://github.com/kolohelios/cuestacean/pull/102) | Unary CUE inclusive bound construction and checked left-only endpoints |
| [#104](https://github.com/kolohelios/cuestacean/pull/104) | CUE-only four-digit Unicode escape and surrogate-pair admission |
| [#106](https://github.com/kolohelios/cuestacean/pull/106) | Bounded composable scanner predicates and Unicode control compaction |
| [#108](https://github.com/kolohelios/cuestacean/pull/108) | Bounded repeat control and counted Unicode digit validation |
| [#110](https://github.com/kolohelios/cuestacean/pull/110) | Eight-digit Unicode scalar admission and checked JSON bridge |
| [#112](https://github.com/kolohelios/cuestacean/pull/112) | Checked prefixed integers and inclusive thresholds |
| [#114](https://github.com/kolohelios/cuestacean/pull/114) | CUE-controlled computed integer bound thresholds |

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
`bootstrap-values` additionally checks JSON string semantics against Go CUE;
these probes are not admitted source-parser fixtures.

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
- Nix deletes staged JSON/Rust, builds six native compiler/VM seeds without
  default features, regenerates everything, then compiles/tests the final package.
  Seeds now embed fresh upstream-exported lowering-policy JSON; exports must not
  depend on seeds or committed artifacts. No native opcode-emission fallback.
  Seeds interpret the policy; default `compiled-lowering` uses generated Rust.
  The reference interpreter remains for explicit custom policies, sharing exact
  rendering/quoting/resource primitives. `compiled-lowering` is independent of
  semantics/source and has its own feature check. The same driver now assembles
  modules with CUE-owned statements; expression-only custom policies cannot
  silently fall back to native assembly. See the module contract for new bounds.
  The Pratt compiler now uses a separate policy through the same lowering engine;
  seeds interpret it and production uses `compiled_pratt_lowering`. Keep this
  driver in the feature-isolation removal set, not only `lowering*.rs`.
  Unification compilation now has the same arrangement via `unify-lowering.rs`.
  Its internal typed serialization distinguishes value/endpoint input tags;
  accepted control wire syntax remains unchanged.
  AST-builder compilation likewise uses `ast-lowering.rs`; native binding/type/use
  counting supplies data, CUE chooses cloning/moving and source layout. Include
  this driver in feature-isolation removal. See the AST backend contract.
  Scanner compilation now uses `scan-lowering.rs` with native flow/label facts
  and bounded pattern rendering. CUE owns source composition and cached branch
  simplification; include this driver in feature-isolation removal too.
  `meta-lowering.rs` now emits whole lowering drivers, including its own regeneration;
  seeds interpret fresh meta JSON. Remove it for semantics-only isolation too.
  Native code projects canonical dispatch/module facts; CUE owns all source assembly.
  One global compiler context includes shell/module output. Typed decoding, checked
  VM/kernels and Go export remain boundaries, not full compiler bootstrap closure.
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
  four zero-divisor fixtures and boolean/type conflicts to their diagnostics. Extend expected
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
3. The original #54 restart point had no implementation in flight. Subsequent
   work adds line-comment trivia, Unicode/break primitives and boolean scalars; inspect
   current history and live workers instead of relying on that old session state.
4. Select a meaningful vertical slice and define interfaces before delegation.
   Line-comment handling is now covered by the scanner contract. Structural
   values and references need a design first: `Value` now owns shared text and is
   Clone, not Copy. The single-line source-string profile is deliberately narrower
   than JSON/CUE string syntax. Do not assume another arithmetic feature is selected.
5. Use at most two isolated Herdr workers, clear file ownership and a coordinator
   for integration/fresh generation. Read back reviews, fix findings, validate
   locally, retire created workers, enable CI-gated auto-merge and watch required
   CI through confirmed merge. Never bypass checks or use administrative overrides.
   See [tooling notes](../infra/tooling/README.md) and `AGENTS.md`.

The approximate 1,500-line PR budget includes goldens. CI-gated auto-merge is
permitted after focused reviews and local validation; continue from confirmed
merged main without requiring another maintainer prompt. Pause for ambiguous
scope/design, security-sensitive changes or unresolved validation failures.
Keep estimates and claims about compatibility/self-hosting cautious;
references, cycles, disjunctions/defaults and propagation remain major work.
