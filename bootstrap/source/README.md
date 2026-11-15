# First source frontend

Run a real CUE expression from the repository root:

```sh
nix run ./infra/tooling#source -- bootstrap/source/valid/nested.cue
```

It prints `42`. `bootstrap-source <expression.cue>` reads source, parses it into
the existing version-1 expression tree, then invokes the compiled generated
evaluator. Success prints an integer and newline; errors print to stderr and
exit 2 without a result. No Go process is invoked at runtime.

`../grammar.cue` describes operator tokens, precedence, associativity, grouping,
and AST constructor names for a bootstrap Pratt parser. It exports to
`../grammar.json` and is compiled into the frontend. The lexer and Pratt
algorithm are handwritten Rust scaffolding; this is grammar configuration,
not a claim that parser machinery is self-hosted.

## Deliberately small subset

The default grammar accepts decimal signed integers, addition, multiplication,
concrete integer unification (`&`), and parentheses. Multiplication binds more
tightly than addition, which binds more tightly than `&`; all are left-associative.
These choices live in CUE grammar data, not in the parser algorithm.
`1 + 2 * 3` produces `7`, while `(1 + 2) * 3` produces `9`.
`1 + 2 & 3` produces `3`; `3 & 4` fails without printing a result. A negative sign must touch its digits. Spaces and tabs are allowed;
line breaks can precede operands or follow an opening parenthesis, but cannot
follow an operand before another token. This avoids silently ignoring CUE's
implicit comma insertion at line breaks. Trailing whitespace is allowed.

Unification only compares concrete i64 results; it does not solve symbolic
constraints or represent bottom as a value. Full declarations, packages,
comments, references, type constraints, other number
spellings, other operators, and general unary expressions are unsupported.
Integers and results remain checked i64 values. The parser limits a source to
64 AST nodes and 32 recursive parsing levels; these are bootstrap limits, not
CUE limits. It rejects trailing input rather than accepting a partial parse.

The `rejected/` examples include both invalid CUE and valid CUE outside this
subset, plus resource and arithmetic limits. They are not assertions that
upstream CUE should reject the same inputs. In contrast, every `conflicts/`
fixture is in the supported subset and must fail in both implementations.

## Checks and regeneration

Rust tests cover source execution, the existing nested AST, grammar-controlled
associativity, precedence and constructors, grammar validation, resource limits,
and CLI failures. The Nix `bootstrap-source` check compares every `valid/*.cue` result
byte-for-byte against the pinned upstream `cue export`, using exact decimal
output without passing integers through a floating-point JSON formatter. It
also requires both upstream CUE and the bootstrap tool to reject `conflicts/`
fixtures with no result. Conflict messages and exit-code numbers need not match.
Existing interpreter, generated JSON CLI, and code-generation checks remain.

Regenerate grammar data from the repository root in the devshell:

```sh
cue export bootstrap/schema.cue bootstrap/grammar.cue --out json | jq -S . > bootstrap/grammar.json
```

Fresh Nix builds regenerate this grammar before compiling the frontend. Source
fixtures must have an expression as their top-level value, not a field wrapper.
This is a narrow source path, not general CUE compatibility or self-hosting.
