# First source frontend

Run a real CUE expression from the repository root:

```sh
nix run ./infra/tooling#source -- bootstrap/source/valid/nested.cue
```

It prints `42`. `bootstrap-source <expression.cue>` reads source, parses it into
the existing version-1 expression tree, then invokes the compiled generated
evaluator. Success prints an integer and newline; errors print to stderr and
exit 2 without a result. No Go process is invoked at runtime.

`../grammar.cue` describes atoms, bound prefixes, infix operators, precedence,
associativity, grouping, and AST constructors for a bootstrap Pratt parser.
It exports to `grammar.json` in the Nix `exports` output and is compiled into
the frontend. [Whitespace, decimal, and ASCII identifier scanning](../scanner.md)
are CUE-authored statement programs compiled to Rust, including start-byte
recognition, scanning loops, decimal lexical rejection, operand-newline policy,
and parenthesis-token consumption/validation.
Checked i64 conversion, grammar validation, keyword lookup, AST construction,
bound/infix dispatch, expression recursion, and Pratt parsing remain native;
grammar configuration alone is not self-hosted parser machinery.

## Deliberately small subset

The default grammar accepts decimal signed integers, the `int` type atom,
inclusive bounds (`>=`, `<=`), addition, multiplication, unification (`&`),
and parentheses. Multiplication binds more
tightly than addition, which binds more tightly than `&`; all are left-associative.
These choices live in CUE grammar data, not in the parser algorithm.
`1 + 2 * 3` produces `7`, while `(1 + 2) * 3` produces `9`.
`1 + 2 & 3` produces `3`; `3 & 4` fails without printing a result.
`int & 3` produces `3`, `(int & 3) + 1` produces `4`, and bare `int` is incomplete.
A negative sign must touch its digits. Spaces and tabs are allowed;
line breaks can precede operands or follow an opening parenthesis, but cannot
follow an operand before another token. This avoids silently ignoring CUE's
implicit comma insertion at line breaks. The check and its diagnostic now live
in the CUE after-operand program. Trailing whitespace, including newlines at
EOF, is allowed. This is still conservative rejection, not full comma insertion.

### Inclusive bounds

`int & >=0 & <=10 & 3` produces `3`; replacing `3` with `11` conflicts.
Ranges can be intersected and retained as abstract intermediates. Empty
intersections conflict without a concrete candidate. Nonempty ranges are
incomplete, even `int & >=3 & <=3`: like upstream CUE, the evaluator does not
materialize a singleton from bounds alone. Adding `& 3` makes it concrete.
Missing endpoints are infinite, not clipped to i64: `int & >=9223372036854775807`
is still incomplete.

Bound operands are signed decimal i64 literals only. Spaces/newlines may
separate a prefix from its literal. Bounds bind before binary operators;
`>=1 + 2` tries arithmetic on an abstract bound and cannot produce a result.
Exclusive bounds, fractional thresholds, and general bound operands such as
`>=(1 + 2)` are outside this source subset.

Unification supports concrete integers, abstract `int`, and inclusive bounds,
but no other types, general symbolic constraints, or first-class bottom values.
Only scalar output requires concreteness. Arithmetic requires concrete operands,
so `int + 1` cannot produce a result. Full declarations, packages, comments,
references, other number spellings/operators, and general unary expressions
are unsupported. Concrete values, thresholds, and arithmetic results are bounded
to i64, narrower than upstream CUE's concrete integer domain. The parser limits
a source to 64 AST nodes and 32 recursive parsing levels; these are bootstrap
limits, not CUE limits. It rejects trailing input rather than a partial parse.

The `rejected/` examples include both invalid CUE and valid CUE outside this
subset, plus resource and arithmetic limits. They are not assertions that
upstream CUE should reject the same inputs. In contrast, every `conflicts/`
fixture is in the supported subset and must fail in both implementations.
`incomplete/` contains abstract outputs and arithmetic with non-concrete
operands; both tools must also reject these scalar exports.

## Checks and regeneration

Rust tests cover source execution, the existing nested AST, grammar-controlled
associativity, precedence and constructors, grammar validation, resource limits,
and CLI failures. The Nix `bootstrap-source` check compares every `valid/*.cue` result
byte-for-byte against the pinned upstream `cue export`, using exact decimal
output without passing integers through a floating-point JSON formatter. It
also requires both upstream CUE and the bootstrap tool to reject `conflicts/`
and `incomplete/` fixtures with no result. Conflict messages and exit-code numbers need not match.
Existing interpreter, generated JSON CLI, and code-generation checks remain.

Build grammar data from the repository root:

```sh
nix build ./infra/tooling#exports --out-link result-exports
```

The grammar is `result-exports/grammar.json`, not a committed JSON file. Fresh
Nix builds regenerate it before compiling the frontend. The devshell supplies
its directory as `CUESTACEAN_BOOTSTRAP_JSON` for local Cargo builds too. Source
fixtures must have an expression as their top-level value, not a field wrapper.
This is a narrow source path, not general CUE compatibility or self-hosting.
