# First Rust generation boundary

The evaluator modules are generated from rule JSON by the handwritten Rust
`bootstrap-codegen` binary. This lowers our fourteen instruction primitives, not
arbitrary CUE source. The interpreter remains the default executable and a
reference for tests. The normal generated module also runs as a separate CLI:

```sh
nix build ./infra/tooling#exports --out-link result-exports
nix run ./infra/tooling#generated -- result-exports/addition.json
```

Run that command from the repository root; it prints `42`. The rules are baked
in, so this CLI accepts only a program path. The alternate and escaping modules
remain test-only.

- `evaluation.rs` comes from the normal CUE-exported rules.
- `evaluation-left-twice.rs` comes from the alternate CUE-exported rules.
- `codegen-escaped-rules.rs` comes from an adversarial JSON fixture that tests
  escaping names containing quotes, backslashes, a newline, and Unicode. This
  fixture exercises the emitter, not the narrower CUE schema.

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
`scanner-decimal-permissive.rs` demonstrates alternate lexical policy;
`scanner-reject.rs` checks escaped reasons and rejection short-circuiting;
`scanner-state.rs` tests flag reads, stateful loops, and byte-presence checks;
`scanner-lookahead.rs` checks maximum lookahead and EOF handling.

The `ast-program`, `ast-atom`, `ast-integer`, `ast-binary`, and `ast-unary` modules come from
the separate [AST construction DSL](../ast.md). They replace native parser map
assembly. `ast-program-v2`, `ast-escaped`, and `ast-duplicate` are test-only
programs for changed policy, escaping, and repeated owned arguments.

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
test-only policies. Intermediate values may be abstract `int` or
bounds; the generated public
entry point requires a concrete i64 only at the output boundary. Tests compare values
and errors against the interpreter and known expectations. Sharing primitives
means this is not an independent test of their correctness or of CUE conformance.

Regenerate from the repository root in `nix develop ./infra/tooling`. Re-enter
the shell after editing CUE rules so its export directory reflects those edits:

```sh
cargo build --manifest-path bootstrap/executor/Cargo.toml --no-default-features \
  --bin bootstrap-codegen --bin bootstrap-scan-codegen \
  --bin bootstrap-ast-codegen --bin bootstrap-pratt-codegen --bin bootstrap-unify-codegen
set -o pipefail
for rules in "$CUESTACEAN_BOOTSTRAP_JSON/evaluation.json" \
  "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/evaluation-left-twice.json" \
  bootstrap/executor/tests/fixtures/codegen-escaped-rules.json; do
  name=$(basename "$rules" .json)
  bootstrap/executor/target/debug/bootstrap-codegen "$rules" \
    | rustfmt --edition 2024 > "bootstrap/generated/$name.rs"
done
for machine in whitespace after-operand after-word-operator group-open group-close negation-prefix bound-prefix infix decimal identifier \
  tests/valid/scanner-horizontal \
  tests/valid/scanner-stalled tests/valid/scanner-advance \
  tests/valid/scanner-decimal-permissive tests/valid/scanner-reject tests/valid/scanner-state \
  tests/valid/scanner-lookahead tests/valid/scanner-char-break; do
  name=$(basename "$machine")
  bootstrap/executor/target/debug/bootstrap-scan-codegen "$CUESTACEAN_BOOTSTRAP_JSON/$machine.json" \
    | rustfmt --edition 2024 > "bootstrap/generated/$name.rs"
done
for builder in ast-program ast-atom ast-integer ast-binary ast-unary tests/valid/ast-program-v2 \
  tests/valid/ast-escaped tests/valid/ast-duplicate; do
  name=$(basename "$builder")
  bootstrap/executor/target/debug/bootstrap-ast-codegen "$CUESTACEAN_BOOTSTRAP_JSON/$builder.json" \
    | rustfmt --edition 2024 > "bootstrap/generated/$name.rs"
done
for control in primary tests/valid/primary-no-atoms tests/valid/primary-high-minimum \
  tests/valid/primary-cross-phase tests/valid/primary-low-negation \
  pratt tests/valid/pratt-right tests/valid/pratt-stalled \
  tests/valid/pratt-invalid-state tests/valid/pratt-overflow; do
  name=$(basename "$control")
  bootstrap/executor/target/debug/bootstrap-pratt-codegen "$CUESTACEAN_BOOTSTRAP_JSON/$control.json" \
    | rustfmt --edition 2024 > "bootstrap/generated/$name.rs"
done
for control in unify tests/valid/unify-right tests/valid/unify-scoped; do
  name=$(basename "$control")
  bootstrap/executor/target/debug/bootstrap-unify-codegen "$CUESTACEAN_BOOTSTRAP_JSON/$control.json" \
    | rustfmt --edition 2024 > "bootstrap/generated/$name.rs"
done
```

## Fresh Nix builds

The pinned formatter defines the Rust artifact's byte layout. Nix builds in two
stages, clearing leftover JSON and generated Rust from each staged source copy
first. This never deletes files in the working tree. CUE JSON exports are an
independent Nix output built only from CUE sources, not committed artifacts.

1. Compile the handwritten evaluator, scanner, AST, Pratt and unification compiler
   seeds with `--no-default-features`. This excludes generated frontend and
   semantic dependencies; `source` normally enables `semantics`. Seed tests need generated modules,
   so they run in the final stage rather than creating a bootstrap cycle.
2. Use the fresh JSON output and the seeds to regenerate evaluator, scanner,
   AST-builder, Pratt-control and unification Rust. Compile all binaries with the default `source` feature enabled and
   run the complete formatting, Clippy, and test suite against fresh artifacts.

Committed Rust modules remain reviewable golden files: the Rust generation
check still rejects drift. JSON has repeatability and schema checks, not a
committed byte baseline. CLI tests compare results, errors, and
exit statuses with the interpreter. Neither the generator nor the shared
runtime is self-hosted yet; the seeds are handwritten Rust and CUE exports still
require the upstream Go implementation.
