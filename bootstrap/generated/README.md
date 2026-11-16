# First Rust generation boundary

The evaluator modules are generated from rule JSON by `bootstrap-codegen`,
whose per-instruction expression lowering runs a [compiled CUE program](../lowering.md).
Checked rendering/cursor kernels and typed metadata projection remain native;
[complete driver source emission](../meta-lowering.md) is CUE-owned;
[module assembly](../module.md) and traversal are now CUE-owned too. Compiler-only seeds use the reference policy interpreter. This lowers our eighteen instruction primitives, not
arbitrary CUE source. The interpreter remains the default executable and a
reference for tests. The normal generated module also runs as a separate CLI:

```sh
nix build ./infra/tooling#exports --out-link result-exports
nix run ./infra/tooling#generated -- result-exports/addition.json
```

Run that command from the repository root; it prints `42`. The rules are baked
in, so this CLI accepts only a program path. The alternate and escaping modules
remain test-only.

`meta-lowering.rs` is the CUE-defined full driver compiler; fresh seeds interpret
its policy, then the compiled emitter regenerates its whole source byte-exact.
`lowering-meta-forced.rs` changes unchanged number-lowering IR's actual compiled
nonzero output to `forced` rather than `more`. `meta-driver-keys.rs` and
`meta-driver-previous.rs` compile unchanged keys-only module IR: the default emits
both quoted keys, while an alternate meta compiler emits only the first.

- `evaluation.rs` comes from the normal CUE-exported rules.
- `evaluation-left-twice.rs` comes from the alternate CUE-exported rules.
- `codegen-escaped-rules.rs` comes from an adversarial JSON fixture that tests
  escaping names containing quotes, backslashes, a newline, and Unicode. This
  fixture exercises the emitter, not the narrower CUE schema. It also changes
  the `true` rule to emit false, checking compiled boolean rule-data control.

`whitespace.rs` is generated from a different, executable [scanner DSL](../scanner.md):
its loop and newline branch come from shared CUE statements in
`scanner-fragments.cue`. `after-operand.rs` composes those statements with
state/EOF checks and rejection policy. The shared trivia loop now includes
CUE-owned `//` comments, UTF-8 scalar advancement and scoped breaks. `group-open.rs` recognizes the opener;
`group-close.rs` composes whitespace scanning with closing-token validation,
consumption, and diagnostics. `bound-prefix.rs` and `infix.rs` recognize symbol
spans; grammar lookup remains native. `decimal.rs` uses the
same DSL for optional-minus/digit scanning and lexical rejection; `identifier.rs`
supplies ASCII identifier start/continuation scanning. Checked i64 conversion,
grammar validation and keyword lookup remain native. The `scanner-horizontal`,
`scanner-stalled`, and `scanner-advance` modules are test-only algorithms that
exercise alternate branches, non-progress, and native cursor guards.
`scan-lowering.rs` now compiles scanner layout from the shared CUE backend;
`scan-unicode-advance.rs` changes the compiler policy, not scanner IR, and consumes
λ where native ASCII advancement rejects. `lowering-bindings.rs` pins lexical
rendered-value captures, escaping, equality and lazy boolean selection.
`scanner-decimal-permissive.rs` demonstrates alternate lexical policy;
`scanner-reject.rs` checks escaped reasons and rejection short-circuiting;
`scanner-state.rs` tests flag reads, stateful loops, and byte-presence checks;
`scanner-lookahead.rs` checks maximum lookahead and EOF handling.

The `ast-program`, `ast-atom`, `ast-integer`, `ast-binary`, and `ast-unary` modules come from
the separate [AST construction DSL](../ast.md). They replace native parser map
assembly. `ast-program-v2`, `ast-escaped`, and `ast-duplicate` are test-only
programs for changed policy, escaping, and repeated owned arguments.
`ast-lowering.rs` is their shared [CUE compiler backend](../ast-lowering.md);
`ast-forced-integer.rs` compiles the same literal-7 input with an altered integer
compiler case and returns 99. `lowering-numbers.rs` tests compiled signed extrema,
escaped joined separators and lazy zero branching.

`string-literal.rs` supplies the [source string lexical subset](../strings.md);
`ast-text.rs` builds its owned string leaf. Native code only checks spans/stages and
decodes the already admitted escape subset. Both regenerate in fresh seed builds.

`pratt.rs` is the generated [expression control program](../pratt.md), containing
the operator loop, binding-power condition and associativity branch. The
`pratt-right`, `pratt-stalled`, `pratt-invalid-state`, and `pratt-overflow` drivers
are test-only programs. Frame operations remain native. `primary.rs` uses the
same compiler for [primary dispatch](../primary.md), group commitment, recursive
minimum and closing order. `primary-no-atoms` and `primary-high-minimum` are
test-only policies, not production fallbacks. `primary-low-negation` weakens
the CUE-supplied unary binding power. `negation-prefix` is a generated scanner
that distinguishes signed decimal literals from expression negation. `primary-cross-phase` deliberately
attempts expression operations inside primary dispatch to exercise state guards.

