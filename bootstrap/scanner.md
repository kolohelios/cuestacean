# First executable parser control flow

[`scanner-fragments.cue`](scanner-fragments.cue) defines shared whitespace/comment
statements used by [`whitespace.cue`](whitespace.cue),
[`group-close.cue`](group-close.cue), and [`after-operand.cue`](after-operand.cue).
The latter owns the conservative
newline-after-operand rule.
[`decimal.cue`](decimal.cue) supplies optional-minus/digit scanning and lexical
validation, using a generic `reject` instruction for diagnostics.
[`identifier.cue`](identifier.cue) supplies ASCII identifier start/continuation
scanning. [`group-open.cue`](group-open.cue) and `group-close.cue` recognize and
consume parenthesis tokens. [`bound-prefix.cue`](bound-prefix.cue) and
[`infix.cue`](infix.cue) recognize symbols. The handwritten
`bootstrap-scan-codegen <program.json>` compiler lowers these programs into
Rust control flow. The parser calls generated whitespace, after-operand,
decimal, identifier, parenthesis, and symbol scanners, not the statement interpreter.
No Go process runs during source evaluation.

This is more than a byte-class table: CUE defines the loop and branch bodies,
including when to advance, inspect state, update state, and reject input.
These are small scanning algorithms and policies,
**not a self-hosted parser or compiler**. Checked i64 conversion, grammar
validation, grammar lookup and frame safety,
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
| `advanceChar` | Advance one complete UTF-8 scalar; fail at EOF. |
| `break` | Exit the nearest enclosing `while`; invalid outside loops. |
| `setFlag` | Write the literal boolean `value` to the result register. |
| `reject` | Stop with the current byte offset and a nonempty `reason` string. |

The predicate `byteIn` tests the current byte against a nonempty ASCII byte
list. Optional `ahead` (default 0, maximum 64) reads that many bytes ahead without
consumption. Reading past EOF never matches. Duplicate entries have set semantics; generation
sorts/deduplicates them and compacts contiguous runs into Rust range patterns.
`flagIs` compares the current flag to its literal boolean `value`; `hasByte`
tests for any remaining byte, including non-ASCII bytes. These predicates are
pure reads; each surrounding `if`/`while` test consumes one execution step.
`charIn` tests the current Unicode scalar against 1..=128 single-character
strings; duplicate/order differences have set semantics. Generated character
literals are safely escaped. This is scalar matching, not grapheme matching.
There are no arbitrary Rust expressions, calls, named variables, AST
construction, or backtracking yet.

Both execution paths start at a caller-supplied byte offset, with the flag
false, and return `{offset, flag}`. The whitespace program skips space, tab, LF,
and CR and remembers whether it encountered LF/CR. An alternate CUE program
skips only spaces/tabs and reports whether the last consumed byte was a space;
its compiled branch includes an `else` that clears the flag.

## CUE-owned line-comment trivia

Shared trivia recognizes `//` before stopping at a non-whitespace token. The
comment loop advances complete UTF-8 scalars until LF or EOF. CR alone remains
comment content; LF is left to the outer whitespace loop, which records the
newline flag. Unicode text is allowed inside comments, not in identifiers or
ordinary whitespace. CUE rejects NUL and embedded BOM with byte offsets and
`unsupported character in line comment`; those decisions are not cursor code.

Leading/trailing and operator-before-operand comments are supported. A comment
separates tokens rather than joining identifiers/word operators across lines.
Pinned CUE rejects `/* ... */`; block comments are not introduced here.

`break` propagates through `if` but stops at its nearest loop. The compiler omits
unreachable suffixes, merges identical pure branches after charging the `if`
step, and lowers provably once-only loops to a condition plus a labeled block.
This preserves flow/fuel without unreachable-code or never-loop lint suppression.
A compiled alternate program tests Unicode escaping and nested break targets.

## Operand newline policy, not full comma insertion

After scanning trivia, the operand policy tests the newline flag and
whether any byte remains. It rejects a newline followed by more input, including
Unicode or a closing parenthesis, but accepts trailing newlines at EOF. The
parser no longer tests this condition or owns the rejection text.

