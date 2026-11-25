# Native canonical Pratt minima

[#303](https://github.com/kolohelios/cuestacean/issues/303) separates two
construction inputs from `bootstrap/pratt-fragments.cue`. Independent
owner/root pairs preserve the original `*0 | 100` choices. The index minimum
controls both ordinary indices and slice lower endpoints; the slice minimum
controls upper endpoints. These are distinct from the slice-control lower
runtime input documented in [#296](native-slice-lower-default.md).

Canonical `pratt` and `file-pratt` use the same two native-selected helpers.
The six alternate selectors and open reference contexts use both unresolved
owners. Check the whole tuple before composition, even when a selector
suppresses postfix parsing. A literal `100` override succeeds with its raw
owner and conflicts with a selected zero; an opposing `(*100 | 0)` preference
can stay incomplete raw when it would succeed against frozen zero. Do not
retry a failed selection with another provider.

`file-pratt` continues to use `file-pratt-lowering.json` for Rust emission;
`file-primary` and the overlay itself receive no new provider. Verification
uses the real compiler with the appropriate policy and stages fresh PROGRAM
and Rust pairs in the normal driver slots to exercise both compiled and
interpreted parsers without widening their interfaces. Separate source and
closure checks and independent source mutations prove producer identity;
identical zero payloads alone cannot do that.

Go CUE still exports and validates both full PROGRAMs. This cut neither
moves their schema to native construction nor settles interacting defaults
tracked by [#175](https://github.com/kolohelios/cuestacean/issues/175).
