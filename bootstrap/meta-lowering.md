# CUE-defined lowering-driver compiler

`meta-lowering-body.cue` and the two canonical scalar alias sources
(`meta-text-alias.cue`, `meta-next-rule-alias.cue`) own the compiler instructions.
Their [native policy-data artifact](../docs/native-meta-lowering-policy.md) supplies
the canonical backend without Go construction; `meta-lowering-fragments.cue` is
the Go-only schema/required-slot bridge for independent validation and raw
contextual overrides. The compiler emits complete Rust drivers for the
[shared lowering DSL](lowering.md): header, dispatch layout, alias arms, expression
bodies, optional assembly method, module statements and footer. Native `Expr::emit`
and `module::emit` are removed. The compiled meta driver regenerates its whole
implementation byte-exact after pinned formatting. This is self-regenerating
source emission, not a fully self-hosted CUE parser/compiler or bootstrap closure.

## Bootstrap and native boundary

Six no-default-feature seeds build without generated Rust. The lowering compiler
seed interprets fresh natively constructed `meta-lowering.json`; parsing/interpreting that
policy does not recursively call the compiler. It generates `meta-lowering.rs`.
Production `compiled-lowering` uses that compiled driver for full-module emission,
including its own regeneration. JSON stays build-only; Rust is a drift baseline,
never a seed input. AST and meta are now two independently native policy-data
inputs; the shared six-compiler seed still depends on four other Go-constructed
policies. This does not establish native full-schema evaluation or Go-free seeds.

Typed/scoped validation, original-text structural checking, canonical metadata
projection, reference DSL execution and checked kernels remain native. The native
compiler projects data into one `driver` node: ordered cases, canonical-first and
sorted alias key lists, expression trees, module presence and statements. It emits
no Rust snippets. CUE chooses traversal, declaration and branch layout, method
spelling and all source composition; there is no native shell/module fallback.

Call arguments, concat parts and select entries carry role tags and child values.
Module text serializes as internal `moduleText`, avoiding expression-text ambiguity;
its original accepted `text` wire tag is unchanged and `moduleText` is rejected on
input. Absent versus empty module programs remain distinct. These are private
compiler metadata projections, not a new public policy wire language.

## Reuse and lexical rendering

Optional `aliases` map opcode strings directly to canonical cases. Cases plus
aliases total at most 64; keys use 1..64-byte bounds. Collisions, dangling targets,
chains/cycles, null maps and original-text duplicates reject even when unused.
Dispatch preserves the original node/opcode and kernel charges. CUE groups the
canonical then sorted alias keys into one match arm. The unchanged 512-value/32-level
policy guard still applies; the full default meta export uses 472 JSON values.

`identifier {field, snake?, upperFirst?}` validates a 1..64-byte ASCII component before writing:
first letter/underscore, remaining letters/digits/underscores. Snake mode lowercases
capitals and inserts underscores before every noninitial capital. Upper-first
mode changes only the initial byte, preserving the rest. Both flags default false
and cannot both be true; eager validation includes unused branches, and the
kernel defends the same invariant. [Identifier rendering](identifier-rendering.md)
recovers unification compiler headroom without native semantic spelling tables.
The CUE emitter supplies both kernel arguments and self-regenerates. Output streams
through shared budgets. This is lexical rendering, not keyword/name/type resolution
or arbitrary source injection. CUE supplies prefixes/syntax; Rust checks the result.

## API and whole-driver limits

`lowering::generate(policy)` uses the default meta compiler;
`generate_with_policy(policy, compiler)` and
`bootstrap-lower-codegen <policy.json> [compiler.json]` select a custom reference
compiler policy. Full target validation precedes compiler-policy validation.
Compiler overrides must now handle the complete `driver` metadata root: old
expression-only overrides fail, rather than receiving an implicit native shell.

One shared compiler Context covers the WHOLE driver: 4096 steps, 64 instruction
levels, 128 combined frames and 1 MiB cumulative rendered bytes including copies.
Selected strings remain bounded at 64 KiB. These limits now include shell and module
emission; there is no per-case reset or uncharged native assembly. Thus individually
compilable cases or statements can fail when combined. Parsing/metadata projection
are outside rendering accounting; this is not comprehensive hostile-input hardening.
Reference execution/assembly retain their separate bounds and are not narrowed.
CLI failures return no partial stdout.

## Evidence

All previous client/evaluator/frontend goldens remain byte-exact; the expanded
meta driver self-regenerates from fresh seeds and compiled production. An actual
compiled alternate meta policy changes unchanged keys-only module IR from next to
previous traversal: normal compiled/reference assembly emits both quoted keys;
the altered compiled driver emits only the first. Expression lowering is unchanged.
The earlier forced-expression adversary also remains covered. Tests pin whole-driver
case/module budget accumulation, eager validation, absent/empty modules, lexical
safety, escaping and no native fallback. Shared kernels are not independent semantic
implementations; performance remains unmeasured and full CUE support is not claimed.
