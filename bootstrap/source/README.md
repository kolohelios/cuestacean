# First source frontend

`../grammar.cue` describes operator tokens, precedence, associativity, grouping,
and AST constructor names for a bootstrap Pratt parser. It exports to
`../grammar.json`; the parsing algorithm itself is initially handwritten Rust.
This is grammar configuration, not a claim that parser machinery is self-hosted.

The first supported source subset will be decimal signed integers, binary
addition, parentheses, and ASCII whitespace. Full CUE declarations, comments,
references, constraints, other number spellings, and other operators are out of
scope. Integers remain bounded to the bootstrap runtime's checked i64 domain.

The frontend will produce the existing expression-tree format and invoke the
generated evaluator. Supported examples will be checked against the pinned
upstream CUE tool. This is a narrow source path, not general CUE compatibility.

Regenerate grammar data from the repository root in the devshell:

```sh
cue export bootstrap/schema.cue bootstrap/grammar.cue --out json | jq -S . > bootstrap/grammar.json
```

Nix includes grammar export repeatability and malformed-grammar rejection in
the existing gate. No parser executable is introduced by the grammar-only PR.
