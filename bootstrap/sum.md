# CUE-owned scalar addition

The next binary-control slice makes `sum.cue` choose addition semantics: two
concrete integers use checked i64 addition; two concrete texts concatenate; other
pairs fail the existing Arithmetic error class. The diagnostic becomes
`arithmetic requires compatible concrete operands`, since addition is no longer
integer-only. Other arithmetic operations remain integer-only, and abstract
operands are still rejected rather than propagated or solved backwards.

Both evaluator paths evaluate the left child and then the right child before
invoking this policy. Child errors precede enclosing operand validation. Source
`+` keeps its existing precedence and associativity; JSON addition AST/instruction
wire forms are unchanged. String equality, identity and unification still belong
to the separate CUE unifier. No implicit string/numeric/boolean coercion is added.

## Reused finite control, checked kernels

The finite unification-control DSL is reused for binary value policies. An optional
validated `function` enum (`unify` by default, or `sum`) chooses only the compiled
entry-point name, not runtime behavior. CUE lowering selects that spelling; raw
metadata cannot inject identifiers. `integerSum` and `textConcat` expressions call
native borrowed-input primitives. Those primitives validate representations and
check integer/capacity arithmetic; they do not choose overloaded addition policy.
Text output is owned `Arc<str>`; Unicode is preserved without normalization.
The String buffer uses fallible reservation, but converting it to `Arc<str>` may
allocate again through the standard allocator. This remains prototype-level
allocation handling, not comprehensive OOM hardening.

`runtime::sum` uses the generated CUE sum driver under `semantics`, with no native
overload fallback. The reference control interpreter runs the same validated data.
The existing sixth compiler and shared CUE backend generate sum and unification;
no new compiler binary or generated seed dependency is introduced. Existing
512-node/64-level control guards and compiler rendering guards remain unchanged.

Independent tests compare normal and alternate compiled control, reference control,
evaluator APIs and source CLI, checking mixed domains, abstract values, ownership,
Unicode/empty strings, left-first errors and all i64 addition boundaries. Pinned
Go CUE scalar comparisons cover the admitted concatenation profile. A test-only
sum policy returns its right input to prove control data affects compiled behavior.
This does not add further string operations, complete CUE solving, bootstrap
closure, graph ownership or a measured performance claim.
