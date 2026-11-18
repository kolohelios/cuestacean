# Solver architecture spike (#162)

## Design contract — review before implementation

This is a test-only native experiment, not production source syntax or a solver
replacement. The existing eager VM and immutable concrete-value DAG stay intact.
A flat integer-constraint IR stands for the small CUE examples below; no CUE parser,
open-struct/definition semantics, general disjunction or backward arithmetic solver
is implied. CUE snippets are independent pinned oracles, not inputs silently
accepted by the production parser.

### Identity, scope and ownership

One graph owns an arena of named field nodes and compiled expression constraints.
Intern all field names before resolving references; canonical name order gives
stable IDs independent of declaration order. Each field has zero or more conjuncts
and at most one preferred integer annotation. Multiple CUE declarations for a name
would normalize into one IR field with multiple conjuncts; duplicate IR fields
reject. References are checked arena indices, never owning pointers. Thus reference
cycles do not create Rust ownership cycles or reuse the concrete-value Arc DAG.
The graph topology is immutable after construction; each solve owns fresh domain,
queue/pending and accounting arrays. No graph state survives into production values.
There is no cross-solve cache or edit invalidation: rebuild/re-solve on edits. IDs
are graph-scoped, not persistent identities across changed scopes/topologies.

Expression fragment: top/int conjuncts, i64 literal, reference, addition. Inline
abstract operands inside addition reject as Unsupported: a concrete target must
not mask an unresolved computation (see findings). Subtraction in probes is
addition of a negative literal. Expressions are owned bounded trees; only
references point to field IDs. No recursive reference expansion.

### Refinement and scheduling

Domain lattice: Any -> Int -> exact i64, plus conflict. Meet never replaces one
exact value with a different one. A task evaluates one conjunct against current
field domains, then meets its result into its target. Addition computes only when
both operands are exact; otherwise its result is Int in this integer-only IR.
Checked i64 overflow is a distinct unsupported integer-range error, not a claim
of CUE contradiction (CUE integers are not limited to i64).

Direct reference conjuncts are aliases: meet both endpoints. Register the alias
task as a watcher of both endpoints so an anchored alias cycle can refine even
when its literal conjunct runs last. References inside arithmetic register reads,
not inverse arithmetic rules. Do not infer b=2 from a=b+1 and a=3 by inventing an
unimplemented arithmetic solver.

Use a deduplicated worklist of constraint-task IDs, with reverse read/watch edges.
A successful refinement enqueues affected tasks; unchanged evaluation does not.
Test FIFO/LIFO and ascending/descending initial tasks, as well as declaration
permutations. Domains have at most two strict refinements per field per phase;
this finite-height argument, not arbitrary repeated evaluation, supports convergence
for this fragment. Output counters may vary with schedule; meanings must not.

### Defaults are a separate projection

Support one preferred integer annotation per graph initially: *1 | int means an
Int constraint plus a preference for 1, never an early destructive assignment.
First run constraint propagation without defaults and retain that result. If the
preferred value remains compatible, run a fresh projection phase seeded with it;
then propagate dependencies again (including a=b+1, b=*1|int). A concrete
constraint of 2 excludes the preference and still yields 2. Do not mutate the
saved constraint result with a projected default.

Multiple interacting defaults and a preferred projection that conflicts with
otherwise unresolved constraints are explicitly Unsupported, not silently chosen
by queue order or reported as proof of global unsatisfiability. Full default
calculus/backtracking is a follow-up decision, not hidden inside this spike.

### Quiescence, cycles and failures

A drained queue is not necessarily concrete. Classify unresolved SCCs separately:
pure reference cycles remain Incomplete; unresolved SCCs containing arithmetic
edges are Cycle, matching the pinned examples rather than labeling every cycle
bottom. Anchors/default projection may resolve a cycle before classification.
Literal/domain incompatibility is Conflict; missing references/invalid IR,
unsupported default interaction, i64 range and resource exhaustion are separate
errors. SCC traversal operates on bounded IDs/edges, not recursive evaluation.

The pinned toolchain (reporting CUE language v0.16.1, binary version `(devel)`)
probes established that a:a+1 and the balanced a:b+1,b:a-1
both fail concrete export with cycle diagnostics while cue eval succeeds; the
balanced equations are not a mathematical contradiction. a:b,b:a exports
incomplete, and adding b:2 resolves both the alias cycle and balanced arithmetic
cycle. Tests must preserve these distinctions and not equate export failure with
contradiction. These observations are for the pinned toolchain, not general proofs.

### Limits and boundary

Hard structural caps: 64 fields, 256 expression nodes, 128 reference edges, 16
expression levels and 64-byte nonempty field names. Bound constraint tasks by expression count;
queue coalescing bounds queued tasks. Reverse alias edges/watch entries can add
at most another 128 entries to the 128 original reference occurrences. Count solve
work for task pops, expression
visits, watcher/edge traversal and SCC work; cap 4096 across constraint and projection
phases. Tests may lower work limits, never raise hard caps. Validate structures
before building adjacency/state; caller-owned input allocation, map comparisons,
allocator bytes and wall time are not comprehensively bounded by these counters.
Construction/destruction of caller-supplied expression trees is not an adversarial
wire-input boundary; this manual test IR must not be exposed as one.
SCC call/explicit stacks must be bounded by the field cap. No larger production
budgets/stacks or lint suppression.

