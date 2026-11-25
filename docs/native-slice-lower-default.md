# Native canonical slice lower default

[#296](https://github.com/kolohelios/cuestacean/issues/296) cuts one input to the
canonical `slice-control` PROGRAM. The existing two-file native exporter
selects the lower-bound default from `*0 | 1`. Go CUE still validates and exports
the PROGRAM against its original schema. This does not add native PROGRAM or
schema evaluation, change a standalone compiler input, or resolve [#175](https://github.com/kolohelios/cuestacean/issues/175).

## Provider boundary

The canonical export imports the whole native helper object as
`#SliceLowerInput`. The literal-one override and the tail control still load the
same unresolved CUE owner. Each route selects one provider before assembling the
export arguments and validates its complete object in isolation. This prevents
an open context from filling a missing or incomplete lower input; it does not
prove that a valid integer came from the intended producer. The constructor
source inventory and source-mutation check address that separately.

Do not refine the selected canonical object. The original raw choice and an
incoming `(*1 | 0)` preference leave the lower operand incomplete. Replacing the
original choice with the selected `0` incorrectly makes that export succeed.
The existing literal-one override instead succeeds with the raw owner and
conflicts with the selected `0`.

## Verification boundary

Compare the complete canonical, lower-one, and tail PROGRAMs against the pinned Go
oracle and their existing compiled Rust baselines. Source mutation must genuinely
export the altered preference through the unchanged native exporter and drive the
real Pratt compiler; changing a JSON answer by hand is not a construction test.
Keep the existing callback-order and missing-versus-explicit-lower tests. The
helper's recursive build closure must exclude Go CUE, downstream exports and
executor, seed and verification inputs, and committed generated answers.
