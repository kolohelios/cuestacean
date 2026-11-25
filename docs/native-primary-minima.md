# Native canonical record and conjunction minimum inputs

[#298](https://github.com/kolohelios/cuestacean/issues/298) covers two inputs to
`record-primary` and `conjunction-primary`. Each input keeps its original
`*0 | 100` choice in a separate owner. Each native invocation uses one owner
and one sole-reference root, using the existing once-instantiated scalar
profile. Neither invocation reads the other owner or a generated answer.
Go CUE still exports the full PROGRAMs against the original schema.

## Provider boundary

Canonical calls import their own selected native helper. The two high-minimum
fixtures load their own unresolved owner instead. A read-only gate validates
one complete object with exactly one concrete CUE integer, either `0` or `100`,
before the PROGRAM is composed. The two canonical outputs are byte-identical,
so shape cannot identify the producer. The fixed route mapping, separate
construction closures, and independent source mutations prove that boundary.

Never use selected zero in an open context. The opposing `(*100 | 0)` preference
is incomplete with the unresolved owner but succeeds with frozen zero. The
existing literal-100 override instead succeeds with the raw owner and conflicts
with frozen zero. Keep both distinctions, including ordinary evaluation,
export status, and empty stdout on failure.

## Verification

Compare the complete PROGRAMs and compiled Rust with independent expectations
and the merged baselines. Flip each source mark separately, run the unchanged
native exporter and real Pratt compiler, and run the newly generated driver in
compiled and interpreted parser tests. The sibling driver must be unchanged.
Preserve the existing selected-driver tests, [slice lower provider](native-slice-lower-default.md),
and all other minimum slots, including file variants. Standalone compiler
construction and committed generated Rust do not change.

This is a two-input construction cut, not native PROGRAM/schema or broader
default support. [#175](https://github.com/kolohelios/cuestacean/issues/175) and
[#293](https://github.com/kolohelios/cuestacean/issues/293) remain open.
