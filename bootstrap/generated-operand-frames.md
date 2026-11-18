# Generated wide-operand frame layout

## Bounded representation/layout contract

The optional-endpoint slice experiment (#158) exposed a compiled debug-build stack
overflow at the existing 64-active-AST boundary while reference evaluation passed.
The generated evaluator's common evaluate_node match retained debug stack slots
for owned operands from unrelated arms. This preparatory change isolates the
existing ternary listSlice and binary unify expressions in immediately invoked,
CUE-emitted closures. They reborrow the same EvaluationContext, evaluate the same
children once in order and return the same Result. No AST entry, callback, default,
public API, value representation, instruction or compiler/seed is added.

Instruction lowering/layout remains CUE-owned. The closure syntax is emitted by
the existing generic lowering language, not a native emitter or policy fallback.
One logical-or layout joins existing binary aliases to retain the unchanged 512
policy guard; all other compiler/source/AST/value limits remain. The generated
source deliberately changes for the two isolated arms; the logical-or alias keeps
its exact prior rendering. Existing error/selection/upstream tests and new nested
63/64 wrapper tests retain success at 64 entries and depth rejection at 65.

No stack override, larger guard or test weakening. This is a local debug-layout
repair on validated x86_64-linux, not a comprehensive native-stack guarantee or
measured performance claim. Full pinned preflight/five actual sandbox checks/six
artifact-free seeds/profiles/generation+drift/strict Clippy remain required.
