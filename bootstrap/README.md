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

These are experimental syntax and serialization fixtures, not an evaluator,
compiler, or settled intermediate representation. They exercise the
source-to-artifact boundary that Cuestacean will eventually need to reproduce
without upstream CUE. They do not yet describe executable implementation rules.

From the repository root, enter the pinned environment and regenerate:

```sh
nix develop ./infra/tooling
cue export bootstrap/schema.cue bootstrap/literal.cue --out json | jq -S . > bootstrap/literal.json
cue export bootstrap/schema.cue bootstrap/addition.cue --out json | jq -S . > bootstrap/addition.json
```

The byte format here is JSON formatted by the pinned `jq -S .`: recursively
sorted object keys, two-space indentation, and a trailing newline. This is a
local output convention, not a claim of general JSON canonicalization.

Run `shaka preflight` from the repository root to validate. Its Nix check runs
two separate exports for each fixture, compares them byte-for-byte, and compares
the results against the committed artifacts. It also requires every fixture in
`tests/invalid/` to fail export: unknown node kinds, missing operands, nested
non-integer values, and unexpected fields. Differing exports, stale JSON, or
accepting any invalid tree fails the check. CI uses this same gate.

Agreement here only tests these fixtures with pinned tools. It establishes
neither self-hosting nor compatibility with the rest of CUE.
