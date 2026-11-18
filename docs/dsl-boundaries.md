# DSL/native boundary audit (#164)

Audit base: `b6f2cb92134a43a32c8e76653853a8fc1b7169cb`. This is a
bounded implementation audit, not a capability milestone or performance result.
Read with the [checkpoint](bootstrap-status.md), [closure inventory](bootstrap-closure.md)
and [solver findings](solver-spike.md). Paths below are repository-relative;
`src/` means `bootstrap/executor/src/`, `generated/` means `bootstrap/generated/`.
The component contracts linked below describe intended interfaces; the named Rust
functions and emitted modules establish what currently implements them.

## What owns decisions, and what still runs natively

Go CUE first evaluates specification files into concrete, build-only JSON. That
requires actual CUE defaults, references and comprehensions. The represented
`if`, `lower` or `argument` objects are then instructions for different Rust VMs
and compilers, not CUE expressions evaluated by the production source evaluator.
CUE-owned executable policy is therefore real, but not bootstrap closure.

**Scanner trace** ([contract](../bootstrap/scanner.md), [backend](../bootstrap/scan-lowering.md)):
`bootstrap/scanner-fragments.cue:#WhitespaceBody/#LineCommentLoop` chooses the
whitespace/comment loops, LF-only comment termination, NUL/BOM rejection text,
flag updates and scalar advancement. `src/scanner.rs::execute` interprets those
statements with checked Cursor calls. `generate` validates first, then `prepare`
computes nearest-loop labels, guaranteed breaks, once-only facts and unreachable
suffix pruning. These are native control-flow algorithms, not passive storage.
`bootstrap/scan-lowering-fragments.cue` chooses Rust loop/branch/label layout,
caches rendered branches with bind/get and merges equal pure branches while
retaining the condition charge. `generated/whitespace.rs::scan` visibly contains
the same loops, `cursor.step()`, `advance_char()` and rejection message.
The cursor supplies checked UTF-8/ASCII movement, lookahead and fuel; pattern
sorting/deduplication/escaping remain native rendering computations. There is no
hidden native skip-comments algorithm behind this example.

**Pratt/primary trace** ([Pratt](../bootstrap/pratt.md), [primary](../bootstrap/primary.md),
[backend](../bootstrap/pratt-lowering.md)):
`bootstrap/pratt-fragments.cue` sequences primary, mixed postfixes and infix
iteration; precedence comparison precedes commitment, and left associativity
adds one to RHS minimum. `primary-fragments.cue` selects ordered guarded clauses,
group minimum 0, prefix minimum 101 and return points. Its preferred flags/minima
are resolved during Go export, not runtime default projection. Runtime grammar
precedence, associativity and constructor data are still native-validated/queried.
`src/pratt.rs::execute` is the reference control VM; `project_body` computes
return-position facts and canonical action categories, while the CUE backend
chooses method spelling, headers and charging. `generated/pratt.rs::execute`
contains the postfix loop and checked binding addition, not a call to parsePratt.
`generated/primary.rs` supplies the primary branches and early returns.

This does **not** make `src/parser.rs::Frame` generic storage: `scan_negation`
combines a generated scanner candidate with optional grammar registration;
`parse_negation` recurses through the selected engine; `build_negation` charges
an AST node and invokes the unary builder. Index/selector/record/conjunction
stages and callbacks are feature-specific native glue. `primary` dispatches the
selected primary engine, rejects reentry and checks completion/pending stages.
Those protocols constrain available algorithms even though CUE chooses the next
action. Native grammar admission, top-level completion and protocol diagnostics
also remain policy boundaries; not every diagnostic originates in CUE.

**Evaluator/conjunction trace** ([assembly](../bootstrap/module.md),
[conjunction](../bootstrap/conjunction.md)):
`bootstrap/evaluation.cue` maps AST kinds to instructions. In `src/lib.rs`,
`evaluate` charges AST entry and `instruction` dispatches native enum arms;
`binary_instruction` evaluates left then right before applying a runtime callback.
This is native reference-VM sequencing, not simply numeric storage. Both source
logical children are eager even when the later logical value policy ignores a
concrete RHS type. `bootstrap/lowering-fragments.cue` independently expresses
child selection, ordered calls, Result propagation, context wiring and module
traversal. `src/codegen.rs::emit` serializes validated instructions from canonical
BTreeMap storage and hands them to `src/module.rs::Frame`; CUE selects first/next
or last/previous and match-arm layout. `generated/evaluation.rs` shows the entry
wrapper and left/right recursive calls; wide listSlice/unify closures isolate
operand temporaries, without establishing a stack or speed guarantee.

