# First Rust generation boundary

These modules are generated from rule JSON by the handwritten Rust
`bootstrap-codegen` binary. This lowers our four instruction primitives, not
arbitrary CUE source. The interpreter remains the default executable and a
reference for tests. The normal generated module also runs as a separate CLI:

```sh
nix run ./infra/tooling#generated -- bootstrap/addition.json
```

Run that command from the repository root; it prints `42`. The rules are baked
in, so this CLI accepts only a program path. The alternate and escaping modules
remain test-only.

- `evaluation.rs` comes from the normal CUE-exported rules.
- `evaluation-left-twice.rs` comes from the alternate CUE-exported rules.
- `codegen-escaped-rules.rs` comes from an adversarial JSON fixture that tests
  escaping names containing quotes, backslashes, a newline, and Unicode. This
  fixture exercises the emitter, not the narrower CUE schema.

Generated code contains node-kind dispatch and lowered rule expressions. It
shares the handwritten `bootstrap_executor::runtime` primitives for JSON
validation, field access, and checked integer arithmetic. Tests compare values
and errors against the interpreter and known expectations. Sharing primitives
means this is not an independent test of their correctness or of CUE conformance.

Regenerate from the repository root in `nix develop ./infra/tooling`:

```sh
cargo build --manifest-path bootstrap/executor/Cargo.toml --bin bootstrap-codegen
set -o pipefail
for rules in evaluation.json tests/valid/evaluation-left-twice.json executor/tests/fixtures/codegen-escaped-rules.json; do
  name=$(basename "$rules" .json)
  bootstrap/executor/target/debug/bootstrap-codegen "bootstrap/$rules" \
    | rustfmt --edition 2024 > "bootstrap/generated/$name.rs"
done
```

## Fresh Nix builds

The pinned formatter defines the artifact's byte layout. Nix now builds in two
stages, clearing the JSON exports and generated Rust from each staged source
copy first. This never deletes files in the working tree.

1. Compile only the handwritten `bootstrap-codegen` seed. Its tests need
   generated modules, so they run in the final stage rather than creating a
   bootstrap dependency cycle.
2. Use upstream CUE to regenerate the expression and rule JSON, then the seed
   to regenerate Rust. Compile all binaries and run the complete formatting,
   Clippy, and test suite against these fresh artifacts.

Committed artifacts remain reviewable golden files: separate export and Rust
generation checks still reject drift. CLI tests compare results, errors, and
exit statuses with the interpreter. Neither the generator nor the shared
runtime is self-hosted yet; the seed is handwritten Rust and CUE exports still
require the upstream Go implementation.
