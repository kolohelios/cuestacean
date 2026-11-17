# Early return and flat primary control

The Pratt/primary version-1 DSL adds `{op: "return"}`. It consumes one shared
frame step and exits the current driver invocation, unwinding nested `if` and
`while` blocks rather than merely breaking the nearest loop. Recursive parser
calls have their own driver invocation; returning from one does not return from
its caller. Primary completion/stage guards still run after the driver returns:
returning before a complete primary is an error, not a shortcut around validation.

Whole-program decoding/512 JSON-value and 32-level guards remain eager. Native
validation also rejects statements following a structurally definite return in
the same block. Definite return means `return`, or `if` whose two arms definitely
return; a block definitely returns when its last statement does. Loops are not
assumed to terminate/return, and conditions are not constant-folded. Both arms
and loop bodies validate even when unselected. This is structural checking, not
a general termination/control-flow proof.

The native compiler projects a canonical root `tail` fact (`return` or `value`),
not Rust source or a user-supplied wire field. Returns also carry a canonical
`position` fact (`early` or `tail`): function-tail context propagates through a
final definitely-returning `if`, never into loop bodies. CUE owns explicit early
return versus tail-expression text and whether to emit the final `Ok(())`. Structurally unreachable statements reject
rather than generating warnings, dropping unvalidated code or suppressing lints.
Existing return-free programs regenerate byte-exact under the default compiler.

Production primary dispatch reduces maximum original JSON depth from 15 to 8
(under the unchanged 32-level guard), becoming sequential guarded clauses, each returning
after completing its primary. CUE still chooses group/not/negation/bound/string/
atom/integer order, threshold-group minimum and all frame calls. An actual old
nested primary from main `a6960ab0` (`bootstrap/generated/primary.rs`) remains
compiled and reference-executable for fixed/full-corpus
AST and diagnostic comparisons; the existing threshold-minimum alternate uses
the flat template. No source language, value/solver or grammar change is implied.

Tests pin nested loop/if return, current-invocation scope, unfinished-primary
rejection, validation/forged metadata rejection, shared fuel and independent
AST/error outcomes. Existing source node/depth/step and scanner/compiler limits
stay unchanged. Same six artifact-free seeds, repeated generation/drift, feature
profiles and upstream comparisons apply; JSON stays build-only and Rust a
committed drift baseline. Native checked frames, validation/flow facts, reference
VM and compilers remain boundaries; no bootstrap closure/performance claim.