`bootstrap/conjunction-control-fragments.cue` chooses explicit top seeding,
first/evaluate/unify/next traversal and finish; `generated/conjunction-control.rs`
contains precisely that sequence. Reference execution uses the conjunction target
of `src/pratt.rs::execute_items`, with native family-specific action dispatch.
`src/runtime/conjunction.rs::ConjunctionFrame` is a feature-specific protocol:
Start/Seeded/Ready/Evaluated/Unified/Exhausted/Done, exact merge-count completion,
checked cursor movement and capacity before callbacks. `unify_item` clones both
handles before the injected atomic merge so failure preserves accumulator and
candidate. CUE selects when to seed/merge/move; native code implements these
operations and their error precedence. Counts/stages do not prove arbitrary
trusted control visited every distinct index. The semantics-gated facade injects
`runtime::unify`; compiler-only callers supply an explicit Unifier, not a stub.

**Unification/lowering trace** ([control](../bootstrap/unify.md),
[backend](../bootstrap/unify-lowering.md), [lowering](../bootstrap/lowering.md)):
`bootstrap/unify.cue` orders top identity before domains, separates null,
collections, text and booleans from numeric identity, and chooses interval
endpoints/infinity/conflicts. `src/unification.rs` validates arity, lexical locals
and even dead branches; its native eval methods interpret finite control.
Projection distinguishes inputValue/inputEndpoint and integerValue/integerEndpoint;
`project_unary` supplies unaryFail only after validation. The CUE backend emits
branches, clones, failure choices and simultaneous letBounds tuple initialization
(both initializers see outer locals). `generated/unify.rs` visibly tests top and
mixed domains first. `src/runtime.rs::unify` delegates to it, without a native meet
fallback. Native tag/endpoint inspection, finite comparisons, checked i64 math,
text operations and error construction remain necessary kernels. Missing endpoints
are not native min/max policy; CUE chooses their infinity interpretation.
Concrete-output admission/incomplete diagnostic selection likewise uses generated
policy; extraction and exact JSON encoding are separate native work.

`src/lowering.rs::Policy/Expr` interprets compiler policy over a checked Context;
`generated/lowering.rs` is the compiled evaluator-lowering driver. The meta policy
in `bootstrap/meta-lowering-fragments.cue` emits complete drivers, including itself.
Native typed decoding, scoped validation, role/alias metadata and rendering kernels
remain; self-regenerating Rust source does not implement CUE specification evaluation.
The [AST backend](../bootstrap/ast-lowering.md) makes the same distinction:
`src/ast.rs::check/prepare` computes bindings/types/remaining uses in key/value
order; CUE chooses clone-before-last-use versus final move and JSON/Rust layout.
Use-count analysis is native algorithmic machinery, not just allocation.

## Duplication and guard pressure: a concrete finding

In `bootstrap/lowering-fragments.cue`, Unary and Binary method tables each feed
both aliases and select(op). `Policy::render` resolves a one-hop alias but preserves
the original node; `generated/lowering.rs` groups aliases into match arms and
then selects the original opcode's method. This factors repeated operand/Result
layout, but leaves dispatch in two places. Canonical unaryRuntime/binaryRuntime
names themselves lack method selections: canonical-case validity alone does not
mean every trusted internal opcode is executable. They are not rule-wire opcodes.

A direct pinned `cue export` of schema + lowering-fragments + lowering measured
**exactly 512 original JSON values**, depth 10, 26 canonical cases, 21 aliases,
10 unary and 11 binary selections. `jq '[..] | length'` counts containers and
scalar values, not object keys; fresh output normalized with `jq -S .` matched
the devshell export. The earlier 500-value figure in the conjunction contract is
historical, not current headroom. Other devshell exports measured Pratt 370,
unification 484, scanner 438, AST 213 and meta 472 values. Counts support this
local maintainability finding, not a desired CUE/native ratio or timing claim.

Repeated binary templates still exist for ordering, notEqual and remainder;
sum deliberately retains its independently selectable RHS alternate. Unify and
listSlice have distinct wide-operand closure layouts. Do not flatten these
exceptions blindly. Pratt's seven parse-minimum aliases and canonical action /
selectorAction categories, and unification's choose-to-if alias/identifier spelling,
show reusable mechanisms already avoiding repetitive trees. CUE let bindings
improve authored reuse but export expands repeated objects: short CUE text is not
necessarily small policy JSON. Aliases/select reduce duplication without proving
that another native primitive or a new parameterized template language is needed.

## Guards are different phase boundaries

| Phase | Actual checks and interpretation |
| --- | --- |
| Specification export/input | Go CUE schema checks and export completeness precede native DSL validation. These are not source-runtime limits. |
| Native program admission | Scanner: 64 statements/16 levels plus predicate bounds. Pratt: 512 JSON values/32 levels and target/dead-arm checks. Unification: 512/64, arity/scope. AST: 16 parameters/64 expressions/16 levels. |
| Compiler-policy admission | `lowering.rs::JsonCheck` streams original values, including duplicate-key values, at 512/32; original-text typed decode rejects duplicates. Cases+aliases cap 64. Pratt/unification control structural checks instead walk a parsed generic value before original-text decoding; do not claim streaming duplicate accounting there. |
| Compiler execution/rendering | Shared 4096 steps, 64 instruction levels, 128 combined frames, 1 MiB cumulative rendered bytes/copies; selected strings 64 KiB. Whole-driver meta emission and module assembly have cumulative contexts, not unlimited per-case resets. |
| Source runtime | Scanner fuel is saturating 128*(remaining bytes+1); parser has 64 AST nodes/32 expression levels and 4096 steps per expression frame shared with primary. |
| Evaluation/value runtime | 4096 cumulative AST entries/64 active nodes; conjunction and collection controllers separately have 4096 steps/frame and width 64. Concrete mixed-value shape certificates bound depth 32/logical occurrences 4096, not solver graph identity. Finite unification control has no loop fuel. |

