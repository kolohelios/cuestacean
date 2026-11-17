# CUE-owned inclusive bound construction

`lower-bound.cue` / `upper-bound.cue` admit one concrete integer threshold and
construct inclusive Bounds through CUE endpoint placement. Other domains keep
BoundThreshold. Default rules read scalar threshold fields first; custom reference
rules test child errors preceding admission, not generated threshold-child rules.
Source syntax, precedence, AST/rule wire and i64/infinity semantics are unchanged. Exclusive bounds and
expression thresholds in source remain unsupported; missing endpoints remain
infinite, not extrema, and singleton bounds remain incomplete.

Unary control now admits `bounds` with a checked left-only endpoint subset:
left input projection, integer literals, empty and choose. Choose uses existing
left-only kind/boolean conditions; both arms validate eagerly. Right projection
and locals are forbidden even in unused arms. Endpoint conditions, letBounds,
pair tests and binary arithmetic remain excluded from unary programs. The
reference engine evaluates endpoints on one input, never a fabricated RHS.
Existing serialization-only endpoint aliases and CUE compiler layout are reused.

Function metadata adds unary lowerBound/upperBound, and input-independent
boundThreshold failure is valid in either arity. Production runtime facades are
semantics-gated and delegate generated control without native threshold or layout
fallbacks. Native checked endpoint projection and representation remain; no new
compiler or bound-specific construction kernel is introduced.

An actual compiled alternate lowerBound program places its threshold in the
upper endpoint: input 7 yields {lower: None, upper: Some(7)} rather than the
production/reference {lower: Some(7), upper: None}. Fixed extrema/domain/errors,
endpoint-scope/unused-branch, output/unification and source/upstream checks pin
behavior independently. Six artifact-free seeds, JSON build-only/Rust drift
policy and all budgets remain. This is executable scalar policy progress, not
full constraint solving, compiler bootstrap closure or measured performance.
