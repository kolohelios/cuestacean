# Bootstrap rule executor

Disposable handwritten Rust machinery that gives the exported CUE rules their
first executable meaning. This is not a general CUE parser, compiler, or constraint
solver. The long-term goal remains replacing this scaffolding with generated
code from a CUE-hosted implementation DSL.

From the repository root:

```sh
nix build ./infra/tooling#exports --out-link result-exports
nix run ./infra/tooling -- result-exports/literal.json result-exports/evaluation.json
nix run ./infra/tooling -- result-exports/addition.json result-exports/evaluation.json
```

Both print `42`. The CLI takes exactly two paths: program JSON, then rule JSON.
Success prints an exact JSON integer, boolean or string and newline; errors go to stderr with exit
status 2 and no result on stdout.

The generated evaluator is a separate CLI:

```sh
nix run ./infra/tooling#generated -- result-exports/addition.json
```

It prints `42` and takes only a program path. The normal `evaluation.json` rules
are compiled into it through `../generated/evaluation.rs`; it does not read or
interpret a rule file at runtime. Nix builds JSON from CUE specifications and
regenerates Rust in its build sandbox before compilation. JSON is build-only;
after changing CUE rules, regenerate the committed Rust modules so their drift
check passes. The default Nix entry point remains the
interpreter.

Both evaluators use the same output and evaluation-error conventions. Their
usage messages differ because their argument lists differ.

Library `execute_json(program, rules)` and `generated::execute_json(program)`
return concrete JSON scalars. Legacy `execute` entry points still return i64:
concrete booleans/strings report `integer output requires an integer value`.
The normal generated module is a public, `semantics`-gated library API reused by
both compiled CLIs; compiler-only seeds omit it entirely.

## Instruction meanings

The executor looks up a node's string `kind` in the supplied rule map, then
interprets that rule's result instruction against the current node:

- `null` constructs the concrete null singleton; `top` constructs abstract top.
  [CUE lattice policy](../lattice-values.md) owns top identity and null isolation.
- `int` / `bool` / `stringType` construct abstract type values.
- `text` reads a named string field into owned shared text.
- `boolean` constructs its literal `value` from rule data, not a node-kind case.
- `lowerBound` / `upperBound` execute their `value` instruction, require a
  concrete integer threshold, and construct an inclusive one-sided bound.
- `field` reads the named field as a concrete integer value.
- `evaluate` selects the named child and recursively dispatches its rule.
- `sum` executes its left instruction, then its right instruction, against the
  same current node. [CUE binary control](../sum.md) chooses checked integer
  addition, concrete text concatenation, or operand rejection.
- `product` executes its left and right instructions in the same order and
  multiplies their integer results using checked arithmetic. Like difference and
  negation, [CUE integer control](../integer-control.md) now owns domain/operation
  dispatch; borrowed native kernels retain the exact checked arithmetic.
- `difference` evaluates left before right and subtracts right from left using
  checked i64 subtraction. It never negates the right operand first; subtracting
  i64::MIN from itself correctly yields zero.
- `negate` evaluates its `value` instruction and applies checked i64 negation.
  Negating i64::MIN overflows; parsing that signed literal directly does not.
- `divide` / `modulo` use Euclidean division and nonnegative remainder;
  `quotient` / `remainder` truncate toward zero with dividend-signed remainder.
  Both children evaluate left-first before concreteness/zero-divisor checks.
  Quotient overflow fails; both remainders of i64::MIN by -1 are zero.
  [CUE division policy](../division-control.md) now owns domain/zero/mode and
  unit-divisor selection; native primitives retain checked numeric computation.
- `logicalNot` evaluates its `value` child then uses [CUE unary control](../logical-not.md)
  to validate/invert a concrete boolean. It has a genuine single-value signature.
- `logicalAnd` / `logicalOr` evaluate both children left-first, then use [CUE
  logical selection](../logic.md). Both values must be concrete; a determining
  left boolean can ignore a concrete RHS type, but never its evaluation error.
