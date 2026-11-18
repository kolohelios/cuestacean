# CUE-controlled explicit source list slices

## Contract: review before implementation

Extend postfix brackets with `operand[lowerExpression:upperExpression]`. Both
endpoints are required; no omitted endpoints, stride, text slicing or coercion.
Direct one-node listSlice AST with operand/lower/upper fields reuses existing
[three-child semantics](list-slicing.md), budgets, CUE range/domain policy and
certified immutable storage. Each endpoint parses once through all three selected
engines at CUE minimum0. Slices/index/selectors chain left-associatively above
unary/binary operators. Syntax parses completely before evaluation, as before.

Optional grammar slice:{kind,operandField,lowerField,upperField} selects the AST
constructor; nonempty/distinct fields, none kind, validate eagerly even without
use. Absence disables slicing without fallback. For this bounded shared-bracket
profile slice requires index metadata; absence of index disables all brackets.
Existing generic/typed index metadata and APIs stay unchanged. Omitted lower or
upper remains a source operand error. Conservative after-operand newline handling
applies before brackets and separators; normal leading endpoint trivia is allowed.

## Executable CUE / private checked bridge

After advanceIndex/parseIndex parses the lower-or-index expression, CUE scanSlice
probes an optional colon using a new tiny scanner. hasSlice is only a candidate
fact. CUE chooses advanceSlice/parseSliceUpper(minimum0)/closeSlice or the existing
closeIndex. Native may only commit checked spans, call one recursive expression,
and defend stages/cross-phase/completion; it does not classify bytes or loop over
postfixes. Existing index-close scanner owns the required bracket/trivia/error.

IndexState adds SliceScanned/Advanced/Parsed phases, preserving old Parsed→
closeIndex for actual historical drivers. closeIndex also accepts negative slice
probe. A private Option<Box<JSONValue>> stores only the upper AST when present,
keeping ordinary recursive frames narrow and allocating only for actual slices.
Lower remains in existing right slot. Active/pending slice state rejects primary,
binary, selector and early-return misuse; failed node charge must retain complete
pending operands. closeSlice clears index/selector/upper state only on success.
Existing CUE AST compiler emits a new ternary builder; no native JSON-map assembly.
All three selected engines and shared frame fuel propagate unchanged. No new
compiler, seed, native emitter, stack override or raised guard.

The expanded backend stays within 512/32/64 bounds by replacing the ordinary
validated action-name→method map with CUE's checked snake identifier rendering.
The resulting backend has 330 original values (previously 497).
Native still eagerly validates original wire/targets before projecting canonical
unit-action metadata; there is no raw action/parseMinimum wire escape. CUE owns
method naming/layout/charging. Actual prior driver renderings must be byte-exact.

## Pinned upstream limitation

Pinned Go CUE reports incomplete for
`close({"items":[close({"x":[1,2]})]}).items[0:1][0].x`, while the corresponding
direct list literal `[close({"x":[1,2]})][0:1][0].x` exports concretely. The former
composition remains an independent complete-tree/value test of this concrete
profile, not a claimed upstream equivalence. Differential fixtures use the latter;
this does not establish full CUE compatibility or general field solving.

## Independent evidence

Retain actual prior full Pratt as pratt-no-slices, plus older no-selector/no-index
artifacts byte-identically. Default and interpreted parsers need independent full
trees/results/errors for computed/mixed chained ranges, precedence, nested fields,
Unicode/extrema, missing/reversed/negative endpoints, unsupported source forms,
metadata/collisions/absence, selected drivers and parse/node/frame budgets. A real
compiled upper-minimum100 alternate must change endpoint parsing, including
through selected nested engines. Check private stages/progress/fuel/unfinished
returns and failed node-charge ownership. Exact-i64 upstream source arrays and
atomic CLI failures remain required, as do full Rust/strict Clippy, focused
reviews, pinned preflight/five x86 sandboxes/six artifact-free seeds/profiles and
repeated generation/drift. JSON stays build-only; generated Rust is never a seed
input. No full CUE/bootstrap closure or measured performance claim.
