# CUE-hosted scalar unification

[`unify.cue`](unify.cue) owns the current value-unification algorithm: abstract
`string`, `bool` and `int` identity, concrete equality/conflict, mixed-domain rejection,
interval intersection, infinity handling, empty-range rejection and concrete
candidate selection. Production
`runtime::unify` calls generated Rust, not a native intersection fallback.
Both rule-interpreted and rule-generated evaluators use this same policy.

This is an executable solver boundary, **not full CUE unification or bootstrap
closure**. Values are the native owned `Text(Arc<str>)`/`String`, `Boolean(bool)`/`Bool`,
`Integer(i64)`/`Int` and inclusive `Bounds` scaffold; `Value` is no longer Copy.
See the [ownership design](../docs/string-values.md). Projection, tag tests, finite i64
comparisons, error construction, checked arithmetic, concrete output, typed
decoders and generic compiler/interpreter kernels remain Rust. Per-opcode
[compiler lowering](unify-lowering.md) now lives in a separate CUE policy. Go CUE still exports specifications. This policy
does not define accepted source syntax.

## Finite typed control

`#UnifyProgram` version 1 contains a `result` expression. The same finite binary
control now implements [addition](sum.md): optional `function` is the validated
compiled entry-name enum `unify` (default), `sum`, `equal` or `notEqual`, not a runtime
behavior mode. [Scalar equality](equality.md) adds `boolean {value}` construction
and `comparison` failure; CUE owns concrete-domain guards and result selection.
[Ordering](ordering.md) adds four validated entry names, `textCompare` conditions,
inclusive finite relations and `ordering` failure. Existing unification semantics
and missing-endpoint guards remain unchanged.
`integerSum` and `textConcat` call checked borrowed-input kernels; `fail` also
supports `arithmetic`. The interpreter executes `result` independently of the
entry-name metadata. There is no native overloaded-addition fallback. Value expressions
select an input, branch with `if`, bind endpoints with `letBounds`, construct
bounds, or fail with string, boolean, type, integer or bounds conflict errors. Endpoint
expressions project an input endpoint, read a lexical local, produce `empty`,
or `choose` conditionally. Conditions test kinds, endpoint presence, finite
comparisons, same-value equality, or short-circuit `and` / `or`. Boolean
identity/equality follow the string-domain guards; all mixed-domain rejection
precedes numeric identity. Inspection primitives borrow inputs; selected results
clone explicitly via CUE compiler policy. Text equality is content-based with no
normalization. Projecting a string/boolean endpoint yields none; that absence does not
mean the value is accepted as numeric.

There is no native optional-endpoint maximum/minimum operation: CUE explicitly
chooses present endpoints and supplies their negative/positive infinity policy.
The comparison primitive rejects missing endpoints rather than ordering them.
Bounds alone, including singleton ranges, stay incomplete; a valid concrete
candidate stays concrete. Missing endpoints are not clipped to i64 extrema.

Both `letBounds` initializers see the outer scope; its result sees the newly
bound pair. The compiler uses simultaneous tuple binding to preserve this rule
under nested shadowing. Locals are validated even in unselected branches.
`bootstrap-unify-codegen <control.json> [lowering.json]` lowers expressions to
real Rust branches/blocks through the shared CUE backend. The reference interpreter executes the same validated data.

There are no loops or recursive calls in this DSL, so it needs no fuel counter.
Documents are limited to 512 JSON nodes and 64 JSON levels, counting the root
as level one. Original-text typed decoding rejects duplicate/unknown fields;
these limits are bootstrap checks, not comprehensive hostile-input hardening.

## Bootstrap and evidence

Default `source` enables `semantics`; `semantics` makes evaluation and generated
unification available without the parser. Compiler-only `--no-default-features`
excludes evaluation entry points and generated semantics entirely. Nix deletes
staged Rust/JSON, builds six native compiler/VM seeds in that profile, regenerates
all modules, and then compiles/tests the normal frontend. There is no seed stub
or handwritten unification fallback. Generated JSON remains build-only; generated
Rust remains committed and checked for deterministic drift. Seeds interpret the
CUE compiler policy; production uses its compiled driver under `compiled-lowering`.

Independent expected values/errors cover identity, conflicts, interval shapes,
infinities, extrema and incompleteness. Sampled meet laws and three-way parity
compare the interpreter, generated control and production dispatch. Existing
source differential checks still compare the production pipeline with pinned
Go CUE. `unify-right` deliberately returns the right input, proving policy is
executable; `unify-scoped` checks compiled lexical shadowing and short-circuiting,
not valid unification semantics. Validation, structural boundaries and CLI
failure behavior are tested too.

```sh
nix develop ./infra/tooling --command cargo test --manifest-path bootstrap/executor/Cargo.toml --test unification
```