- `less` / `lessEqual` / `greater` / `greaterEqual` use [CUE ordering](../ordering.md)
  for matching concrete integers or text; other pairs reject. Both children
  evaluate left-first before checks. String order is lexicographic, not normalized.
- `equal` / `notEqual` evaluate left then right and delegate to [CUE scalar
  equality](../equality.md): concrete i64/bool/text compare by content, mixed
  concrete types compare unequal, abstract values/bounds reject. No coercion.
- `unify` refines `int`, intersects bounds, and checks concrete candidates.
  Equal concrete integers agree; differing ones report
  `conflicting integer values`. Empty ranges and rejected candidates report
  `conflicting integer bounds`.

The owned Clone (not Copy) value scaffold has null/top and concrete i64/boolean/shared text,
abstract `int`/`bool`/`string`, and inclusive bounds. CUE controls string identity,
content equality/conflict and cross-domain rejection. JSON strings and the [bounded source-string profile](../strings.md) are supported. See the [ownership design](../../docs/string-values.md).
`bool & true` is concrete; `bool & bool` reports `non-concrete bool value`.
Different concrete booleans report `conflicting boolean values`; boolean/numeric
mixes report `conflicting value types` before numeric identity can apply.
Intermediate abstract values are valid: `int & int & 3` produces `3`. Only the
outer scalar-output boundary requires concreteness; bare `int` reports
`non-concrete int value`, not a conflict. Bounds report
`non-concrete integer bounds` until given a concrete candidate, even for
`int & >=3 & <=3`. Missing
endpoints are infinite: `int & >=9223372036854775807` remains incomplete rather
than being clipped into a singleton at i64::MAX. The value representation and
projections remain native; [CUE-defined unification control](../unify.md) now owns
intersection and identity/conflict policy. This is not a general CUE value lattice.

Other primitives remain native, without special cases for expression kinds.
The CUE-exported `tests/valid/evaluation-left-twice.json` in the exports output changes addition to
evaluate the left child twice. With the unchanged addition tree and executable,
that rule file yields `40`, demonstrating that rule data controls behavior.

## Rust generation

The package also includes `bootstrap-codegen <rules.json> [lowering.json]`. It emits a Rust
module to stdout and rejects malformed or unsupported rule documents before
emitting source. Names are escaped as Rust string literals, and dispatch order
is deterministic regardless of input object-key order. Per-instruction recursion,
call selection and Result layout come from a [CUE lowering program](../lowering.md),
not a native instruction-emission match. Default `compiled-lowering` uses a
CUE-generated Rust driver. Compiler-only seeds interpret fresh upstream-built
JSON instead, as do `generate_with_policy`/the optional CLI policy override.
`bootstrap-lower-codegen <policy.json> [compiler.json]` is the sixth bootstrap
compiler. Its [CUE meta compiler](../meta-lowering.md) composes complete driver
source, including shell/module statements and its own regeneration. Seeds interpret
fresh JSON; production uses compiled meta Rust. Native code validates/projects data
and supplies the checked VM/kernels; no native source-assembly fallback remains.
[Module statements](../module.md) now own boilerplate and rule traversal too;
only typed serialization, canonical storage, the policy compiler, checked
rendering and cursor/resource guards remain native. Expression-only policies
need an explicit module program to assemble an evaluator; there is no native
header/footer fallback.

The [generated modules](../generated/README.md) use the shared native runtime
rather than the instruction interpreter. Tests compile them, compare values and
errors against the interpreter, and verify that alternate rule data generates
alternate behavior. `bootstrap-generated` runs the normal generated module;
`bootstrap-executor` remains available for runtime rule changes and comparison.

## Source frontend

