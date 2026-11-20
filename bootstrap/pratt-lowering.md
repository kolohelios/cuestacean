# CUE-owned Pratt compiler backend

[`pratt-lowering-fragments.cue`](pratt-lowering-fragments.cue) and the authoritative
`pratt-lowering-tables.cue` describe how Pratt/primary control becomes Rust. The
[native table module](../docs/native-map-comprehensions.md) evaluates the original
finite MinimumCalls/SelectorCalls maps and three comprehensions. The
[complete native DATA path](../docs/native-pratt-lowering-policy.md) combines that
unchanged module with `pratt-lowering-body.cue` and `pratt-add-policy.cue` under the
existing bounds. Go remains the schema/raw-refinement oracle; no-increment loads
all three unresolved owners before context. Full native policy DATA is not native
schema evaluation or isolated Pratt compiler construction. It also emits the
parser-driver wrapper. `pratt.rs` no longer has per-opcode emission matches or
hardcoded generated-source boilerplate. This is distinct from `pratt.cue` and
`primary.cue`, which describe runtime parser behavior.

The [expression-list target](list-expressions.md) now shares this compiler:
validated target metadata selects CUE-owned Parser Frame versus ListFrame headers.
Checked method names and item-kind spelling are emitted in CUE; native code owns
only typed family validation, canonical facts and frame/reference kernels.
[Conjunction](conjunction.md) adds a fourth header for a transactional reduction
frame; seed/merge/finish are target-specific, append/reject stay list/record-only.
[Optional slicing](optional-slices.md) adds a fifth header and presence/default
frame; item operations stay confined to collection/reduction targets. Native typed
validation and injected unifier/slicer keep compiler-only kernels independent
of generated semantics, without a fallback. Cross-family operations reject before
even a backend that ignores them. Old parser
programs default to parser and regenerate identical Rust.

The source-list extension factors validated unit statements into canonical
`{op:"action",action:<wire-name>}` metadata. Return remains separate. CUE selects
all method names, charging and layout; no native emitter/path is introduced.
Wire `action` tags/fields reject. This keeps 31 top-level cases (rather than 70)
under the unchanged 64-case limit; nested method selection is bounded too.
Selectors would expand the unit-method map to 65, so validated selector statements
use separate `{op:"selectorAction",action:<wire-name>}` metadata. That revision
used 58 ordinary/7 selector method arms and 497 values. [Source slices](source-slices.md)
replace ordinary method selection with CUE checked snake identifier rendering of
the validated canonical action name, retaining seven explicit selector methods.
Seven parse-with-minimum calls share one CUE template via existing one-hop aliases.
Actual prior full/no-selector/no-index driver sources stay byte-identical. Current
backend is 370 original values after optional-slice control (336 at source
conjunction, 333 at the reduction header, 330 at source slices), below unchanged
512/64 guards. Internal action/
parseMinimum tags and extra fields still reject eagerly on wire; no native source
emitter or token/type-policy fallback is introduced.

Both backends use the [same lowering compiler/kernel](lowering.md): evaluator
instruction lowering and parser-control compilation are separate CUE policies,
not separate handwritten generators. The compiler count remains six.

## Generic expression operations

Five reusable operations extend the existing expression vocabulary:

- `text {value}`: trusted fixed target-source text, at most 64 KiB.
- `concat {parts}`: ordered concatenation, including a valid empty sequence.
- `unsigned {field}`: project a u64 and stream its exact decimal spelling.
- `list {field}`: require an array, recursively lower each child using the same
  selected driver, and concatenate results with shared work/depth/output budgets.
- `ifEmpty {field, then, else}`: require an array and evaluate only the selected
  branch. Both templates validate before any execution/compilation.

Source text is trusted implementation code, not instruction-derived data and not
restricted like `path`. Dynamic string values retain Rust quoting; unsigned
values cannot inject syntax. The existing generic structural, selector, work,
combined-stack and rendered-byte limits apply unchanged. These are bootstrap
bounds, not comprehensive hostile-input hardening or a sandbox for native drivers.

## Bridge and bootstrap

Native code validates complete typed control IR, retains the reference parser
interpreter and serializes the statement body as a tagged `program` node.
[Early return](parser-return.md) adds checked canonical root `tail` and return
`position` facts, never accepted wire fields. CUE chooses charged explicit-return
or tail-expression source and the wrapper footer, with no lint suppression or
native Rust emitter. Both-arm flow facts avoid unreachable/needless-return code;
loops do not inherit function-tail context.
Original-text typed decoding now rejects duplicate control fields rather than
silently accepting a generic JSON map's overwritten value. The existing control
JSON structural guard is unchanged; it is not the policy's streaming checker.

`pratt::generate` uses the compiled CUE backend under `compiled-lowering`.
Compiler-only seeds interpret freshly exported `pratt-lowering.json` instead.
`pratt::generate_with_policy` and the optional second argument to
`bootstrap-pratt-codegen <control.json> [lowering.json]` select a reference backend.
Typed validation still precedes backend execution even when a backend ignores
some input fields. The final compiled compiler reproduces all existing Pratt and
primary driver goldens, including alternate/error/fuel policies.

The native policy compiler, decoder/serializer, generic collection traversal,
rendering primitives and guards remain Rust; neither compiler closure nor full
CUE compatibility is claimed. The six fresh seeds do not depend on committed
Rust, JSON stays build-only, and compiled-backend-only/semantics-only profiles
remain checked with generated dependencies removed where appropriate.

## Evidence

`pratt-lowering-no-increment` selects a different `add` compiler case while
leaving the input parser-control program unchanged. The resulting compiled parser
driver nests addition to the right: `MAX + 1 + -1` succeeds with MAX instead of
failing at the normal left-first intermediate overflow. This is a deliberate
alternate policy, not the default source language.

Known source/AST/error expectations and the full compiled/reference parser corpus
remain in force. New tests compare both backend engines, generic primitive
composition/type errors/unused branches/shared limits, original control decoding,
CLI overrides and failures. An actual compiled adversarial collection policy
exercises escaped selectors/opcodes, text, empty concat, list dispatch and u64
extrema rather than merely checking emitted strings.
