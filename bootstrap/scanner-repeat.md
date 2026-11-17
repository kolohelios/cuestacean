# Bounded scanner repetition

The scanner DSL adds `repeat {count, body}`, with literal count 0..64. It repeats
its validated body at most count times; break exits the nearest repeat or while.
Zero iterations still validate the body. Statement/depth/predicate limits count
the body once, not an expanded copy. Existing caps and input-length fuel remain.

Runtime accounting is one entry statement step plus one step for each attempted
iteration, then ordinary body steps. Zero count costs one step; break consumes
its existing action step and skips later iterations without a final loop-test
charge. Fuel can reject large bounded programs, including on empty source. This
is a generic finite control operation, not a Unicode-specific cursor primitive.

Native typed validation, reference execution and flow/nearest-loop facts remain.
CUE emits entry/iteration steps, fixed Rust for loops, zero-count omission and
once-blocks for count one or always-breaking bodies, labeling only when a
break targets that block. This avoids unused-label, unreachable
and never-loop lint suppression. No native statement source emitter is added.

Production Unicode digit validation now repeats one four-digit body rather than
expanding copies. Accepted source spans, exact diagnostics and flag lifecycle are
unchanged; internal fuel use changes explicitly. The earlier unrolled composed
program is retained as an actual compiled baseline alongside the nested baseline.
Independent count/break/nesting/zero/unused-validation/fuel and compiled-loop
oracles supplement the existing exhaustive Unicode and pinned upstream checks.
Paired compiled fuel fixtures accept exactly 256 steps with one remaining byte
and reject the same program plus one action (257), checking both budget edges.

The same six artifact-free seeds, build-only JSON and committed Rust drift policy
apply. This is reusable CUE control/compiler machinery, not new string syntax,
full compiler bootstrap closure or measured performance improvement.