Generated evaluator code contains node-kind dispatch and lowered rule expressions. It
shares the handwritten `bootstrap_executor::runtime` primitives for JSON
validation, field access, integer/type values, inclusive bounds, unification,
and checked integer arithmetic, including `difference`/subtraction. Unification delegates to `unify.rs`, generated
from the [unification DSL](../unify.md); `unify-right` and `unify-scoped` are
test-only policies. Addition now delegates to `sum.rs` through the same finite
binary-control DSL ([contract](../sum.md)); `sum-right` is test-only.
`equal.rs` and `not-equal.rs` implement [scalar equality](../equality.md) through
that same compiler; `equal-false` is test-only. Four [ordering policies](../ordering.md)
use it too, with `less-false` as a compiled test-only alternate. Intermediate values may be abstract `bool`, `int` or
bounds; JSON evaluation also supports owned strings and abstract `string`.
Public `execute_json` requires a concrete JSON integer/boolean/string;
legacy `execute` remains integer-only. The normal module is exposed as
`bootstrap_executor::generated` under `semantics`; both compiled CLIs reuse it. Tests compare values
and errors against the interpreter and known expectations. Sharing primitives
means this is not an independent test of their correctness or of CUE conformance.

Regenerate from the repository root in `nix develop ./infra/tooling`. Re-enter
the shell after editing CUE rules so its export directory reflects those edits:

```sh
cargo build --manifest-path bootstrap/executor/Cargo.toml --no-default-features \
  --bin bootstrap-codegen --bin bootstrap-scan-codegen \
  --bin bootstrap-ast-codegen --bin bootstrap-pratt-codegen --bin bootstrap-unify-codegen \
  --bin bootstrap-lower-codegen
set -o pipefail
for policy in lowering pratt-lowering unify-lowering ast-lowering scan-lowering meta-lowering tests/valid/meta-driver-keys tests/valid/lowering-bindings tests/valid/lowering-numbers tests/valid/lowering-select tests/valid/lowering-left-twice tests/valid/lowering-escaped tests/valid/lowering-reverse tests/valid/lowering-module-escaped tests/valid/lowering-collections; do
  name=$(basename "$policy")
  bootstrap/executor/target/debug/bootstrap-lower-codegen "$CUESTACEAN_BOOTSTRAP_JSON/$policy.json" \
    | rustfmt --edition 2024 > "bootstrap/generated/$name.rs"
done
bootstrap/executor/target/debug/bootstrap-lower-codegen "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/lowering-numbers.json" \
  "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/meta-lowering-forced.json" \
  | rustfmt --edition 2024 > bootstrap/generated/lowering-meta-forced.rs
bootstrap/executor/target/debug/bootstrap-lower-codegen "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/meta-driver-keys.json" \
  "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/meta-driver-previous.json" \
  | rustfmt --edition 2024 > bootstrap/generated/meta-driver-previous.rs
for rules in "$CUESTACEAN_BOOTSTRAP_JSON/evaluation.json" \
  "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/evaluation-left-twice.json" \
  bootstrap/executor/tests/fixtures/codegen-escaped-rules.json; do
  name=$(basename "$rules" .json)
  bootstrap/executor/target/debug/bootstrap-codegen "$rules" \
    | rustfmt --edition 2024 > "bootstrap/generated/$name.rs"
done
bootstrap/executor/target/debug/bootstrap-codegen "$CUESTACEAN_BOOTSTRAP_JSON/evaluation.json" \
  "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/lowering-reverse.json" \
  | rustfmt --edition 2024 > bootstrap/generated/evaluation-reverse.rs
for machine in whitespace after-operand after-word-operator group-open group-close negation-prefix bound-prefix infix decimal identifier string-literal \
  tests/valid/scanner-horizontal \
  tests/valid/scanner-stalled tests/valid/scanner-advance \
  tests/valid/scanner-decimal-permissive tests/valid/scanner-reject tests/valid/scanner-state \
  tests/valid/scanner-lookahead tests/valid/scanner-char-break; do
  name=$(basename "$machine")
  bootstrap/executor/target/debug/bootstrap-scan-codegen "$CUESTACEAN_BOOTSTRAP_JSON/$machine.json" \
    | rustfmt --edition 2024 > "bootstrap/generated/$name.rs"
done
bootstrap/executor/target/debug/bootstrap-scan-codegen "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/scanner-advance.json" \
  "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/scan-lowering-unicode.json" \
  | rustfmt --edition 2024 > bootstrap/generated/scan-unicode-advance.rs
for builder in ast-program ast-atom ast-integer ast-text ast-binary ast-unary tests/valid/ast-program-v2 \
  tests/valid/ast-escaped tests/valid/ast-duplicate; do
  name=$(basename "$builder")
  bootstrap/executor/target/debug/bootstrap-ast-codegen "$CUESTACEAN_BOOTSTRAP_JSON/$builder.json" \
    | rustfmt --edition 2024 > "bootstrap/generated/$name.rs"
done
bootstrap/executor/target/debug/bootstrap-ast-codegen "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/ast-literal.json" \
  "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/ast-lowering-forced.json" \
  | rustfmt --edition 2024 > bootstrap/generated/ast-forced-integer.rs
for control in primary tests/valid/primary-no-atoms tests/valid/primary-high-minimum \
  tests/valid/primary-cross-phase tests/valid/primary-low-negation \
  pratt tests/valid/pratt-right tests/valid/pratt-stalled \
  tests/valid/pratt-invalid-state tests/valid/pratt-overflow; do
  name=$(basename "$control")
  bootstrap/executor/target/debug/bootstrap-pratt-codegen "$CUESTACEAN_BOOTSTRAP_JSON/$control.json" \
    | rustfmt --edition 2024 > "bootstrap/generated/$name.rs"
done
bootstrap/executor/target/debug/bootstrap-pratt-codegen "$CUESTACEAN_BOOTSTRAP_JSON/pratt.json" \
  "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/pratt-lowering-no-increment.json" \
  | rustfmt --edition 2024 > bootstrap/generated/pratt-no-increment.rs
for control in unify sum equal not-equal less less-equal greater greater-equal tests/valid/less-false tests/valid/equal-false tests/valid/sum-right tests/valid/unify-right tests/valid/unify-scoped; do
  name=$(basename "$control")
  bootstrap/executor/target/debug/bootstrap-unify-codegen "$CUESTACEAN_BOOTSTRAP_JSON/$control.json" \
    | rustfmt --edition 2024 > "bootstrap/generated/$name.rs"
done
bootstrap/executor/target/debug/bootstrap-unify-codegen "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/unify-equality-probe.json" \
  "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/unify-lowering-permissive.json" \
  | rustfmt --edition 2024 > bootstrap/generated/unify-permissive-bool.rs
```

