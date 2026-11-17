# Composable scanner predicates

The scanner DSL adds pure `and {left,right}` / `or {left,right}` predicates.
Reference execution and generated Rust use ordered short-circuit boolean reads;
there are no predicate side effects, new cursor operations or general calls.
Validation checks both children even when execution would ignore one. Existing
leaf syntax and old scanner output remain compatible.

Statement limits stay 64 nodes/16 nesting levels. Predicate work gains separate
whole-program bounds of 128 predicate nodes and 16 predicate levels, root at
level one. Existing programs have at most one leaf predicate per conditional
statement and stay within those bounds. Byte/character lists and lookahead keep
their existing caps. The surrounding if/while test still charges one runtime
step, not one per predicate node; bounded pure trees bound each such read.
Existing input-length fuel, cursor/UTF-8 and shared compiler limits are unchanged.

Native typed validation/reference evaluation and canonical metadata remain.
Metadata supplies the desired inversion bit to each child; CUE chooses `&&`/`||`
and De Morgan inversion, parentheses and branch layout. No native predicate-to-
Rust emitter is introduced. Both normal/inverted metadata trees validate before
rendering, and original unknown/duplicate fields still reject.

The Unicode string control replaces nested prefix tests with composition,
recovering statement headroom without changing accepted spans, diagnostics or
flag lifecycle. A committed compiled copy of the prior nested program preserves
an independent before/after control baseline. A dedicated compiled predicate
fixture exercises nested conjunction/disjunction and inversion across bytes,
Unicode/EOF/flags; fixed oracle and structural-budget tests complement parity.

Six artifact-free seeds, build-only JSON and committed Rust drift remain. This
is executable control/compiler capacity work, not new source string syntax,
full compiler bootstrap closure or a performance claim.
