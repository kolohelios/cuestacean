# CUE-controlled unary source calls

Default source `len(expression)` now constructs `length {operand}` and uses
CUE text-byte/list-element dispatch ([source lists](source-lists.md)); the original
`stringLength` API remains text-only. Exactly one parenthesized expression is
supported: empty/multiple arguments, general callees, selectors, structs/bytes
and other builtins remain outside the profile.
Examples: `len("é🙂") == 6`, `len("a" + "b") == 2`, and `>=(len("abc")) & 3`.
No coercion, Unicode normalization, lazy child evaluation or new solver domain.

Optional grammar `calls` entries contain `{token, kind, operandField}`. Tokens
are complete ASCII identifiers, unique and disjoint from atom spellings; kinds
must be nonempty and operand fields nonempty/not `kind`. Missing/empty calls
disable registration. Names and constructors are data, not native builtin cases.
All metadata validates before parsing, including unused entries. Existing source
and return-free/older primary policies remain valid; no native call fallback.

After CUE identifier scanning, `hasCall` exposes checked metadata lookup and
`advanceCall` commits the selected name. CUE then explicitly sequences existing
`scanGroup`, `advanceGroup`, `parseGroup` at CUE minimum 0, `closeGroup`, return.
For the call stage, `scanGroup` invokes the required `call-open` CUE scanner,
which owns trivia, newline-after-name rejection and missing-opening diagnostics.
Spaces are allowed; LF after the name (including through comments) rejects,
matching pinned CUE comma insertion. The shared LF/CR flag also conservatively
rejects standalone CR, like the existing operand profile. This is not full CUE
comma insertion.

Native private stages preserve call index/span/progress/UTF-8 and distinguish
ordinary/bound/call groups. They reject invalid order, unfinished results and
cross-phase actions; both selected drivers propagate through argument recursion.
A successful close adds one node using the existing CUE unary builder. Arguments
share original 32 recursion levels/64 AST nodes and 4096 steps per frame; all
scanner/compiler limits and same six fresh-seed/artifact policies remain.

An actual old compiled `primary-no-calls` stays byte-exact; an actual compiled
`primary-call-high-minimum` changes argument minimum to 100 (and now list-element
minimum too, tested separately), rejecting the
plus in `len("a"+"b")` where production returns 2. Fixed full AST/output/error,
metadata/isolation, private stage/resource/driver, CLI atomicity and expanded
pinned upstream Unicode byte-count comparisons establish this slice. Native
lookup/validation/checked frames and text/output kernels remain explicit; no
full builtin/CUE/compiler closure or measured performance claim.