The native prototype owns all scheduling/transfer/default/SCC code deliberately,
so this issue can test graph mechanics without prematurely extending the DSL.
Checked storage, stable identity, queue primitives and accounting can remain
native. Semantic meet, readiness, transfer/default choice and scheduling policy
are candidates for executable CUE; the current DSL lacks graph/event/environment
interfaces to express this honestly. No claim that the spike is CUE-hosted or that
the current eager VM already supports graph semantics. Audit #164 should assess
those interfaces before a production port.

### Intended prototype interface

A test-only module under bootstrap/spikes, included by one Cargo integration test:
Expr::{Any,Int,Value(i64),Ref(String),Add(Box<Expr>,Box<Expr>)};
Field {name:String,constraints:Vec<Expr>,preferred:Option<i64>};
Domain::{Any,Int,Value(i64)}, Status::{Concrete,Incomplete,Cycle};
Graph::new(Vec<Field>), Graph::solve(Schedule::{Fifo,Lifo},reverse_initial:bool,
work_limit:usize<=4096). Report retains constraints and selected-domain maps,
status and charged work. Expose checked ID/count inspection for identity tests;
no production modules/API are changed. Independent fixtures cover forward refs,
24 diamond permutations, anchored/unresolved cycles, defaults/refinement and bounds.

## Reproduce

```sh
nix develop ./infra/tooling --command cargo test \
  --manifest-path bootstrap/executor/Cargo.toml --test solver_spike
nix develop ./infra/tooling --command cargo test \
  --manifest-path bootstrap/executor/Cargo.toml --no-default-features --test solver_spike
```

Upstream probes live in `bootstrap/spikes/solver/probes.cue`; for example run
`cue export .../probes.cue -e forward --out json` in the pinned shell. Missing CUE
is a test failure, not an optional/skipped comparison. The sandbox check supplies
the pinned CUE executable and exercises both default and compiler-only profiles.

## Findings and decision

Validated on x86_64-linux: 14 spike tests in default and no-default-features
profiles, full Rust tests and strict all-target Clippy; pinned Shaka preflight
13/13 (7 inapplicable checks skipped). All five Nix checks ran in actual sandboxes
(the unchanged JSON export was explicitly rebuilt), retaining six artifact-free
seed binaries, feature profiles, generation/drift and upstream checks. Incompatible
aarch64 systems were omitted, not tested. Focused ownership/scheduling and test
coverage reviews preceded final checks; the readiness finding below was closed
by an explicit fragment restriction and negative tests, not a fallback.

- Stable IDs and watcher-driven refinement handle forward refs and a shared diamond,
  without expanding references into recursive evaluation. Test all 24 diamond
  declaration permutations and four initial-order/queue combinations.
- Default selection needs a separate phase: preserving Int while projecting 1
  leaves refinement to 2 valid. A preferred value can also anchor an otherwise
  unresolved arithmetic SCC; classifying cycles before projection would reject it.
- Pinned arithmetic-cycle diagnostics are not a generic bottom test. Both the
  unsatisfiable a=a+1 and balanced a=b+1,b=a-1 fail export as Cycle; an alias SCC
  with an arithmetic dependent outside it remains Incomplete.
- Domain information is not residual-expression readiness. Review found that
  a:3 & (_+1) would falsely appear concrete if the pending addition were replaced
  by Int. The checked fragment now rejects inline abstract arithmetic operands;
  its isolated upstream probe rejects the invalid operand. Likewise a:3&(b+1),b:_
  remains Incomplete: its partial domain map is not an exportable value and no
  inverse b=2 rule is invented. A future broader solver must retain demands and
  suspended constraints, not erase them behind a domain approximation.
- Multiple defaults and arbitrary CUE integers already exceed this fragment:
  independent upstream probes export two defaults and MAX_i64+1 successfully;
  the prototype reports Unsupported and Overflow respectively. Conflicting
  preferred projections also remain explicitly Unsupported, without claiming
  either full default backtracking or proof of global unsatisfiability.

Decision: retain the eager production evaluator for its existing subset. A graph
arena with stable IDs, fresh per-solve state and explicit dependencies is a viable
bounded experimental direction; this is not evidence that a generic fixed-point
engine alone implements CUE. A production milestone still needs scoped identity,
residual readiness, full default calculus, dynamic dependency discovery and policy
interfaces that can move executable semantics into CUE. Review with #163/#164/#165
under #161 before selecting that milestone; routine feature expansion stays paused.

Suggested next bounded solver investigation (not authorized by this result):
retain suspended arithmetic constraints and explicit export demands while moving
one transfer/readiness policy behind a checked CUE-driven graph-event interface.
Choose its syntax/closure target only after the bootstrap inventory and boundary
audit. Do not replace the current VM or make the native spike a permanent fallback.
