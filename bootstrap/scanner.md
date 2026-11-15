# First executable parser control flow

[`whitespace.cue`](whitespace.cue) owns the whitespace loop and newline branch.
[`decimal.cue`](decimal.cue) reuses the same DSL for the optional-minus branch
and digit loop, without new instructions. The handwritten
`bootstrap-scan-codegen <program.json>` compiler lowers these programs into
Rust control flow. The parser calls the generated [whitespace](generated/whitespace.rs)
and [decimal](generated/decimal.rs) scanners, not the statement interpreter.
No Go process runs during source evaluation.

This is more than a byte-class table: CUE defines the loop and branch bodies,
including when to advance and update state. These are two small algorithms,
**not a self-hosted parser or compiler**. Numeric validation/conversion, other
lexing, prefix parsing, the Pratt algorithm, the statement compiler/interpreter,
JSON loading, and safe cursor
primitives remain handwritten. Upstream Go CUE still exports specifications.

## Statement contract

`#ScanProgram` in `schema.cue` contains `formatVersion: 1` and a `body` list.

| Instruction | Meaning |
| --- | --- |
| `while` | Test `condition` before every iteration; execute `body` while true. |
| `if` | Test `condition`; execute `then` or the optional `else` list. |
| `advance` | Advance the cursor by one ASCII byte. |
| `setFlag` | Write the literal boolean `value` to the result register. |

The predicate `byteIn` tests the current byte against a nonempty ASCII byte
list. EOF never matches. Duplicate entries have set semantics; generation
sorts/deduplicates them and compacts contiguous runs into Rust range patterns.
There are no arbitrary Rust expressions, calls,
variables, AST construction, backtracking, or general-purpose computation yet.

Both execution paths start at a caller-supplied byte offset, with the flag
false, and return `{offset, flag}`. The whitespace program skips space, tab, LF,
and CR and remembers whether it encountered LF/CR. An alternate CUE program
skips only spaces/tabs and reports whether the last consumed byte was a space;
its compiled branch includes an `else` that clears the flag.

## Decimal token boundaries, not validation

The decimal program consumes an optional ASCII `-` followed by ASCII digits.
Its flag records whether a minus was consumed. Both integer literals and bound
thresholds call this generated scanner. It does not accept `+`, Unicode digits,
or other numeric spellings.

Scanning deliberately accepts incomplete or invalid numeric spans: `-` returns
an end offset of 1 with the flag true; `01` consumes both digits. The parser
still rejects missing digits and leading zeroes, then uses Rust's checked i64
conversion. Large digit runs can be scanned even when their numeric value is
out of range. These checks and conversion remain native, not CUE-defined.

## Explicit native safety boundary

The shared `scanner::Cursor` validates the initial UTF-8 character boundary,
checks EOF, and refuses to advance over non-ASCII bytes. Thus even an incorrect
program cannot split a UTF-8 character or index outside the source. Normal
whitespace scanning stops before Unicode; this does not add Unicode lexing.

Each action, `if` test, and `while` condition test consumes one execution step.
The budget is `128 * (remaining source bytes + 1)`, with saturating arithmetic.
A loop that fails to advance terminates with a budget error, not a hang.
Program validation caps statements at 64, nesting at 16, and predicate lists
at 128 ASCII bytes. These are bootstrap limits, not CUE language limits or a
production-hardening claim. Fuel checks are emitted into generated loops as
well as applied by the interpreter.

## Validation and regeneration

From the repository root:

```sh
nix develop ./infra/tooling --command cargo test --manifest-path bootstrap/executor/Cargo.toml --test scanner
```

Tests compare the interpreter, compiled normal/alternate programs, and known
expectations, including all 128 ASCII bytes, decimal sign/digit boundaries,
long whitespace/digit runs, CRLF, Unicode boundaries, EOF and invalid offsets. Separate compiled programs exercise
non-progressing loops and unsafe advancement. Invalid statement documents must
fail before source is emitted. Existing parser tests and upstream source
comparisons continue to check comma-insertion conservatism and all supported
expressions.

See [generated-module regeneration](generated/README.md) for the pinned local
commands. Nix exports JSON only as a build output and regenerates all scanner
Rust before compiling the final package. The handwritten seeds disable the
default `source` feature, excluding the parser/generated scanner dependency;
the final package enables it and runs the full suite. Committed Rust modules
remain drift-check goldens, never inputs required by the seed build.
