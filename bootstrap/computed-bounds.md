# Computed inclusive integer bounds

Issue #113 adds parenthesized expression thresholds: `>=(1 + 2) & 3` and
`<=(0x20 div 2) & 16`. Literal threshold syntax and its scalar-payload AST stay
unchanged. The new `lowerBoundExpression` / `upperBoundExpression` kinds carry
one `operand` expression, sharing the existing closed unary shape. Their CUE
evaluation rules evaluate the child before calling existing unary bound control.
Thresholds must evaluate to concrete checked i64s; there is no backward
propagation, deferred reference evaluation or wider numeric/string-bound domain.
Child errors precede threshold admission. Bounds remain inclusive/infinite and
incomplete even when singleton. Existing integer arithmetic/overflow applies.

CUE primary control selects a threshold group after committing a bound prefix,
using existing scanGroup/groupOpen/advanceGroup/parseGroup/closeGroup operations.
The threshold group's recursive minimum is CUE-supplied zero, propagating both
selected drivers. No new parser/compiler opcode or native dispatch fallback is
needed. Native private stages distinguish ordinary and bound groups, preserve
the bound constructor index, and reject out-of-order/unfinished/cross-phase use.
Closing a bound group adds one node through the existing CUE unary builder.
Literal fallback remains available from the original bound-advanced stage too,
so older primary policies remain valid. Grammar bound entries gain optional
`expression: {kind, operandField}` metadata; absent metadata disables grouped
thresholds for that prefix. Kinds/fields remain grammar-selected and validated.

Only parenthesized expression thresholds are added: bare bounds remain literal,
not a new expression-prefix precedence rule. `>=1 + 2` retains its old parse and
arithmetic rejection. Resource limits remain 64 AST nodes/32 expression levels,
4096 shared frame steps and all existing scanner/compiler guards. A threshold
group adds expression recursion and its bound wrapper adds one AST node, unlike
ordinary grouping. No graph/reference ownership or default/disjunction design
is implied.

Evidence includes fixed full AST/exact values, eager child errors, domain and
incomplete results, legacy/custom/disabled grammar compatibility, private stage
and unchanged resource guards, and actual compiled/reference alternate CUE
threshold-group minimum. Pinned upstream source comparisons and six fresh
artifact-free seeds/repeated Rust drift remain required. JSON stays build-only;
Rust stays committed, never a seed dependency. Native checked frames, grammar
lookup/validation, value representation, numeric and output kernels remain
explicit boundaries. This is not full CUE compatibility/bootstrap closure or
measured performance.
