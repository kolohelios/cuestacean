# First executable parser control flow

[`scanner-fragments.cue`](scanner-fragments.cue) defines shared whitespace loop
statements used by [`whitespace.cue`](whitespace.cue),
[`group-close.cue`](group-close.cue), and [`after-operand.cue`](after-operand.cue).
The latter owns the conservative
newline-after-operand rule.
[`decimal.cue`](decimal.cue) supplies optional-minus/digit scanning and lexical
validation, using a generic `reject` instruction for diagnostics.
[`identifier.cue`](identifier.cue) supplies ASCII identifier start/continuation
scanning. [`group-open.cue`](group-open.cue) and `group-close.cue` recognize and
consume parenthesis tokens. The handwritten
`bootstrap-scan-codegen <program.json>` compiler lowers these programs into
Rust control flow. The parser calls generated whitespace, after-operand,
decimal, identifier, and parenthesis scanners, not the statement interpreter.
No Go process runs during source evaluation.

This is more than a byte-class table: CUE defines the loop and branch bodies,
including when to advance, inspect state, update state, and reject input.
These are small scanning algorithms and policies,
**not a self-hosted parser or compiler**. Checked i64 conversion, grammar
validation, keyword lookup, AST construction, bound/infix dispatch, expression
recursion and Pratt parsing,
the statement compiler/interpreter,
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
| `reject` | Stop with the current byte offset and a nonempty `reason` string. |

The predicate `byteIn` tests the current byte against a nonempty ASCII byte
list. EOF never matches. Duplicate entries have set semantics; generation
sorts/deduplicates them and compacts contiguous runs into Rust range patterns.
`flagIs` compares the current flag to its literal boolean `value`; `hasByte`
tests for any remaining byte, including non-ASCII bytes. These predicates are
pure reads; each surrounding `if`/`while` test consumes one execution step.
There are no arbitrary Rust expressions, calls, named variables, AST
construction, or backtracking yet.

Both execution paths start at a caller-supplied byte offset, with the flag
false, and return `{offset, flag}`. The whitespace program skips space, tab, LF,
and CR and remembers whether it encountered LF/CR. An alternate CUE program
skips only spaces/tabs and reports whether the last consumed byte was a space;
its compiled branch includes an `else` that clears the flag.

## Operand newline policy, not full comma insertion

After scanning whitespace, the operand policy tests the newline flag and
whether any byte remains. It rejects a newline followed by more input, including
Unicode or a closing parenthesis, but accepts trailing newlines at EOF. The
parser no longer tests this condition or owns the rejection text.

This preserves the existing deliberately conservative source subset; it does
not implement full CUE comma insertion or add multiline expression forms.
Whitespace before operands or after an operator still uses the permissive
program. Tests compare both programs on the same input to keep that distinction
explicit. Nix exports load the schema, shared fragment definitions, and selected
program; fragment definitions are not separate JSON artifacts.

## Decimal lexical validation, not conversion

The decimal program consumes an optional ASCII `-` followed by ASCII digits.
Its flag records whether a minus was consumed. Both integer literals and bound
thresholds call this generated scanner. CUE branches reject missing digits and
leading zeroes; the parser no longer implements those decisions. This does not
add `+`, Unicode digits, or other numeric spellings.

A missing digit is reported at the current cursor (after the minus, if any).
A leading-zero run is consumed fully before rejection, preserving the previous
end-of-run byte offset. Reasons come from CUE data, not native numeric error
cases. The parser maps scanner rejection into its ordinary source diagnostic,
without a `scanner failed` prefix. Other cursor/validation/fuel failures retain
the scanner-error context.

Large validly spelled digit runs still scan successfully even when their value
exceeds i64. Checked conversion remains Rust's `parse::<i64>`. A separate
permissive CUE program accepts `01` and `-`, demonstrating that lexical policy
is in the statement program rather than the shared native cursor.

## Identifier boundaries, not keyword lookup

The identifier program requires an ASCII letter or underscore first, followed
by ASCII letters, digits, or underscores. Its flag indicates recognition. An
invalid start (including a digit) leaves the offset unchanged and flag false.
The parser dispatches on this generated flag rather than a native byte-class
test, then looks up the complete span in the grammar's atom table.

Thus `int9` and `int_1` are complete identifiers, not `int` followed by a suffix.
They are unknown atoms with the default grammar, but work if configured as atom
names. Case is preserved; this adds neither reference resolution nor Unicode
identifiers. Keyword lookup and AST construction remain native.

## Parenthesis tokens, not expression recursion

The opening program recognizes `(` at the current offset and consumes one byte,
setting its flag true. Other starts, including EOF, leave offset/flag unchanged.
The parser dispatches on this result rather than classifying the source byte.

The closing program skips shared whitespace, consumes `)`, or rejects with
`expected closing parenthesis` at the current offset. Its flag describes skipped
newlines, not recognition; failure is an error. Grouping is still fixed to
parentheses by native grammar validation, not configurable to other delimiters.

Expression recursion and the order of scanner calls remain native. Empty groups
still fail as missing operands, and a newline before `)` is rejected by the
operand policy before the closing program runs. Direct closing scans can skip
newlines; that does not expand the parser's conservative source subset.

## Explicit native safety boundary

The shared `scanner::Cursor` validates the initial UTF-8 character boundary,
checks EOF, and refuses to advance over non-ASCII bytes. Thus even an incorrect
program cannot split a UTF-8 character or index outside the source. Normal
whitespace scanning stops before Unicode; this does not add Unicode lexing.

Each action, `if` test, and `while` condition test consumes one execution step.
The budget is `128 * (remaining source bytes + 1)`, with saturating arithmetic.
A nonterminating loop exhausts its budget rather than hanging. A stateful loop
may legitimately finish without advancing by changing its flag.
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
identifier start/continuation rules, long runs, CRLF, Unicode boundaries, EOF
and invalid offsets. Operand tests cover ASCII suffix bytes, CRLF, EOF, Unicode,
flag updates, and exact parser/CLI newline diagnostics. Atom tests verify
complete-token lookup and custom names. Parenthesis tests cover all ASCII bytes,
whitespace, Unicode/EOF, nesting, and exact diagnostics/error precedence.
Separate compiled programs exercise non-progressing loops, unsafe advancement,
escaped rejection text, short-circuiting, and alternate decimal policy. Parser
and CLI tests assert exact lexical diagnostic offsets and messages. Invalid statement documents must
fail before source is emitted. Existing parser tests and upstream source
comparisons continue to check comma-insertion conservatism and all supported
expressions.

See [generated-module regeneration](generated/README.md) for the pinned local
commands. Nix exports JSON only as a build output and regenerates all scanner
Rust before compiling the final package. The handwritten seeds disable the
default `source` feature, excluding the parser/generated scanner dependency;
the final package enables it and runs the full suite. Committed Rust modules
remain drift-check goldens, never inputs required by the seed build.
