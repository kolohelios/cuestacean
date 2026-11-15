# Bootstrap rule executor

Disposable handwritten Rust machinery that gives the exported CUE rules their
first executable meaning. This is not a CUE parser, compiler, or constraint
solver. The long-term goal remains replacing this scaffolding with generated
code from a CUE-hosted implementation DSL.

From the repository root:

```sh
nix run ./infra/tooling -- bootstrap/literal.json bootstrap/evaluation.json
nix run ./infra/tooling -- bootstrap/addition.json bootstrap/evaluation.json
```

Both print `42`. The CLI takes exactly two paths: program JSON, then rule JSON.
Success prints a decimal integer and newline; errors go to stderr with exit
status 2 and no result on stdout.

## Instruction meanings

The executor looks up a node's string `kind` in the supplied rule map, then
interprets that rule's result instruction against the current node:

- `field` reads the named field as an integer.
- `evaluate` selects the named child and recursively dispatches its rule.
- `sum` executes its left instruction, then its right instruction, against the
  same current node and adds their integer results.

Rust implements these primitives, not a special case for `integer` or `add`.
The CUE-exported `../tests/valid/evaluation-left-twice.json` changes addition to
evaluate the left child twice. With the unchanged addition tree and executable,
that rule file yields `40`, demonstrating that rule data controls behavior.

## Deliberate limits

- Both documents require `formatVersion: 1`.
- Values and intermediate results are signed 64-bit integers. Out-of-range
  values, non-integer JSON values, overflow, and underflow are errors. No
  wrapping, saturation, or floating-point arithmetic. This is narrower than
  CUE's integer domain and is only a bootstrap limitation.
- Unknown operations, unknown instruction fields, missing accessed fields, and
  missing rules are errors. The executor does not validate the complete AST
  against the CUE schema; it checks only fields that the chosen rule accesses.
- Inputs are ordinary JSON, subject to the JSON parser's nesting limit. This
  small executor is not a production-hardened runtime or a general rule engine.

## Checks

`shaka preflight` builds the package through `nix flake check`, which runs Rust
formatting, Clippy, and the tests in the sandbox. Tests use the real exported
fixtures, the changed-rule export, integer boundary/error cases, and the CLI.
The Rust toolchain is pinned through the tooling flake's Nixpkgs lock; Rust
dependencies are pinned by `Cargo.lock`. No additional workflow is needed.

For a quick local iteration inside `nix develop ./infra/tooling`:

```sh
cargo test --manifest-path bootstrap/executor/Cargo.toml
```
