# Bootstrap expression fixtures

`schema.cue` defines a recursive expression tree with four node kinds:

- `integer`: an integer `value`.
- `add`: ordered `left` and `right` expression children.
- `multiply`: ordered `left` and `right` expression children.
- `unify`: ordered children whose concrete integer results must agree.

`#Program` wraps an expression with `formatVersion: 1`. Node definitions reject
unexpected fields. Missing fields with a single allowed value (such as the
format version) can be supplied by CUE unification; export still requires a
concrete, complete expression.

`literal.cue` instantiates the original integer literal. `addition.cue` describes
`20 + (10 + 12)`, preserving its nested tree rather than evaluating it to `42`.
`multiplication.cue` describes `1 + 2 * 3` as an addition with a multiplication
child. `unification.cue` describes `(1 + 2) & 3`. The pinned upstream Go CUE
tool exports these into matching JSON files.
Definitions such as `#Expression` do not appear in the exported artifacts.

## Evaluation rules as data

`evaluation.cue` instantiates `#EvaluationRules` and exports to `evaluation.json`.
Rules are selected by the input expression's `kind`; each describes how to
produce its `result`:

- `integer`: `{op: "field", field: "value"}` reads the current node's integer.
- `add`: `sum` combines two integer results, obtained by `evaluate` instructions
  targeting the current node's `left` and `right` children.
- `multiply`: `product` combines the evaluated children using checked integer
  multiplication.
- `unify`: return the integer if the evaluated children agree; otherwise fail
  with a concrete-integer conflict. This is not a general constraint solver.

Child evaluation means selecting and applying a rule recursively to that child,
not reading the child tree as an integer. The two operand instructions in a sum
both refer to the current input node, not to each other's results.

Upstream CUE validates and exports these instructions as data; export does not
evaluate the addition tree. The [bootstrap Rust executor](executor/README.md)
interprets the exported rules in a separate step: the original fixtures produce
`42`, the mixed multiplication fixture produces `7`, and unification produces
`3`. The schema constrains
these four rule shapes and their
field names, not a general-purpose rule language. Validation alone does not
establish correct evaluation behavior.

`tests/valid/evaluation-left-twice.cue` deliberately changes addition to evaluate
its left child twice. It is valid rule data, but has different semantics: the
executor produces `40` for the same addition tree. This is a regression check
that changing CUE rule data changes execution, rather than Rust hardcoding the
expression behavior.

## First Rust generation boundary

The handwritten executor now also includes a rule-to-Rust generator. Its
[generated modules](generated/README.md) compile in the test suite and agree
with the interpreter for fixture results and error cases. The normal generated
module also runs through `nix run ./infra/tooling#generated -- <program.json>`,
with its rules compiled in. CLI tests compare it against the interpreter.
The Nix build discards staged intermediate artifacts and regenerates JSON and
Rust before compiling, using a handwritten generator seed. This lowers the
exported rule DSL to Rust; it does not implement CUE evaluation or rebuild the
generator from CUE. The shared native runtime and the generator remain
handwritten.

The syntax and rules remain experimental, not a settled intermediate
representation. They exercise the source-to-artifact boundary that Cuestacean
will eventually need to reproduce without upstream CUE.

From the repository root, enter the pinned environment and regenerate:

```sh
nix develop ./infra/tooling
cue export bootstrap/schema.cue bootstrap/literal.cue --out json | jq -S . > bootstrap/literal.json
cue export bootstrap/schema.cue bootstrap/addition.cue --out json | jq -S . > bootstrap/addition.json
cue export bootstrap/schema.cue bootstrap/multiplication.cue --out json | jq -S . > bootstrap/multiplication.json
cue export bootstrap/schema.cue bootstrap/unification.cue --out json | jq -S . > bootstrap/unification.json
cue export bootstrap/schema.cue bootstrap/evaluation.cue --out json | jq -S . > bootstrap/evaluation.json
cue export bootstrap/schema.cue bootstrap/tests/valid/evaluation-left-twice.cue --out json | jq -S . > bootstrap/tests/valid/evaluation-left-twice.json
```

The byte format here is JSON formatted by the pinned `jq -S .`: recursively
sorted object keys, two-space indentation, and a trailing newline. This is a
local output convention, not a claim of general JSON canonicalization.

Run `shaka preflight` from the repository root to validate. Its export check runs
two separate exports for each fixture, compares them byte-for-byte, and compares
the results against the committed artifacts. It also requires every fixture in
`tests/invalid/` to fail export. Expression cases cover unknown node kinds,
missing operands, nested non-integer values, and unexpected fields. Rule cases
cover unknown operations and child fields, reading a child without evaluating
it, and missing operand instructions. Differing exports, stale JSON, or
accepting any invalid fixture fails the check. A second Nix check builds and
tests the Rust executor and generated evaluators, including formatting and
Clippy. A third check verifies repeated Rust generation and rejects drift in
the committed modules. CI uses this same gate.

Agreement here only tests these fixtures with pinned tools. It establishes
neither self-hosting nor compatibility with the rest of CUE.