These guards bound specific work, not total allocations, parsing bytes, native
stack or wall time. Input rejection, compiler exhaustion and semantic conflict
must stay distinguishable. Protocol/cursor guards remain even if CUE changes its
control order. Nothing here justifies raised guards, larger stacks, new native
primitives, or claims that a bounded VM is an untrusted-code sandbox.

## Prioritized bounded recommendations (not approved implementation)

1. **Retain checked kernels and explicit feature protocols.** Keep facts separate
   from CUE domain/order/layout choices; do not relabel all native work as storage
   or mechanically migrate six compilers. Review new callbacks for hidden semantic
   selection. Independent adversaries: invalid negation stages, cross-target dead
   arms and a failed conjunction merge must preserve exact errors/owned state.
   Fresh/profile experiment: regenerate parser/control artifacts from six clean
   seeds and run compiler-only kernels with explicit callbacks and no semantics.
2. **Factor one existing binary family, preserving exceptions.** First try moving
   remainder or one ordering call into the existing Binary alias/select template;
   keep sum's alternate and wide closures intact. This is CUE-only factoring,
   not a new primitive. Independent tests: fixed operand-error order, exact i64
   remainder/extrema and ordering results, plus left-twice's known 40 versus 42.
   Fresh/profile experiment: compare formatted evaluator bytes from clean seeds
   and compiled regeneration; check compiler-only, semantics-only (drivers removed)
   and compiled-lowering-only profiles. Reject a refactor that loses these or
   merely shifts an unchecked dispatch table into Rust.
3. **Measure representation pressure before redesign.** Record per-policy value
   depth/count, compiler step/copy totals and emitted-size changes for that one
   factoring experiment; this is not a runtime benchmark. Only then consider a
   bounded template representation if repeated alias/select tables remain costly.
   Independent adversaries: 512/513 original values, duplicate nested cases,
   alias chains and missing selections; preserve eager dead-template rejection.
   Fresh/profile experiment: run the same boundary fixtures against reference and
   freshly compiled policies without committed artifacts, under all three profiles
   above. No guard increase or new syntax is authorized by a count near the cap.
4. **Design a graph policy boundary separately from the eager VM.** For a future
   #162 experiment, expose scoped IDs/events/environment reads and dynamic read
   dependencies; retain suspended demands before moving readiness/transfer to CUE. Keep
   storage/queue/accounting native but identify scheduling/default choices honestly.
   Independent adversaries: anchored/unanchored cycles, declaration/schedule
   permutations, excluded preference and `a:3&(b+1),b:_` staying incomplete.
   Fresh/profile experiment: compile that isolated policy from fresh inputs and
   run without production semantics/source artifacts; retain separate upstream
   oracles. Do not turn the test-only native spike into a production fallback.

## Closure and decision gate

The #163 first target remains the **seven-definition, two-raw-file AST atom export**
(`bootstrap/ast-schema.cue` + `bootstrap/ast-atom.cue`). Its omitted formatVersion
is singleton insertion, not default selection; recursive schema templates are not
arithmetic value cycles. This needs scoped definitions, closedness, typed open
lists, tagged disjunction and demand-aware unification absent from the eager VM.
Builder argument names remain data at export: binding checks belong downstream.
A future graph can track/refine dependencies and suspended demands while an eager
VM executes ready computations. Define that readiness/demand interface explicitly;
it is not established production behavior or a retry-on-error graph fallback.

Retain the inventory's isolated native-export derivation, independent expected
JSON/mutations, exact-number comparisons and build-dependency attestation. Deny
Go CUE, exported policies and committed/generated artifacts in that attested cut;
keep upstream comparisons and downstream builder compilation explicitly separate.
A native implementation must be named as such, with a visible policy migration
plan, not sold as CUE-hosted because it exports represented DSL instructions.
The #162 worklist/default experiment does not discharge these schema obligations.

Validation: pinned direct export/count/diff, whitespace and focused alias tests
passed. The complete Rust suite, strict all-target Clippy and pinned preflight
passed (13/13, 7 inapplicable checks skipped). All five Nix checks were explicitly
sandbox-rebuilt, including profiles, generation/drift and upstream comparisons;
the artifact-free six-seed graph remains intact. Only x86_64-linux was validated.
Recommendations above remain proposed experiments, not claims they have run.
Review these findings with #162/#163 and the overview under **#161**, then obtain
user agreement before choosing a production milestone; routine expansion remains
paused. No production code, guard, native primitive or fallback changes here.
