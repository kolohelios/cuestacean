# First executable parser control flow

[`whitespace.cue`](whitespace.cue) owns the whitespace loop, newline branch,
and flag updates. The handwritten `bootstrap-scan-codegen <program.json>`
compiler lowers this statement program into Rust `while` and `if`/`else` control
flow. The parser calls the [generated scanner](generated/whitespace.rs), not
the statement interpreter. No Go process runs during source evaluation.

This is more than a byte-class table: CUE defines the loop and branch bodies,
including when to advance and update state. It is one small algorithm, **not a
self-hosted parser or compiler**. Other lexing, prefix parsing, the Pratt
algorithm, the statement compiler/interpreter, JSON loading, and safe cursor
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
sorts and deduplicates them. There are no arbitrary Rust expressions, calls,
variables, AST construction, backtracking, or general-purpose computation yet.

Both execution paths start at a caller-supplied byte offset, with the flag
false, and return `{offset, flag}`. The normal program skips space, tab, LF,
and CR and remembers whether it encountered LF/CR. An alternate CUE program
skips only spaces/tabs and reports whether the last consumed byte was a space;
its compiled branch includes an `else` that clears the flag.

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
expectations, including all 128 ASCII bytes, long whitespace runs, CRLF, Unicode
boundaries, EOF and invalid offsets. Separate compiled programs exercise
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