`bootstrap-source <expression.cue>` parses the [documented source subset](../source/README.md)
using compiled-in CUE grammar data, then invokes the generated evaluator. From
the repository root, `nix run ./infra/tooling#source -- bootstrap/source/valid/nested.cue`
prints `42`. It does not invoke Go at runtime. Whitespace, decimal, and ASCII
identifier scanning are compiled from the [scanner DSL](../scanner.md).
Decimal lexical validation, operand-newline policy, and diagnostic reasons
are also CUE-defined, along with parenthesis and bound/infix token recognition.
Checked i64 conversion, grammar validation/lookup,
and frame safety remain native. The [Pratt control DSL](../pratt.md) owns the
expression loop and associativity/binding-power decisions; its separate
[primary program](../primary.md) owns dispatch and grouping control. Existing JSON interfaces remain unchanged.
`bootstrap-scan-codegen <program.json> [lowering.json]` uses a
[shared CUE backend](../scan-lowering.md) for scanner source/branch layout;
checked flow/label analysis, cursor and pattern rendering remain native.
The [AST builder DSL](../ast.md) generates typed construction functions used
by the parser; `bootstrap-ast-codegen <builder.json> [lowering.json]` now uses a
[shared CUE backend](../ast-lowering.md) for Rust layout and clone/move selection.
Checked binding/type/use-count facts and the reference interpreter remain native.
`bootstrap-pratt-codegen <control.json>` lowers the expression control program.

## Deliberate limits

- Unification supports null, top, concrete i64/booleans/text, abstract `int`/`bool`/`string`, and inclusive
  integer-threshold bounds. Conflicts are errors, not first-class bottom
  values. Other types, structs, exclusive bounds, disjunctions, defaults,
  references, and full CUE unification remain unsupported.
  Runtime diagnostics are not upstream CUE diagnostics.
- Arithmetic requires concrete operands: `(int & 3) + 1` works, but `int + 1`
  cannot produce a concrete result. There is no backward symbolic propagation.
- Both documents require `formatVersion: 1`.
- Concrete numeric values, bound thresholds, and arithmetic results are signed
  64-bit integers. Out-of-range values, non-integer numeric fields, overflow,
  and underflow are errors. Addition also concatenates concrete strings without
  coercion; other arithmetic remains integer-only. Boolean arithmetic is rejected. No
  wrapping, saturation, or floating-point arithmetic. This is narrower than
  CUE's integer domain and is only a bootstrap limitation. Multiplying `i64::MIN`
  by `-1` is also an overflow error; a later multiplication by zero does not erase
  an earlier intermediate overflow.
- Unknown operations, unknown instruction fields, missing accessed fields, and
  missing rules are errors. The executor does not validate the complete AST
  against the CUE schema; it checks only fields that the chosen rule accesses.
- Inputs are ordinary JSON, subject to the JSON parser's nesting limit. This
  small executor is not a production-hardened runtime or a general rule engine.

## Checks

`shaka preflight` builds the package through `nix flake check`. Handwritten
code-generator seeds enable a [fresh two-stage build](../generated/README.md),
then the final package runs Rust formatting, Clippy, and all tests in the
sandbox. Tests use the real exported
fixtures, the changed-rule export, integer boundary/error cases, all nine
CLIs, the source parser, the scanner DSL, and compiled generated modules. The interpreter and generated CLIs
must agree byte-for-byte on stdout and stderr, and on exit status, for both
exported expressions, the complete program-case corpus, malformed JSON, and
missing program files. Argument errors are checked against each CLI's usage.
A separate Nix check verifies repeated Rust generation and committed-source
drift. The Rust toolchain is pinned through the tooling flake's Nixpkgs lock;
Rust dependencies are pinned by `Cargo.lock`. No additional workflow is needed.

For a quick local iteration with current CUE exports:

```sh
nix develop ./infra/tooling --command cargo test --manifest-path bootstrap/executor/Cargo.toml
```

The shell supplies `CUESTACEAN_BOOTSTRAP_JSON`; no working-tree JSON is needed.
When keeping an interactive shell open, re-enter it after changing CUE sources.
