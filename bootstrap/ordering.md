# CUE-owned scalar ordering

`less`, `lessEqual`, `greater` and `greaterEqual` compare two concrete i64s or
two concrete strings. Shared CUE control chooses matching domains and boolean
results. Booleans, mixed domains, abstract types and bounds fail
`ordering requires matching concrete integers or strings`. Text uses lexicographic
UTF-8 order (equivalent to scalar-code-point order for valid UTF-8), not locale,
natural-number sorting or Unicode normalization. No coercion or propagation.

Both children evaluate left-first before policy checks. Source `<`, `<=`, `>`
and `>=` share equality's precedence 7 and left associativity. CUE infix control
consumes an optional adjacent `=` after `<` or `>`; it leaves stray trailing
symbols for ordinary parser errors. Existing prefix `>=`/`<=` bounds remain
unchanged and abstract; bare prefix `<`/`>` do not add exclusive constraints.

## Primitive and compiler boundary

The finite binary-control compiler gains four enum-validated entry names and a
`textCompare {relation}` condition. The finite relation enum adds `lessEqual`
and `greaterEqual` for both existing endpoint and new text comparison primitives.
CUE guards integer operands before projecting finite endpoints, or text operands
before borrowed text comparison. The native primitives check representations and
compare values; they do not select accepted source operand domains. Missing
endpoints still reject rather than acquiring an ordering/infinity policy.

CUE selects fixed Rust entry/relation spellings and result branches. Runtime
facades delegate generated policies under `semantics`, with no native overload
fallback. JSON AST/instruction kinds correspond to the four operators. The same
six compiler seeds regenerate drivers and evaluators from fresh build-only JSON,
without committed Rust inputs. Existing control/compiler/parser/scanner budgets
remain unchanged. Equality and unification behavior are not broadened.

Independent truth/boundary/Unicode/domain/error oracles, actual compiled alternate
control, source AST/precedence and prefix-bound regressions, CLI atomicity, and
pinned Go CUE source/JSON comparisons establish the admitted slice. Boolean output
is compared byte-exactly. This is not general CUE comparison, structural/reference
support, complete solving, bootstrap closure or a performance claim.
