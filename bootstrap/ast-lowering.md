# CUE-owned AST-builder compiler backend

[`ast-lowering-fragments.cue`](ast-lowering-fragments.cue) owns typed builder
signatures, expression/key layout, JSON constructors and clone-versus-move
selection. The unchanged integer-literal choice has its canonical raw owner in
[`ast-integer-policy.cue`](ast-integer-policy.cue), with a required provider slot
in the remaining policy. Its [native artifact](../docs/native-ast-integer-policy.md)
feeds the canonical default policy; Go CUE still assembles/validates that complete
policy. Override fixtures load the raw choice before refinement, not the concrete
default artifact. No default semantics or compiler wire format is widened. It uses the same [lowering compiler/kernel](lowering.md) as evaluator,
Pratt and unification generation. Six native compiler seeds remain; no new
compiler binary is introduced.

## Native facts, CUE decisions

The existing AST validator still checks parameter uniqueness/types, references,
string keys and the 16-parameter/64-expression/16-level limits. A native checked
pass visits fields in key-then-value order and annotates:

- positional parameter index/type, never a Rust snippet or user-named identifier;
- key versus value role for string/argument expressions;
- remaining uses after consuming an argument occurrence.

CUE decides which types copy, when owned arguments clone or move, how keys differ
from JSON string values, and how objects/signatures are assembled. Strings/JSON
clone while later uses remain and move on their final use; i64 copies even when
reused. Field order and last-field-wins semantics remain unchanged.

Native binding/use-count analysis is still an algorithmic boundary, not claimed
self-hosted. Policies must respect the facts' traversal order; arbitrary trusted
policy changes may produce invalid ownership/type behavior rejected by Rust.
The reference construction interpreter remains native. AST wire syntax is unchanged;
the annotated tagged tree is an internal compiler representation, not a new public
builder language.

## Reusable primitives and guards

- `signed {field}` streams an i64 decimal, including MIN/MAX, through the byte
  budget; numeric data cannot inject target syntax.
- `join {field, separator}` lowers array children using the same selected driver
  and shared work/depth/output budgets. The trusted separator is at most 128
  bytes and is charged before the next child's dispatch. Empty joins emit nothing.
  Existing `list` is exactly empty-separator join, without another frame/step.
- `ifZero {field, then, else}` requires a u64 field, runs only the chosen branch,
  but validates both templates before lowering/compilation.

These operations preserve existing kernel limits and intermediate-copy accounting.
Compilation now also has the shared string/work/output bounds; this is bounded
bootstrap machinery, not comprehensive hostile-input hardening or a sandbox for
native drivers. Trusted policy text differs from quoted input strings.

## Pipeline and evidence

`ast::generate` uses `compiled_ast_lowering` by default. Seeds interpret fresh
`ast-lowering.json`; `generate_with_policy` and the optional second CLI argument
select a reference backend after complete builder validation. Generated Rust and
all existing builder signatures/goldens are unchanged. Six fresh seeds delete
staged Rust first; semantics-only checks remove this compiled backend too.

Existing compiled builder, independent JSON and full parser tests still exercise
actual source semantics, escaped metadata, duplicate keys and repeated ownership.
New tests pin cloned-then-moved arguments, repeated integer copies, deepest valid
builders, 63-node trees beneath the 64-node cap, joined signed extrema, lazy branch
selection, malformed/unused templates and CLI failures without partial stdout.

A deliberately altered integer compiler case emits constant 99 for the same
literal-7 builder input. An actual compiled `ast-forced-integer.rs` returns 99,
while normal reference construction returns 7. This proves compiler-policy effects,
not a production language change or full generator bootstrap closure.
