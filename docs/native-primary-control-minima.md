# Native canonical primary control minima

[#300](https://github.com/kolohelios/cuestacean/issues/300) covers the bound,
call, and list minimum inputs to `primary`. Each keeps its original `*0 | 100`
choice in a separate owner. The existing once-instantiated scalar exporter
reads one owner/root pair per invocation. The three construction graphs are
independent. Go CUE still validates and exports the full PROGRAM against its
original schema.

Only canonical `primary` imports the selected tuple. The five existing alternate
selectors still use all three unresolved owners, including the combined call/list
high-minimum override. The complete tuple must pass isolated shape and concrete
integer checks before composition. Equal canonical payloads cannot prove which
producer supplied them: the fixed routing, per-producer closure checks, and
independent source mutations establish that separately.

An incoming `(*100 | 0)` preference can leave the raw minimum incomplete even
when composing with a frozen `0` succeeds. The literal `100` override instead
succeeds raw and conflicts with a frozen `0`. Keep those outcomes distinct from
setup failures. Never feed selected DATA into open contexts or retry with another
provider.

Verification compares the six complete PROGRAMs and compiled Rust against the
original sources and independent expectations. Flip one source mark at a time,
run the unchanged exporter and real Pratt compiler, then execute the newly
emitted driver on the compiled and interpreted parser paths. Confirm the other
two minima remain unchanged. Preserve the committed Rust baselines, full export
bundle, standalone compiler construction inputs, existing record/conjunction and
slice proofs, and both Pratt minima including file variants.

This removes three canonical DATA dependencies, not full PROGRAM/schema export.
Broader default compatibility in [#175](https://github.com/kolohelios/cuestacean/issues/175)
and [#293](https://github.com/kolohelios/cuestacean/issues/293) remains open.
