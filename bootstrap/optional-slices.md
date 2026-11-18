# CUE-controlled optional JSON slice endpoints

## Contract: review before implementation

New JSON AST `slice {operand:Expression,lower?:Expression,upper?:Expression}` and
rule instruction slice {operandField,lowerField,upperField}. Existing required-
endpoint listSlice AST/instruction/runtime APIs and source slicing stay unchanged.
No text slices, negative wrapping/clamping, stride, abstract lists or new values.

The parent charges one AST visit. Evaluate operand exactly once, then present lower
and present upper in order through the selected evaluator/shared context. Absence
is field absence, not JSON null; present malformed children enter/error normally.
CUE selects lower zero when absent and list count when upper absent. Defaults are
already-evaluated values, never synthetic ASTs/extra visits/doubled operand calls.
Default lower cannot fail; missing-upper operand-domain rejection occurs after all
present child callbacks. Existing generated list_slice policy owns final domain/
half-open range checks and checked immutable copy/constructor. Thus child errors
precede enclosing domain/range/default-count errors; old diagnostics remain exact.

## Compiler / checked frame boundary

Reuse the same structured compiler with slice target, not a new compiler/seed.
CUE owns evaluateSliceOperand; if hasLower evaluateSliceLower else defaultSliceLower(value:0);
if hasUpper evaluateSliceUpper else if sliceKindIs(list) defaultSliceUpper else
rejectSliceOperand; finishSlice. All header/method/loop/branch/Result charging stays
CUE-owned; native never chooses defaults or child traversal. Full original-wire
family/dead-arm/duplicate validation precedes even custom backends. Source parser
slice actions, collection item actions and slice-frame actions remain distinct.

Opaque SliceFrame owns Start/Operand/Lower/Upper/Done stages, three evaluated values,
optional endpoint presence facts, result, shared child context/callback and4096
control steps. Default methods are explicit checked primitives, not native policy
fallbacks. Finish invokes an explicit Slicer fn(Value,Value,Value)->Result<Value,Error>
on cloned handles, retaining operands on failure. Semantics-gated facades pass
runtime::list_slice (pure generated policy); compiler-only kernels require an
injected callback and no generated semantics/stub/fallback. Default count method
uses checked list_count; default lower materializes its CUE-supplied i64 literal
(zero in production) without AST entry.

Public SliceDriver fn(&mut SliceFrame)->Result<(),Error>, Slicer function pointer;
slice_with_driver(node,names:[&str;3],context,evaluate:&mut dyn FnMut(...),slicer,driver),
slice_interpreted(node,names,context,evaluate,program), slice(node,names,context,
evaluate). Internal shared runner accepts capturing reference drivers. New state/
control-budget errors are slice-specific; existing ListSliceOperands/Range errors
are preserved. Array names avoid oversized runner APIs. CUE emits an ordinary six-argument
slice_fields call; a semantics-gated adapter only groups names into the array and
passes the same default driver/slicer to the shared runner. It contains no
traversal/default/range or emission policy.
No public mutable fields/metrics/unchecked value construction.

Shared AST 4096 visits/64 active, value width 64/depth 32/nodes 4096 and all compiler/
source bounds remain. No frame/work/stack override or lint suppression. Existing
source driver goldens stay byte-identical; factor existing CUE aliases as needed
under unchanged 512 guards and prove prior evaluator layout stability. One more
binary alias keeps this policy exactly 512 original values; control backend 370.
The prerequisite [operand-frame layout](generated-operand-frames.md) is already
merged; this feature introduces no further changes to existing instruction layout.
Compiled/reference control charging is 6/7 steps with upper present/absent; a tail
return adds one, including post-completion failure without publishing a result.

## Evidence

Independent reference/generated AST results/errors for all four presence patterns,
empty/end/nested mixed lists, exact i64/Unicode, null-vs-absence and error ordering,
selected evaluator and exact visit/depth charging (defaults are free), operand
single entry and retained ownership/certificates. Real compiled alternate changes
a default through CUE, not native policy; private stages/failed-finish ownership/
early return/counter and actual compiled/reference charging tests. Wrong-target/
dead-arm/raw metadata/duplicate wire rejection and atomic CLIs. Pinned CUE optional
slices with exact-i64 normalization, no jq rounding. Full Rust/strict Clippy/focused
reviews/pinned preflight/five x86 sandboxes/six artifact-free seeds/profiles/repeated
generation+drift. JSON stays build-only; generated Rust never seeds generation.
No full CUE/bootstrap closure, comprehensive allocation/native-stack bound or
measured performance claim.
