# CUE-defined lowering-expression emitter

`meta-lowering{,-fragments}.cue` emits Rust calls for every expression in the
[shared lowering DSL](lowering.md). Native `Expr::emit` is removed. The default
compiled meta driver can regenerate its own expression implementation, byte-exact
after pinned formatting. This is emission-layer self-regeneration, not a fully
self-hosted CUE parser/compiler or complete generator bootstrap closure.

## Bootstrap and native boundary

Six no-default-feature seeds still build without generated Rust. The lowering
compiler seed interprets fresh Go-CUE-exported `meta-lowering.json`; parsing that
policy and interpreting expressions does not call the compiler recursively.
The seed generates `meta-lowering.rs`. Production `compiled-lowering` then uses
that compiled driver for expression emission, including its own regeneration.
CUE JSON stays build-only; Rust stays a drift baseline, never a seed input.

Typed/scoped validation, original-text structural checking, reference DSL execution,
checked kernels, outer driver header/footer/case assembly, alias grouping, and
`module::emit` remain native. In particular module-statement compilation has NOT
moved into CUE in this slice. No native per-expression source fallback remains.

The validated typed expression serializes into an internal compiler tree. Ordered
call arguments, concat parts and select entries receive role tags and child values;
these are data projections, not source snippets. Original accepted expression wire
syntax is unchanged apart from the new optional identifier operation.

## Reuse without raising policy bounds

Optional `aliases` map opcode strings directly to canonical cases. Cases plus
aliases total at most 64; keys use existing 1..64-byte bounds. Collisions, dangling
targets, alias chains/cycles, null maps and original-text duplicate keys reject,
even when unused. Dispatch preserves the original node/opcode and charges the same
kernel work as direct dispatch. Generated matches group canonical then sorted
alias keys into one arm. This lets the meta policy share method templates within
the unchanged 512-JSON-value/32-level guard; its default export uses 354 values.

`identifier {field, snake?}` validates a 1..64-byte ASCII identifier component
before writing: first letter/underscore, remaining letters/digits/underscores.
Optional snake mode lowercases ASCII capitals and inserts an underscore before
every noninitial capital. Output streams through shared budgets. This is a lexical
component primitive, not Rust keyword/name/type resolution or arbitrary source
injection. CUE chooses where to use it and supplies fixed prefixes such as
`_render_`; downstream Rust still checks the generated program.

## API, limits and evidence

`lowering::generate(policy)` selects the default meta emitter;
`generate_with_policy(policy, compiler)` and
`bootstrap-lower-codegen <policy.json> [compiler.json]` select a custom reference
compiler policy. Complete target validation precedes compiler-policy validation.
Each top-level target expression has a fresh shared compiler context: 4096 steps,
64 instruction levels, 128 combined frames and 1 MiB cumulative rendered bytes.
Selected strings are bounded at 64 KiB. These are additional emission limits;
they do not narrow reference execution of an otherwise valid target policy.
The native surrounding shell/module emission is not charged to this context;
do not describe the compiler as globally bounded to 1 MiB or fully hardened.

Existing backend modules remain byte-exact. Fresh sandbox seeds regenerate them,
then final compiled generators repeat/drift-check the meta driver too. A deliberate
alternate meta policy compiles unchanged `lowering-numbers.json` so its actual
compiled nonzero branch returns `forced` instead of the original interpreter's
`more`. Tests independently pin templates, alias semantics, lexical safety,
escaping, self-regeneration and CLI no-partial-output behavior. Shared kernels
are not independent semantic implementations; performance remains unmeasured.
