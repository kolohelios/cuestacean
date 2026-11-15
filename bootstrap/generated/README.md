# First Rust generation boundary

These modules are generated from rule JSON by the handwritten Rust
`bootstrap-codegen` binary. This lowers our three instruction primitives, not
arbitrary CUE source. The interpreter remains the default executable and a
reference for tests; generated evaluators currently run in the Rust test suite.

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

The pinned formatter defines the artifact's byte layout. Nix checks repeated
generation and rejects drift; Rust tests compile all three modules. Neither
the generator nor the shared runtime is self-hosted yet.
