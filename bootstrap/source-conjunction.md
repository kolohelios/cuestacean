# CUE-controlled source conjunction

## Contract: review before implementation

Admit `and([expression, ...])` with explicit/trailing commas, empty/nested operands,
and existing postfix/unary/binary composition. Require literal brackets directly
inside parentheses; computed list arguments and general functions remain outside
this bounded profile. Lower directly to conjunction {operands:[AST...]}, one node
plus ordinary child nodes, with no intermediate list AST/value. Existing abstract
scalar operands/refinement and incremental CUE merge/error order apply. Concrete
list/record admission and certificates are unchanged. Parsing completes before
semantic evaluation, as with source records.

Optional grammar conjunction:{token,kind,operandsField} selects constructor;
default and/conjunction/operands. Eagerly validate identifier, nonempty kind/field
(neither kind), collisions with atoms, unary calls and record token even without
use. Absence disables syntax without fallback; no dependency on grammar.list.
Existing call-open scanner owns newline-after-name rejection. Leading trivia,
required bracket, list-close/list-tail and group-close scanners own tokens/errors.
No omitted implicit commas, references, abstract list storage or general solving.

## Executable CUE and checked stages

Primary checks hasConjunction after scanIdentifier, invokes conjunction and
returns, before record/call dispatch. This primitive dispatches a separate selected
conjunction-primary parser-target driver on the same frame/fuel. CUE sequence:
advanceConjunction, openConjunctionCall, whitespace, openConjunctionList,
whitespace, probeListEnd, while hasListElement {parseListElement(minimum0),
appendListElement, advanceListTail, optional whitespace/probeListEnd}; then
closeConjunctionCall, buildConjunction. Native hasConjunction is only a checked
metadata/token candidate. No native operand loop, token policy or JSON map.
Reuse existing unary AST builder and list buffer, not a new runtime list.

PrimaryState adds separate conjunction selected/named/call/list-ready/parsed/
appended/closed/call-closed variants. Existing list primitives accept paired list
stages and preserve the family on each transition; buildList rejects conjunction
states and buildConjunction rejects list states. Source-record/call/atom/index/
selector operations cannot mutate active conjunction stages. Failed final node
charge retains buffered ASTs. Early returns never expose partial results. No
additional Frame field/large inline state, guard/stack increase or lint suppression.

Add selected fourth parser engine and parse_with_four_drivers /
parse_interpreted_with_four_drivers hooks (expression,primary,record,conjunction).
Existing APIs/signatures stay compatible and choose mode-appropriate defaults for
the new driver. All four selected engines inherit into every nested operand/group/
field/list/RHS. A real high-minimum conjunction driver changes operand parsing
through these contexts; no hidden fallback. Existing source64 nodes/32 recursive
levels/4096 shared steps and evaluator4096 visits/64 active nodes remain. 63 scalar
operands fit source budget; JSON conjunction capacity64 remains independent.

## Pinned upstream boundary

Pinned Go CUE exports `and([1,1])` as 1 but reports a non-concrete operand for
`and([1,1])+1`; it similarly exports a concrete list at the root yet rejects
slicing the direct and([...]) result as type top. This profile eagerly reduces
through existing conjunction semantics. Those compositions get independent
AST/value tests, not an upstream-equivalence claim; differential fixtures compare
exporting root results. No upstream-bug/full-compatibility claim is made.

## Independent evidence

Retain actual prior primary as primary-no-conjunction, plus older no-record/no-list
controls byte-identically. Default/reference complete trees/values/errors for
abstract refinements, empty identity/incomplete, nested/mixed/concrete results,
Unicode/extrema, malformed delimiters/computed arguments, token metadata/absence/
collisions, parse-before-eval and node/depth/frame limits. Private progress/phase/
completion/fuel/failed-charge ownership, full target/dead-wire validation, and
real selected-driver propagation. Pinned and([...]) source comparisons with exact
integer-safe output and atomic errors, all old JSON/runtime tests preserved.
Full Rust/strict Clippy/focused reviews/pinned preflight/five x86 sandboxes/six
fresh artifact-free seeds/profiles/repeated generation+drift remain required.
JSON stays build-only; generated Rust never seeds generation. No full CUE/bootstrap
closure, comprehensive allocation/native-stack bound or performance claim.
