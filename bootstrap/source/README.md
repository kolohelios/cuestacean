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
parenthesis-token handling, and bound/infix token recognition. The native parser
no longer peeks directly at source bytes. Checked i64 conversion, grammar
validation/lookup and frame safety remain native;
grammar configuration alone is not self-hosted parser machinery. The
[Pratt expression loop](../pratt.md) now executes CUE-authored control flow for
operator repetition, binding-power checks, associativity and recursive-right
sequencing. The separate [primary program](../primary.md) owns dispatch among
groups, bounds, atoms and integers, plus the grouping minimum and close order.
This is not yet a fully self-hosted parser.
[AST construction](../ast.md) now uses CUE-defined builder expressions compiled
to typed Rust functions, rather than native JSON-map assembly.

## Deliberately small subset

The default grammar accepts decimal signed integers, the `int` type atom,
inclusive bounds (`>=`, `<=`), addition, subtraction, multiplication,
integer division/remainder (`div`, `mod`, `quo`, `rem`), unification (`&`),
and parentheses. Multiplication and integer division/remainder bind more tightly than
addition/subtraction, which share a precedence and bind more tightly than `&`;
binary operators are left-associative, and unary negation binds more tightly.
These choices live in CUE grammar data, not in the parser algorithm.
`1 + 2 * 3` produces `7`, while `(1 + 2) * 3` produces `9`.
`1 + 2 & 3` produces `3`; `3 & 4` fails without printing a result.
`int & 3` produces `3`, `(int & 3) + 1` produces `4`, and bare `int` is incomplete.
A digit-adjacent minus stays part of a signed decimal literal. Other minus
prefixes use expression negation, allowing spaces or newlines after the sign.
Spaces and tabs are allowed;
line breaks can precede operands or follow an opening parenthesis, but cannot
follow an operand before another token. This avoids silently ignoring CUE's
implicit comma insertion at line breaks. The check and its diagnostic now live
in the CUE after-operand program. Trailing whitespace, including newlines at
EOF, is allowed. This is still conservative rejection, not full comma insertion.

### Checked subtraction

`50 - 8` produces `42`; `10-3-2` produces `5`, while `10-(3-2)` produces `9`.
Binary minus and signed decimal primaries are distinct parsing contexts, so
`1--2` produces `3` and `-2-3` produces `-5`. The CUE infix scanner recognizes
minus; the existing CUE grammar, Pratt control and AST builder supply its meaning.
No special subtraction branch is added to parser control.

The rule `difference` evaluates left before right and calls native checked i64
subtraction. Subtracting i64::MIN from itself yields zero; overflowing intermediates
fail even if later subtraction would bring the mathematical result into range.
There is no wrapping or arbitrary precision. Abstract arithmetic such as `int - 1` still needs concrete
operands rather than backward constraint propagation.

### Integer division and remainder

`div` and `mod` use Euclidean division: the remainder is nonnegative and less
than the divisor's magnitude. `quo` truncates toward zero and `rem` has the
same sign as the dividend (or is zero). Thus `-7 div 3` is `-3`, `-7 mod 3`
is `2`, while `-7 quo 3` is `-2` and `-7 rem 3` is `-1`.

All four share multiplication's precedence and are left-associative. The CUE
scanner requires complete keywords: `quo2` and `div_name` do not consume any
operator bytes. Whitespace is not required at a number boundary (`8quo 2` is
`4`), and a signed RHS remains intact (`7quo-3` is `-2`). This does not enable
rational `/`, which remains unregistered in the default grammar.

All zero divisors fail without output. The two quotient operators reject
i64::MIN divided by `-1` as overflow; both remainder operators correctly return
zero for that pair. A divisor of i64::MIN is valid: `-1 mod -9223372036854775808`
produces i64::MAX, without taking an overflowing absolute value. Values and
intermediates remain i64; arithmetic still requires concrete operands. Integer
zero is printed canonically as `0`, even where upstream CUE spells it `-0`.

### Unary negation

`-(1+2)` produces `-3`, `-(1+2)*3` produces `-9`, and `--1` produces `1`.
CUE primary control selects the prefix, recursively parses with minimum 101
(above the grammar's maximum binary precedence 100), then builds a unary AST.
The separate CUE prefix scanner leaves digit-adjacent minus to decimal parsing,
so `-9223372036854775808` remains a valid literal without constructing its
out-of-range positive magnitude. Negating that value as an expression overflows.
The native `negate` primitive uses checked i64 negation; abstract operands still
require concreteness. Optional grammar metadata controls prefix registration
and constructor names/fields, without disabling signed literals when absent.

The former rejected double-sign, sign-space and sign-newline fixtures are now
supported upstream-compared cases. A dangling sign followed by a newline now
reports the missing operand at EOF, after unary whitespace processing.

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
references, other number spellings/operators, and other unary operators
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
against pinned upstream `cue export`: only the exact scalar spelling `-0` is
canonicalized to `0`, since both denote integer zero. All nonzero decimal output
remains byte-exact, without a floating-point JSON formatter. It
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
