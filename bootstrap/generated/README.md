# First Rust generation boundary

The evaluator modules are generated from rule JSON by the handwritten Rust
`bootstrap-codegen` binary. This lowers our eight instruction primitives, not
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
its loop and newline branch come from `whitespace.cue`. `decimal.rs` uses the
same DSL for optional-minus/digit scanning and lexical rejection; `identifier.rs`
supplies ASCII identifier start/continuation scanning. Checked i64 conversion,
grammar validation, keyword lookup, and AST construction remain native. The `scanner-horizontal`,
`scanner-stalled`, and `scanner-advance` modules are test-only algorithms that
exercise alternate branches, non-progress, and native cursor guards.
`scanner-decimal-permissive.rs` demonstrates alternate lexical policy;
`scanner-reject.rs` checks escaped reasons and rejection short-circuiting.

Generated evaluator code contains node-kind dispatch and lowered rule expressions. It
shares the handwritten `bootstrap_executor::runtime` primitives for JSON
validation, field access, integer/type values, inclusive bounds, unification,
and checked integer arithmetic. Intermediate values may be abstract `int` or
bounds; the generated public
entry point requires a concrete i64 only at the output boundary. Tests compare values
and errors against the interpreter and known expectations. Sharing primitives
means this is not an independent test of their correctness or of CUE conformance.

Regenerate from the repository root in `nix develop ./infra/tooling`. Re-enter
the shell after editing CUE rules so its export directory reflects those edits:

```sh
cargo build --manifest-path bootstrap/executor/Cargo.toml --no-default-features \
  --bin bootstrap-codegen --bin bootstrap-scan-codegen
set -o pipefail
for rules in "$CUESTACEAN_BOOTSTRAP_JSON/evaluation.json" \
  "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/evaluation-left-twice.json" \
  bootstrap/executor/tests/fixtures/codegen-escaped-rules.json; do
  name=$(basename "$rules" .json)
  bootstrap/executor/target/debug/bootstrap-codegen "$rules" \
    | rustfmt --edition 2024 > "bootstrap/generated/$name.rs"
done
for machine in whitespace decimal identifier tests/valid/scanner-horizontal \
  tests/valid/scanner-stalled tests/valid/scanner-advance \
  tests/valid/scanner-decimal-permissive tests/valid/scanner-reject; do
  name=$(basename "$machine")
  bootstrap/executor/target/debug/bootstrap-scan-codegen "$CUESTACEAN_BOOTSTRAP_JSON/$machine.json" \
    | rustfmt --edition 2024 > "bootstrap/generated/$name.rs"
done
```

## Fresh Nix builds

The pinned formatter defines the Rust artifact's byte layout. Nix builds in two
stages, clearing leftover JSON and generated Rust from each staged source copy
first. This never deletes files in the working tree. CUE JSON exports are an
independent Nix output built only from CUE sources, not committed artifacts.

1. Compile only the handwritten `bootstrap-codegen` and `bootstrap-scan-codegen`
   seeds with `--no-default-features`. This excludes the `source` feature's
   parser/generated-scanner dependency. Seed tests need generated modules,
   so they run in the final stage rather than creating a bootstrap cycle.
2. Use the fresh JSON output and the seeds to regenerate evaluator and scanner
   Rust. Compile all binaries with the default `source` feature enabled and
   run the complete formatting, Clippy, and test suite against fresh artifacts.

Committed Rust modules remain reviewable golden files: the Rust generation
check still rejects drift. JSON has repeatability and schema checks, not a
committed byte baseline. CLI tests compare results, errors, and
exit statuses with the interpreter. Neither the generator nor the shared
runtime is self-hosted yet; the seeds are handwritten Rust and CUE exports still
require the upstream Go implementation.
