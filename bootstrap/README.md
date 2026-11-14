# First bootstrap boundary

`literal.cue` defines a constrained integer-literal shape and instantiates it.
The pinned upstream Go CUE tool evaluates it into `literal.json`. Definitions
such as `#IntegerLiteral` do not appear in the exported artifact.

This is a serialization fixture, not a compiler or a settled intermediate
representation. It establishes the first source-to-artifact boundary that
Cuestacean will eventually need to reproduce without upstream CUE.

From the repository root, enter the pinned environment and regenerate:

```sh
nix develop ./infra/tooling
cue export bootstrap/literal.cue --out json | jq -S . > bootstrap/literal.json
```

The byte format here is JSON formatted by the pinned `jq -S .`: recursively
sorted object keys, two-space indentation, and a trailing newline. This is a
local output convention, not a claim of general JSON canonicalization.

Run `shaka preflight` from the repository root to validate. Its Nix check runs
two separate exports, compares them byte-for-byte, and compares the result
against the committed artifact. Invalid CUE, differing exports, or a stale or
hand-edited JSON artifact fails the check. CI uses this same gate.

Agreement here only tests this fixture with pinned tools. It establishes
neither self-hosting nor compatibility with the rest of CUE.
