# CUE-owned logical value selection

`logical-and.cue` / `logical-or.cue` implement `&&` / `||` through shared finite
binary control. Both children evaluate left-first before policy selection. Both
values must be concrete prototype scalars (null, i64, bool or text), and the left value
must be boolean. If the left determines the result, CUE returns that boolean
without requiring a boolean right value; otherwise the right must be boolean.
Thus `false && null` is false and `true || null` is true. Top remains abstract.
Likewise `false && 1` is false and `true || "x"` is true, but `true && 1`,
`false || "x"`, `false && int` and `true || (1 div 0 == 0)` fail. These are
pinned Go CUE observations, not Rust-style lazy child evaluation or coercion.
Abstract values/bounds and invalid selected types report `invalid logical operands`.

CUE owns concreteness guards, boolean selection and result construction. The
only new native primitive, `boolean_is`, checks boolean representation and
compares a literal bool. Enum-validated function metadata maps to fixed Rust
`logical_and` / `logical_or`; facades delegate generated policies under `semantics`.
The rule interpreter and CUE rule lowering remain eager and ordered. No native
logical overload fallback, new compiler, callbacks or memoization are introduced.

Source uses existing binary builders and Pratt control. Default precedence is
`&` 5, `||` 6, `&&` 7, comparisons 8, addition/subtraction 10, multiplication 20.
Comparison metadata moves from 7 to 8 to leave two distinct logical levels;
relative precedence of all previously supported operators is unchanged. Existing
grammar entry order is preserved. CUE scanning recognizes `&&` before single `&`
and only complete `||`, with guarded flag-based progress to prevent token merging.
Single `|`, disjunction/defaults and reference/structural logic remain unsupported.
[Unary `!`](logical-not.md) now has separate one-operand CUE control. Parser/scanner/control/compiler budgets are unchanged.

Independent truth/type/abstract/error-order oracles, source AST/precedence/token
checks, CLI atomicity, actual compiled alternate control and pinned Go CUE probes
exercise normal and reference execution. Boolean outputs compare byte-exactly.
Six artifact-free seeds still regenerate from fresh build-only JSON; Rust remains
committed for drift. This is not full CUE support, general solving, self-hosting
closure or measured performance.
