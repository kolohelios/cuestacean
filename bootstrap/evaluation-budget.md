# Shared AST evaluation budget

Default reference and CUE-generated scalar evaluator APIs create one
`runtime::EvaluationContext` per invocation. Every AST evaluation entry, including
the root and repeated evaluation of the same child, shares at most 4096 visits and
64 simultaneously active AST nodes (root counts as one). These are new evaluator
limits, not raised parser/compiler/scanner budgets. All default admitted source
ASTs fit the existing 64-node/32-parser-level profile.

Native `visit` checks remaining visits first, then active depth; it enters only
when both allow it. An accepted entry consumes one visit even when kind/field/
child/semantic evaluation fails. A depth-rejected entry consumes no visit. The
callback receives the same mutable context. Child field lookup precedes attempting
its AST entry: a missing field is still a lookup error, not a charged visit.
Depth unwinds on both success and
ordinary Result errors, but visits never replenish. `EvaluationVisits` displays
`evaluation exceeds 4096 AST visits`; `EvaluationDepth` displays
`evaluation exceeds 64 active AST nodes`. API calls get fresh contexts; counters
are not global, per-rule, per-subtree or replenished on errors.

CUE owns default context creation, recursive propagation, and the checked entry
wrapper plus node-dispatch function source. Instruction `evaluate` calls the
checked entry with the current context, never a fresh one or the unchecked node
body. Native reference recursion mirrors it; the native context only manages
checked counters, not node-kind dispatch or language policy. Existing eager
left-first child/operand/output behavior stays intact below the limits. Custom
compiler modules/native callbacks are trusted, not a sandbox: they can choose
different wiring and are not silently repaired by a native fallback.

This counts AST visits, not instruction evaluations, byte/allocation work, JSON
parsing or destruction, kernel cost, or every native stack frame. Finite rule
instruction recursion retains its pre-existing decoding/compiler boundaries.
The guard is not comprehensive hostile-input hardening, a new solver, reference
ownership design, full CUE/compiler closure or a measured performance result.

Independent tests pin 4096/4097 visits and 64/65 active nodes, fresh invocation,
error precedence and depth restoration. The actual compiled left-twice rule
visits a small left-spine tree exponentially: 11 additions produce 4095 visits;
adding one/two unary wrappers yields 4096 success/4097 failure. Reference and
compiled rules use the same expected outcome, not merely agreement. Existing
source/upstream corpus, six artifact-free seeds, generated Rust drift and feature
profiles remain; JSON stays build-only and committed Rust is never a seed input.
