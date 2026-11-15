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

The generated evaluator is a separate CLI:

```sh
nix run ./infra/tooling#generated -- bootstrap/addition.json
```

It prints `42` and takes only a program path. The normal `evaluation.json` rules
are compiled into it through `../generated/evaluation.rs`; it does not read or
interpret a rule file at runtime. Nix regenerates the JSON and Rust in its build
sandbox before compilation. After changing CUE rules, regenerate the committed
artifacts too so the drift checks pass. The default Nix entry point remains the
interpreter.

Both evaluators use the same output and evaluation-error conventions. Their
usage messages differ because their argument lists differ.

## Instruction meanings

The executor looks up a node's string `kind` in the supplied rule map, then
interprets that rule's result instruction against the current node:

- `int` constructs an abstract integer-type value.
- `field` reads the named field as a concrete integer value.
- `evaluate` selects the named child and recursively dispatches its rule.
- `sum` executes its left instruction, then its right instruction, against the
  same current node and adds their integer results.
- `product` executes its left and right instructions in the same order and
  multiplies their integer results using checked arithmetic.
- `unify` refines the abstract `int` value with either operand. Equal concrete
  integers agree; differing integers report `conflicting integer values`.

The native value scaffold has two variants: a concrete i64 and abstract `int`.
Intermediate abstract values are valid: `int & int & 3` produces `3`. Only the
outer scalar-output boundary requires concreteness; bare `int` reports
`non-concrete int value`, not a conflict. The value representation is still
handwritten Rust and is not a general CUE value lattice.

Rust implements these primitives, not special cases for the expression kinds.
The CUE-exported `../tests/valid/evaluation-left-twice.json` changes addition to
evaluate the left child twice. With the unchanged addition tree and executable,
that rule file yields `40`, demonstrating that rule data controls behavior.

## Rust generation

The package also includes `bootstrap-codegen <rules.json>`. It emits a Rust
module to stdout and rejects malformed or unsupported rule documents before
emitting source. Names are escaped as Rust string literals, and dispatch order
is deterministic regardless of input object-key order.

The [generated modules](../generated/README.md) use the shared native runtime
rather than the instruction interpreter. Tests compile them, compare values and
errors against the interpreter, and verify that alternate rule data generates
alternate behavior. `bootstrap-generated` runs the normal generated module;
`bootstrap-executor` remains available for runtime rule changes and comparison.

## Source frontend

`bootstrap-source <expression.cue>` parses the [documented source subset](../source/README.md)
using compiled-in CUE grammar data, then invokes the generated evaluator. From
the repository root, `nix run ./infra/tooling#source -- bootstrap/source/valid/nested.cue`
prints `42`. It does not invoke Go at runtime; the lexer and Pratt algorithm are
still handwritten scaffolding. Existing JSON interfaces remain unchanged.

## Deliberate limits

- Unification supports concrete i64 values and abstract `int` only. Conflicts
  are errors, not first-class bottom values. Other types, structs, bounds,
  disjunctions, defaults, references, and full CUE unification remain unsupported.
  Runtime diagnostics are not upstream CUE diagnostics.
- Arithmetic requires concrete operands: `(int & 3) + 1` works, but `int + 1`
  cannot produce a concrete result. There is no backward symbolic propagation.
- Both documents require `formatVersion: 1`.
- Concrete values and arithmetic results are signed 64-bit integers. Out-of-range
  values, non-integer JSON values, overflow, and underflow are errors. No
  wrapping, saturation, or floating-point arithmetic. This is narrower than
  CUE's integer domain and is only a bootstrap limitation. Multiplying `i64::MIN`
  by `-1` is also an overflow error; a later multiplication by zero does not erase
  an earlier intermediate overflow.
- Unknown operations, unknown instruction fields, missing accessed fields, and
  missing rules are errors. The executor does not validate the complete AST
  against the CUE schema; it checks only fields that the chosen rule accesses.
- Inputs are ordinary JSON, subject to the JSON parser's nesting limit. This
  small executor is not a production-hardened runtime or a general rule engine.

## Checks

`shaka preflight` builds the package through `nix flake check`. A handwritten
code-generator seed enables a [fresh two-stage build](../generated/README.md),
then the final package runs Rust formatting, Clippy, and all tests in the
sandbox. Tests use the real exported
fixtures, the changed-rule export, integer boundary/error cases, all four
CLIs, the source parser, and the compiled generated evaluators. The interpreter and generated CLIs
must agree byte-for-byte on stdout and stderr, and on exit status, for both
exported expressions, the complete program-case corpus, malformed JSON, and
missing program files. Argument errors are checked against each CLI's usage.
A separate Nix check verifies repeated Rust generation and committed-source
drift. The Rust toolchain is pinned through the tooling flake's Nixpkgs lock;
Rust dependencies are pinned by `Cargo.lock`. No additional workflow is needed.

For a quick local iteration inside `nix develop ./infra/tooling`:

```sh
cargo test --manifest-path bootstrap/executor/Cargo.toml
```