## Fresh Nix builds

The pinned formatter defines the Rust artifact's byte layout. Nix builds in two
stages, clearing leftover JSON and generated Rust from each staged source copy
first. This never deletes files in the working tree. CUE JSON exports are an
independent Nix output built only from CUE sources, not committed artifacts.

1. Export fresh CUE JSON, including evaluator and Pratt compiler policies. Compile the
   native evaluator-generator VM, lowering-policy, scanner, AST, Pratt and unification compiler
   seeds with `--no-default-features` and that JSON export directory. This excludes generated frontend and
   semantic dependencies; `source` normally enables `semantics`. Seed tests need generated modules,
   so they run in the final stage rather than creating a bootstrap cycle.
2. Use the fresh JSON output and the seeds to regenerate lowering drivers,
   evaluator, scanner, AST-builder, Pratt-control and unification Rust. Compile
   all binaries with default `source` and `compiled-lowering` features enabled and
   run the complete formatting, Clippy, and test suite against fresh artifacts.
   Normal rules plus the alternate `lowering-left-twice` policy must reproduce
   the alternate-rule golden exactly; that freshly policy-produced module is
   included in the final compiled tests. Drift checks run the final compiled
   default generator and compare its evaluator output to the interpreter-seeded
   artifacts. The shared CUE backend also reproduces every existing Pratt/primary
   driver. Unification emission now uses another shared backend while preserving
   its previous goldens and scoped tuple semantics; a deliberately permissive
   compiled unifier checks alternate compiler-policy behavior. AST-builder emission
   likewise preserves all old goldens while native checked binding/use facts feed
   CUE-owned layout/clone/move rules; a compiled fixed-integer builder tests altered
   compiler policy without changing builder IR.
   `pratt-no-increment` compiles an alternate compiler policy without
   modifying parser-control IR. Normal, alternate and escaped/zero-argument lowering drivers also
   have repeat/drift checks. Reverse traversal additionally regenerates and
   compiles `evaluation-reverse.rs` while normal evaluator goldens stay unchanged. The semantics-only feature check temporarily removes
   every generated lowering driver to prove its independence.

Committed Rust modules remain reviewable golden files: the Rust generation
check still rejects drift. JSON has repeatability and schema checks, not a
committed byte baseline. CLI tests compare results, errors, and
exit statuses with the interpreter. Neither the generator nor the shared
runtime is fully self-hosted yet; generic seeds remain Rust and CUE exports still
require the upstream Go implementation.
