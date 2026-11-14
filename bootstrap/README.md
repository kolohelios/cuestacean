# Bootstrap expression fixtures

`schema.cue` defines a recursive expression tree with two node kinds:

- `integer`: an integer `value`.
- `add`: ordered `left` and `right` expression children.

`#Program` wraps an expression with `formatVersion: 1`. Node definitions reject
unexpected fields. Missing fields with a single allowed value (such as the
format version) can be supplied by CUE unification; export still requires a
concrete, complete expression.

`literal.cue` instantiates the original integer literal. `addition.cue` describes
`20 + (10 + 12)`, preserving its nested tree rather than evaluating it to `42`.
The pinned upstream Go CUE tool exports these into the matching JSON files.
Definitions such as `#Expression` do not appear in the exported artifacts.

## Evaluation rules as data

`evaluation.cue` instantiates `#EvaluationRules` and exports to `evaluation.json`.
Rules are selected by the input expression's `kind`; each describes how to
produce its `result`:

- `integer`: `{op: "field", field: "value"}` reads the current node's integer.
- `add`: `sum` combines two integer results, obtained by `evaluate` instructions
  targeting the current node's `left` and `right` children.

Child evaluation means selecting and applying a rule recursively to that child,
not reading the child tree as an integer. The two operand instructions in a sum
both refer to the current input node, not to each other's results.

These are proposed instructions for a future executor. Upstream CUE validates
and exports them as data; nothing executes them yet or computes `42` from the
addition tree. The schema constrains these two rule shapes and their field
names, not a general-purpose rule language. Validation alone does not establish
correct evaluation behavior.

The syntax and rules remain experimental, not a settled intermediate
representation. They exercise the source-to-artifact boundary that Cuestacean
will eventually need to reproduce without upstream CUE.

From the repository root, enter the pinned environment and regenerate:

```sh
nix develop ./infra/tooling
cue export bootstrap/schema.cue bootstrap/literal.cue --out json | jq -S . > bootstrap/literal.json
cue export bootstrap/schema.cue bootstrap/addition.cue --out json | jq -S . > bootstrap/addition.json
cue export bootstrap/schema.cue bootstrap/evaluation.cue --out json | jq -S . > bootstrap/evaluation.json
```

The byte format here is JSON formatted by the pinned `jq -S .`: recursively
sorted object keys, two-space indentation, and a trailing newline. This is a
local output convention, not a claim of general JSON canonicalization.

Run `shaka preflight` from the repository root to validate. Its Nix check runs
two separate exports for each fixture, compares them byte-for-byte, and compares
the results against the committed artifacts. It also requires every fixture in
`tests/invalid/` to fail export. Expression cases cover unknown node kinds,
missing operands, nested non-integer values, and unexpected fields. Rule cases
cover unknown operations and child fields, reading a child without evaluating
it, and missing operand instructions. Differing exports, stale JSON, or
accepting any invalid fixture fails the check. CI uses this same gate.

Agreement here only tests these fixtures with pinned tools. It establishes
neither self-hosting nor compatibility with the rest of CUE.