This preserves the existing deliberately conservative source subset; it does
not implement full CUE comma insertion or add multiline expression forms.
Whitespace before operands or after an operator still uses the permissive
program. Tests compare both programs on the same input to keep that distinction
explicit. Pinned CUE also rejects a newline after a word operator. CUE Pratt
control selects `after-word-operator.cue` using the operator token length (all
current word operators are three bytes); the native frame only exposes that
metadata and runs the selected scanner. This covers comments before the LF too.
Nix exports load the schema, shared fragment definitions, and selected
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
identifiers. Keyword lookup remains native; [AST construction](ast.md) uses a
separate CUE-defined expression DSL, not scanner instructions.

## Parenthesis tokens, not expression recursion

The opening program recognizes `(` at the current offset and consumes one byte,
setting its flag true. Other starts, including EOF, leave offset/flag unchanged.
The parser dispatches on this result rather than classifying the source byte.

The closing program skips shared whitespace, consumes `)`, or rejects with
`expected closing parenthesis` at the current offset. Its flag describes skipped
newlines, not recognition; failure is an error. Grouping is still fixed to
parentheses by native grammar validation, not configurable to other delimiters.

The separate [Pratt control DSL](pratt.md) owns the expression loop and
right-hand recursion decisions. Its [primary program](primary.md) also owns
primary dispatch, grouping minimum and closing order. Empty groups
still fail as missing operands, and a newline before `)` is rejected by the
operand policy before the closing program runs. Direct closing scans can skip
newlines; that does not expand the parser's conservative source subset.

## Symbol recognition, not grammar meaning

The bound-prefix program checks `>=` or `<=` with one-byte lookahead before
advancing. A partial match such as `>`, `> x`, or `>λ` leaves both offset and
flag unchanged. The infix program recognizes a byte from `+`, `-`, `*`, `/`, `&`
or a complete `div`, `mod`, `quo`, `rem` keyword. CUE checks the three keyword
bytes and rejects an ASCII identifier-continuation suffix before advancing;
partial/extended words leave offset and flag unchanged. A keyword consumes
exactly three bytes, leaving a following minus for operand parsing. Recognizing
`/` does not enable rational division: the default grammar has no `/` entry.

The native parser looks up complete recognized spans in runtime grammar data.
It commits an infix span only after the binding-power check, so lookahead does
not consume lower-precedence operators prematurely. Disabled bounds/operators,
constructor names, fields, precedence and associativity still come from grammar
data. These configuration tests are not assertions of alternate CUE semantics.
The parser no longer directly peeks at source bytes. Grammar lookup remains
native; AST construction and expression-loop policy use their separate CUE DSLs.

## Explicit native safety boundary

The shared `scanner::Cursor` validates the initial UTF-8 character boundary,
checks EOF, and keeps `advance` ASCII-only. The explicit `advanceChar` primitive
consumes a whole scalar instead; neither can split UTF-8 or index outside the
source. Current default programs use scalar advancement only inside comments.
Ordinary whitespace/identifier recognition remains ASCII.
`peek_at` uses checked offset addition and bounds-checked byte access, returning
no byte past EOF or on overflow. It may inspect bytes inside UTF-8, but does
not advance the cursor or create a string slice.

Each action, `if` test, and `while` condition test consumes one execution step.
The budget is `128 * (remaining source bytes + 1)`, with saturating arithmetic.
A nonterminating loop exhausts its budget rather than hanging. A stateful loop
may legitimately finish without advancing by changing its flag.
Program validation caps statements at 64, nesting at 16, and predicate lists
at 128 ASCII bytes or 128 Unicode characters. These are bootstrap limits, not CUE language limits or a
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
whitespace, Unicode/EOF, nesting, and exact diagnostics/error precedence. Symbol
tests cover ASCII pairs, partial matches, maximum lookahead, disabled grammar
entries, operator configuration and all ASCII keyword suffixes.
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
