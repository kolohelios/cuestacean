# CUE-owned concrete-output admission

`json-output.cue` and `integer-output.cue` choose accepted scalar domains and
incomplete-value diagnostics through genuine unary CUE control. JSON admits
null, integers, booleans and owned text. Integer-only output admits integers;
other concrete scalars keep the IntegerOutput error. Top, int, bool, string and
bounds retain their distinct incomplete errors before either output conversion.
Singleton bounds stay incomplete. No source syntax, solver or output behavior
expands; exact i64 and JSON string/null/bool handling remain unchanged.

Production `runtime::concrete` / `concrete_json` invoke generated admission, then
native `encode_integer` / `encode_json`. Checked encoders perform representation
extraction and JSON allocation only; unsupported values produce the defensive
`OutputPolicy` error rather than reimplementing incomplete-domain policy. Default
CUE rejects such inputs first. Encoders remain available to compiler-only seeds;
production facades and generated admission are semantics-gated without stubs.

The finite function enum adds unary `integerOutput` / `jsonOutput`. Failure tags
add `incompleteInt`, `incompleteValue`, `incompleteBool`, `incompleteString`,
`incompleteBounds` and `integerOutput`, all input-independent and thus allowed
in unary control. Existing pair-dependent failures, endpoint conditions, binary
operations and right references still reject even in unused branches. The typed
bridge uses the existing serialization-only unaryFail tag, not new wire syntax.
CUE explicitly maps incompleteInt to the legacy Error::Incomplete spelling.

Normal control returns the selected input through existing explicit Clone
lowering; owned text retains its Arc allocation until native JSON string copying.
There is no clone-free/performance or comprehensive allocation-failure claim.
Entry metadata selects arity/name, not intrinsic semantics: an actual compiled
alternate returns integer zero for a boolean input, yielding JSON 0 rather than
default false through the same checked encoder. Fixed domain/error/ownership and
CLI/evaluator tests supplement that proof and existing pinned Go CUE comparisons.

Six artifact-free seeds, fresh build-only JSON, committed Rust drift and all
resource guards remain. Native numeric/representation/serialization kernels and
validation still exist; this is CUE admission policy, not full compiler/solver
bootstrap closure or a CUE implementation of JSON encoding.
