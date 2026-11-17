# CUE-owned unification compiler backend

[`unify-lowering-fragments.cue`](unify-lowering-fragments.cue) owns all emitted
unification-control Rust: value/endpoint projections, boolean conditions, branch
layout, failure constructors, lexical tuple bindings and the function wrapper.
It reuses the same [lowering compiler/kernel](lowering.md) as evaluator and Pratt
compilation; the bootstrap compiler count remains six.

Native `unification.rs` still decodes and validates the whole original control
program, exposes checked value primitives, and interprets solver control for
reference tests. Its per-opcode Rust emission and enum-to-source mapping methods
are removed. This is not full compiler self-hosting or a new CUE language feature.

## Selection and the typed bridge

The generic `select {field, cases}` expression projects a string field and runs
exactly one case through the current driver. Missing/wrong-type fields and
unmatched tags fail. All branches validate before use, including unused ones;
case-map size and key limits match root policy maps (1–64 cases, 1–64 byte keys).
Selectors remain 1–128 bytes. Original-text decoding rejects duplicate nested
case keys after the streaming policy guard counts every original value.

Selection consumes one expression frame/step and charges the chosen intermediate
copy. Branch code and emitted string patterns use Rust escaping. Selected data
only chooses trusted expressions; it never becomes a raw Rust path or snippet.
The shared work, stack and output budgets are unchanged.

Two control expression families originally use `op: "input"`. The typed bridge
serializes these internally as `inputValue` versus `inputEndpoint` so a single
backend can distinguish them. The original wire still accepts only `input`;
serialized-only aliases are not new control syntax. Native enums serialize side,
endpoint, kind, relation and failure tags as data; CUE's select maps own their
Rust spelling. The bridge always supplies a validated `function` enum (`unify`,
`sum`, comparison/logical and integer-control names), defaulted for legacy
control wire documents. CUE's checked `identifier` with snake conversion emits
the compiled entry name and emits the `integerSum`/`textConcat` kernel calls used by
[addition control](sum.md). It also emits boolean result construction for
[equality control](equality.md), mapping `notEqual` to the fixed `not_equal` name.
[Ordering control](ordering.md) adds four enum-validated names and a `textCompare`
condition; CUE also maps inclusive relation names to fixed Rust variants. Native
finite/text comparison kernels do not choose allowed source domains.
Logical control adds two fixed entry names and `booleanIs` literal/side projection.
Unary logical control adds root `unary` arity facts and serialization-only
`unaryFail` for an already validated logical/arithmetic failure. CUE chooses a one-input
header and emits failure without a nonexistent RHS. Original wire still uses
`fail`; the unary subset rejects all binary/right dependencies before lowering.
Reusable binary CUE templates explicitly use `#BinaryControlProgram`.
The integer primitive's validated operation tag selects fixed checked-kernel
paths in CUE. Function identifiers originate only in the native whitelist; raw
compiler metadata and policies are trusted hooks, not an arbitrary-code sandbox.
Malformed control-wire names still reject before any rendering. These are not
unchecked user-supplied identifiers or paths. Complete typed/lexical validation precedes any backend execution.

`letBounds` still emits a simultaneous tuple initializer: both initializers see
the outer `_lower`/`_upper` binding, then the result sees the new pair. Conditional
branches and control-condition `and`/`or` retain their order and lazy behavior.
This is not source `&&`/`||` child laziness: logical instructions evaluate both
children before CUE binary policy runs. The
existing compiled scoped fixture and meet-law/error tests remain independent
checks, not merely two engines agreeing on source text.

## Bootstrap and evidence

`unification::generate` uses `compiled_unify_lowering` by default. No-default
seeds interpret fresh `unify-lowering.json`; explicit `generate_with_policy` and
`bootstrap-unify-codegen <control.json> [lowering.json]` also use the reference
backend. The final compiled compiler matches reference-generated unification goldens.
Owned values now use borrowed inspection and CUE-emitted clones for selected inputs.
The feature-isolation removal set includes this backend along with Pratt and
evaluator drivers. JSON remains build-only, Rust committed, and seeds never read
committed generated Rust.

A deliberately permissive compiler policy changes `same` to true while leaving
a focused boolean/integer control probe unchanged (not the growing production
solver). Its actual compiled unifier accepts
opposite booleans, while mixed-domain/unequal-integer conflicts remain. This is
an alternate compiler experiment, not production semantics or a compatibility
claim. An adversarial compiled select policy checks escaped selectors/tags,
nested driver recursion, lazy selection and exact errors.

The existing control structural guard still inspects a generic JSON value before
original-text typed decoding. It is not a streaming count of overwritten control
values; do not confuse that pre-existing prototype boundary with the policy's
streaming guard or comprehensive hostile-input hardening.
