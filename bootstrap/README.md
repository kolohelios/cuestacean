# Bootstrap expression fixtures

`schema.cue` defines a recursive expression tree with seven node kinds:

- `integer`: an integer `value`.
- `intType`: the abstract `int` constraint, with no payload.
- `lowerBound`: an inclusive lower threshold in integer `value`.
- `upperBound`: an inclusive upper threshold in integer `value`.
- `add`: ordered `left` and `right` expression children.
- `multiply`: ordered `left` and `right` expression children.
- `unify`: ordered children whose integers, `int`, or bounds must agree.

`#Program` wraps an expression with `formatVersion: 1`. Node definitions reject
unexpected fields. Missing fields with a single allowed value (such as the
format version) can be supplied by CUE unification; export still requires a
concrete, complete expression.

`literal.cue` instantiates the original integer literal. `addition.cue` describes
`20 + (10 + 12)`, preserving its nested tree rather than evaluating it to `42`.
`multiplication.cue` describes `1 + 2 * 3` as an addition with a multiplication
child. `unification.cue` describes `(1 + 2) & 3`. The pinned upstream Go CUE
tool exports these into matching JSON files in a Nix build output, not version
control. `int-constraint.cue` describes
`int & 3`, introducing an abstract intermediate value. `bounds.cue` describes
`int & >=0 & <=10 & 3`, retaining abstract range intermediates.
Definitions such as `#Expression` do not appear in the exported artifacts.

## Evaluation rules as data

`evaluation.cue` instantiates `#EvaluationRules` and exports to `evaluation.json`.
Rules are selected by the input expression's `kind`; each describes how to
produce its `result`:

- `intType`: `{op: "int"}` constructs abstract `int`.
- `integer`: `{op: "field", field: "value"}` reads the current node's integer.
- `lowerBound` / `upperBound`: execute their `value` instruction to obtain a
  concrete threshold, then construct an inclusive one-sided bound.
- `add`: `sum` combines two integer results, obtained by `evaluate` instructions
  targeting the current node's `left` and `right` children.
- `multiply`: `product` combines the evaluated children using checked integer
  multiplication.
- `unify`: intersect bounds, refine `int`, and check concrete candidates.
  Conflicts fail; abstract results remain incomplete, not conflicting.
  Even equal endpoints stay abstract until given a concrete candidate.
  This is not a general constraint solver.

Child evaluation means selecting and applying a rule recursively to that child,
not reading the child tree as an integer. The two operand instructions in a sum
both refer to the current input node, not to each other's results.

Upstream CUE validates and exports these instructions as data; export does not
evaluate the addition tree. The [bootstrap Rust executor](executor/README.md)
interprets the exported rules in a separate step: the original fixtures produce
`42`, the mixed multiplication fixture produces `7`, and unification produces
`3`. The schema constrains
these seven rule shapes and their
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
The Nix build exports JSON from CUE into an immutable output and regenerates
staged Rust before compiling, using handwritten generator seeds. This lowers the
exported rule DSL to Rust; it does not implement CUE evaluation or rebuild the
generator from CUE. The shared native runtime and the generator remain
handwritten.

The first [executable scanner DSL](scanner.md) separately expresses the
whitespace, decimal, and ASCII identifier scanning in CUE. Its compiler emits
Rust control flow rather than calling native scanning algorithms. Decimal
missing-digit/leading-zero rejection and the conservative operand-newline rule
are also CUE-defined, as are parenthesis recognition/consumption and missing-close
diagnostics. Expression recursion remains native. Checked i64 conversion,
grammar validation, keyword lookup, AST construction, other parsing, and the
scanner compiler/runtime
remain handwritten.

The syntax and rules remain experimental, not a settled intermediate
representation. They exercise the source-to-artifact boundary that Cuestacean
will eventually need to reproduce without upstream CUE.

From the repository root, build and inspect the exports:

```sh
nix build ./infra/tooling#exports --out-link result-exports
nix run ./infra/tooling -- result-exports/addition.json result-exports/evaluation.json
```

The devshell supplies `CUESTACEAN_BOOTSTRAP_JSON`, pointing to this same Nix
output. Cargo tests and the frontend's embedded grammar use that directory,
not working-tree JSON. No manual regeneration is required on a clean checkout.
Re-enter `nix develop ./infra/tooling` after editing CUE specifications to refresh
the shell's export directory. Generated JSON paths are ignored; handwritten
JSON cases under `executor/tests/fixtures/` remain version-controlled.

The byte format here is JSON formatted by the pinned `jq -S .`: recursively
sorted object keys, two-space indentation, and a trailing newline. This is a
local output convention, not a claim of general JSON canonicalization.

Run `shaka preflight` from the repository root to validate. Its export check runs
two separate exports for each fixture and compares them byte-for-byte. There
is no committed JSON byte baseline; semantic expectations, interpreter/generated
parity, and upstream comparisons check behavior. It also requires every fixture in
`tests/invalid/` to fail export. Expression cases cover unknown node kinds,
missing operands, nested non-integer values, and unexpected fields. Rule cases
cover unknown operations and child fields, reading a child without evaluating
it, and missing operand instructions. Differing repeated exports or accepting
any invalid fixture fails the check. A second Nix check builds and
tests the Rust executor and generated evaluators, including formatting and
Clippy. A third check verifies repeated Rust generation and rejects drift in
the committed modules. CI uses this same gate.

Agreement here only tests these fixtures with pinned tools. It establishes
neither self-hosting nor compatibility with the rest of CUE.
