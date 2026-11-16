# Bootstrap expression fixtures

`schema.cue` defines a recursive expression tree with sixteen node kinds:

- `integer`: an integer `value`.
- `intType`: the abstract `int` constraint, with no payload.
- `true` / `false` / `boolType`: payload-free boolean values and type constraint.
- `lowerBound`: an inclusive lower threshold in integer `value`.
- `upperBound`: an inclusive upper threshold in integer `value`.
- `add`: ordered `left` and `right` expression children.
- `multiply`: ordered `left` and `right` expression children.
- `subtract`: ordered children, subtracting right from left.
- `negate`: an `operand` expression, negated with checked i64 arithmetic.
- `divide` / `modulo`: Euclidean quotient and nonnegative remainder.
- `quotient` / `remainder`: truncating quotient and dividend-signed remainder.
- `unify`: ordered children whose integer/bounds or boolean domains must agree.

`#Program` wraps an expression with `formatVersion: 1`. Node definitions reject
unexpected fields. Missing fields with a single allowed value (such as the
format version) can be supplied by CUE unification; export still requires a
concrete, complete expression.

`literal.cue` instantiates the original integer literal. `addition.cue` describes
`20 + (10 + 12)`, preserving its nested tree rather than evaluating it to `42`.
`multiplication.cue` describes `1 + 2 * 3` as an addition with a multiplication
child. `subtraction.cue` describes `50 - 8`, producing `42`.
`division.cue` combines all four integer-division shapes into a result of `42`.
`negation.cue` describes `-(1 + 2)`, preserving its unary node.
`unification.cue` describes `(1 + 2) & 3`. The pinned upstream Go CUE
tool exports these into matching JSON files in a Nix build output, not version
control. `int-constraint.cue` describes
`int & 3`, introducing an abstract intermediate value. `bounds.cue` describes
`int & >=0 & <=10 & 3`, retaining abstract range intermediates.
`booleans.cue` describes `bool & true`, retaining the abstract type leaf.
Definitions such as `#Expression` do not appear in the exported artifacts.

## Evaluation rules as data

`evaluation.cue` instantiates `#EvaluationRules` and exports to `evaluation.json`.
Rules are selected by the input expression's `kind`; each describes how to
produce its `result`:

- `intType`: `{op: "int"}` constructs abstract `int`.
- `boolType`: `{op: "bool"}` constructs abstract `bool`; `true` / `false` use
  `{op: "boolean", value: true/false}` constants supplied by rule data.
- `integer`: `{op: "field", field: "value"}` reads the current node's integer.
- `lowerBound` / `upperBound`: execute their `value` instruction to obtain a
  concrete threshold, then construct an inclusive one-sided bound.
- `add`: `sum` combines two integer results, obtained by `evaluate` instructions
  targeting the current node's `left` and `right` children.
- `multiply`: `product` combines the evaluated children using checked integer
  multiplication.
- `subtract`: `difference` subtracts the evaluated right child from the left
  using checked i64 subtraction, not addition of a negated operand.
- `divide` / `modulo` and `quotient` / `remainder`: evaluate both children
  left-first, require concrete integers/nonzero divisor, and apply the chosen
  integer-division convention with checked quotient overflow.
- `negate`: evaluates its `operand` and applies checked integer negation.
- `unify`: intersect bounds, refine `int`/`bool`, and check concrete candidates.
  Different scalar domains conflict; concrete booleans agree only when equal.
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
these sixteen rule shapes and their
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
generator from CUE. The value scaffold and generators remain handwritten. The separate
[unification control program](unify.md) now lowers identity, intersection and
conflict decisions from CUE into the shared runtime's production unification.
[Instruction-to-Rust lowering](lowering.md) now interprets CUE-owned recursive
expression programs instead of a native opcode-emission match. Its generic VM
and outer module assembly remain native; normal output matches existing goldens.
Compiler seeds consume freshly exported policy JSON, not committed artifacts.

The first [executable scanner DSL](scanner.md) separately expresses the
whitespace, decimal, and ASCII identifier scanning in CUE. Its compiler emits
Rust control flow rather than calling native scanning algorithms. Decimal
missing-digit/leading-zero rejection and the conservative operand-newline rule
are also CUE-defined, as are parenthesis recognition/consumption and missing-close
diagnostics. Bound/infix symbol recognition also uses the scanner DSL. The native
parser only looks up complete spans. Checked i64 conversion,
grammar validation, keyword lookup, native flow analysis and cursor/pattern kernels
remain native. The [scanner compiler backend](scan-lowering.md) now uses shared
CUE source/branch layout.

[AST builders](ast.md) separately define program wrappers, atoms, integer/bound
leaves, and binary records as CUE construction expressions. Generated typed
functions replace the parser's native JSON-map assembly while keeping dynamic
grammar field names. Their [compiler backend](ast-lowering.md) now uses shared
CUE layout/ownership rules; checked binding/use-count analysis and the reference
construction interpreter remain native.

The [Pratt control program](pratt.md) now owns expression repetition,
binding-power comparison, associativity adjustment and recursive-right sequencing.
The separate [primary program](primary.md) now owns primary dispatch and grouping
control using the same DSL/compiler. Both call checked native frame operations
for scanner/builder calls, grammar lookup and operand handling; this is not yet
a fully self-hosted parser/compiler.

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
