# CUE-hosted expression control flow

[`pratt.cue`](pratt.cue), through `pratt-fragments.cue`, owns the expression loop:
parse a primary, run a CUE [postfix index loop](list-indexing.md), then inspect
an infix candidate, compare its precedence with the frame minimum, adjust the
right binding power for left associativity, commit the operator, recurse, and
combine the operands. `bootstrap-pratt-codegen <control.json> [lowering.json]` emits real Rust
`while`/`if` control flow and checked frame calls. The production parser executes
that generated function, not the reference interpreter or a native Pratt loop.

This is a larger executable boundary, **not a fully self-hosted parser or
compiler**. The backend still handles grammar validation/lookup, scanner/builder
invocation, checked i64 conversion, and safe frame transitions. The separate
[primary program](primary.md) now owns primary dispatch and grouping control,
including the group minimum. Right-hand recursion takes its binding power from
the expression program.
[Compiler emission](pratt-lowering.md) now runs from a separate compiled CUE
backend using the shared lowering compiler. Typed decoding, the reference parser
interpreter and checked rendering/frame kernels remain native. Go CUE still
exports the specifications; see the [source profile](source/README.md) for
supported syntax.

## Control contract

`#PrattProgram` contains version 1, a statement `body`, and optional `target`
(default `parser`). The [list family](list-expressions.md) reuses structured
if/while/return with checked list-cursor actions, not parser callbacks. Both
schema/native decoders eagerly reject cross-family actions and conditions,
including unused branches. Parser driver Rust remains byte-identical; list-target
headers and calls are CUE-selected, without another compiler or seed. Statements include
`while`, `if`/`else`, charged [driver `return`](parser-return.md),
`setBindingPower`, and explicit frame intrinsics:
`primary`, `afterOperand`, `lookupOperator`, `advanceOperator`, `parseRight`,
and `buildBinary`. `primary` dispatches the separate CUE primary program.
There is deliberately no `parsePratt` instruction that hides the loop in Rust.
The [primary contract](primary.md) documents its additional operations and
conditions in this same version-1 DSL.

[Record selectors](source-selectors.md) interleave with bracket indexing under
CUE hasPostfix iteration; native candidate facts do not choose token or precedence
policy. The actual former bracket-only driver remains pratt-no-selectors.
Postfix scanIndex/hasIndex/advanceIndex/parseIndex/closeIndex run before binary
lookup, with a CUE-selected index minimum 0. Native private stages prevent pending
indices from escaping on return or colliding with primary/binary operations.
Optional grammar constructor metadata can disable indexing. An actual pre-index
Pratt baseline remains byte-exact, and a compiled minimum100 alternate changes
index parsing without a native fallback. The right-associative alternate also
includes postfix control, propagating all three selected expression/primary/
record-primary engines into indices.

Conditions read `hasOperator`, `leftAssociative`, literal booleans, numeric
`atLeast`, or short-circuit `and`. Numbers read the frame `minimum`, current
operator `precedence`/`tokenLength`, a literal u8, or checked `add`. In the normal program,
left-associative operators recurse with precedence + 1; right-associative ones
use precedence unchanged. Operator commitment occurs after the comparison,
so a lower-precedence candidate remains available to the caller. After commit,
CUE checks token length and selects `afterWordOperator` for the current three-byte
word operators, preserving pinned CUE's newline rejection before RHS parsing.

Each expression frame owns its left/right values, pending operator, proposed
binding power, and input minimum/depth. Native intrinsics reject invalid action
ordering rather than exposing mutable fields. Group recursion and right-hand
recursion select the same driver, including alternate compiled/interpreted
programs. Binary construction calls the existing CUE-defined AST builder.

## Safety and evidence

Each action, `if` test, and `while` condition test consumes one of 4096 steps
per expression frame. Existing native source limits remain 64 AST nodes and
32 expression levels. Binding-power addition is checked u8 arithmetic, not
wrapping. Control documents are limited to 512 JSON nodes and 32 JSON levels
before typed decoding/emission. Whole-program validation also rejects statements
following a structurally definite return, even in an unused branch; return does
not bypass primary completion checks. These are bootstrap limits, not production
hardening or CUE language limits.

Tests compare complete trees **and error strings** between generated and
interpreted execution over every source fixture, including rejected inputs.
Existing pinned-upstream comparisons exercise the production generated driver.
Alternate CUE control omits the associativity increment, producing a right-nested
chain and changing checked-intermediate-overflow behavior. Separate compiled
programs verify fuel exhaustion, invalid state, and binding-power overflow.
Runtime grammar mutations still control precedence, associativity, kinds and
field names. Parser/CLI lexical diagnostics remain covered by their existing
exact checks.

`parse_with_driver` and `parse_with_drivers` are trusted native hooks; arbitrary Rust
drivers are not sandboxed. Fuel checks are guaranteed by the compiler and
interpreter for this DSL, not by arbitrary caller-written Rust.

Nix builds the native control compiler bridge in its artifact-free seeds, using
the reference engine of the same CUE compiler policy. It regenerates the compiled
backend and all parser drivers before final default-feature compilation/tests.
JSON remains build-only; committed Rust remains a drift baseline. Local checks:

```sh
nix develop ./infra/tooling --command cargo test --manifest-path bootstrap/executor/Cargo.toml --test pratt
```

See [generated-module regeneration](generated/README.md) for the full commands.
