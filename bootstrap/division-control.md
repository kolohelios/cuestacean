# CUE-owned integer division and remainder policy

The four existing operators now use finite CUE control for concrete-integer
admission, zero-divisor rejection, primitive selection and the unit-divisor
remainder case. `div`/`mod` select checked Euclidean primitives; `quo`/`rem`
select checked truncating primitives. Rust retains those numeric computations,
representation checks and defensive zero checks. This is CUE-owned orchestration,
not a CUE implementation of machine division itself.

CUE returns literal integer zero for `mod`/`rem` with divisor -1, including
MIN/-1. Raw checked remainder primitives deliberately retain their overflow result
for MIN/-1, so this exception belongs to CUE rather than a native fallback.
Quotients still overflow there. No `abs(MIN)`, floor-division substitution,
wrapping or intermediate-error suppression is introduced. Both children evaluate
left-first before operand/zero checks; public values, diagnostics, AST/rule wire,
source syntax and precedence remain unchanged. Abstract operands still reject.

## Literal and compiler boundary

The finite control DSL adds checked-i64 `integer {value}` literals in value and
endpoint positions. The typed bridge serializes these as `integerValue` and
`integerEndpoint`; those aliases are not accepted control wire. Values construct
`Value::Integer`; endpoints construct finite `Some(i64)`. Missing endpoints still
reject comparison, never acquiring an ordering or infinity policy. Unary control
may construct an integer but cannot inspect endpoints or use binary primitives.

The typed integer primitive/function enums gain four division operations, and
binary failure gains `divisionByZero`. CUE maps primitive tags to fixed kernel
paths and emits literal/branch layout. A compacted conditional header preserves
all existing function bytes while staying below the unchanged policy guard.
Native code still validates complete original-text IR, arity, scope and guards.

Six artifact-free seeds and the shared compiler regenerate four drivers and an
actual compiled alternate: unchanged Euclidean division control is compiled to a
truncating kernel, yielding -2 rather than -3 for -7,3. Independent sign/extrema,
zero/type/order, literal-role and unit-divisor checks complement existing source
and pinned Go CUE comparisons. Generated JSON stays build-only and Rust remains
committed for drift. No budgets are raised, no native dispatch fallback is kept,
and no full CUE solving, bootstrap closure or runtime performance claim is made.
