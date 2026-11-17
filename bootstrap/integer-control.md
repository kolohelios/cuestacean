# CUE-owned checked integer dispatch

Negation, subtraction and multiplication now delegate accepted-domain and
operation selection to CUE control. Negation requires one concrete integer;
subtraction/product require two. Other values retain the Arithmetic error class.
Native borrowed-input kernels validate representations defensively and use
`checked_neg`, `checked_sub` and `checked_mul`, preserving all existing overflow
errors. Subtraction never negates its right operand first; MIN minus MIN is zero.
No language syntax, AST/rule instruction, precedence or value-domain expansion.

Both children still evaluate left-first before binary policy checks; unary child
errors precede negation checks. Abstract operands remain rejected, not propagated
or solved backwards. Signed MIN literals still parse directly without negating an
out-of-range positive literal. [Division/remainder control](division-control.md)
extends this boundary with CUE zero/unit-divisor policy over checked math.

## Finite control and compiler boundary

`integerPrimitive {operation}` selects the validated enum `negate`, `difference`
or `product`. CUE owns the mapping to fixed checked-kernel paths. Binary reference
execution routes typed primitives; the unary subset admits only negation plus
logical/arithmetic failure, and rejects binary/right dependencies even if unused.
There is no dummy RHS. New function enum tags select arity and compiled entry
symbols, not intrinsic semantics; normal CUE programs own their guards/results.
Production runtime facades use generated drivers under `semantics` with no native
operator-dispatch fallback. Compiler-only seeds retain only checked primitives.

Entry-name emission uses CUE's checked `identifier` with snake conversion after
native enum validation, replacing a growing literal spelling map. All accepted
control names remain enum-whitelisted; malformed wire names cannot become source.
Raw compiler metadata and implementation policies remain trusted native hooks,
not a sandbox. Kernel paths still come only from the fixed CUE primitive map.
Existing compiler/control budgets stay unchanged, and old evaluator/control
Rust should remain byte-identical except the expanded backend and new modules.

The same six artifact-free seeds generate all drivers from fresh build-only JSON;
committed Rust remains a drift baseline. Independent extrema/type/error-order
checks compare kernels, reference control and production. An actual compiled
alternate compiler maps unchanged subtraction control to multiplication, yielding
42 rather than 1 for 7 and 6. Pinned upstream/source checks continue. This is
progress in executable CUE ownership, not full solver/compiler closure or a
measured runtime performance claim.
