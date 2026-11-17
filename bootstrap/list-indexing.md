# Closed-list indexing

Index the existing concrete list profile; [nested values](../docs/nested-list-values.md)
now share certified immutable child storage. JSON AST
`listIndex {left,right}` and binary rule `listIndex` evaluate left then right with
the same evaluation context before CUE policy. CUE requires a closed list and
concrete i64, then chooses `0 <= index < count` before checked element extraction.
No coercion, negative/from-end indexing, string/struct indexing or general solver.
Errors: `indexing requires a concrete closed list and integer` (IndexOperands),
`list index out of range` (IndexRange). Child errors precede those guards.

Native always-available `list_at(&Value,&Value)` defensively checks domains,
checked i64-to-usize conversion and `.get`, cloning just the selected value (including a shared nested list).
Text selection shares its Arc and survives list/input drop. CUE uses existing
endpoint comparison plus left-only `listCount` endpoint (serialized internally
as listCountEndpoint, not accepted as wire) for range policy. Existing compiler
emits the binary listElement kernel call; name/arity do not imply semantics.
The compiled list-index-zero control demonstrates altered behavior.

Source postfix `operand[indexExpression]` binds above all binary/unary operators,
chains left-associatively and explicitly parses each index at CUE minimum 0.
The expression program owns scanIndex/hasIndex/advanceIndex/parseIndex/closeIndex
iteration after primary/operand trivia and before binary lookup. No native
postfix loop. `list-open` recognizes opening and a CUE `index-close` scanner owns
required closing/trivia/diagnostic. Existing conservative operand-newline policy
remains, including before brackets. Optional grammar index:{kind,leftField,
rightField} validates eagerly (distinct nonempty fields, neither kind); absence
disables syntax without fallback. Default constructor listIndex/left/right uses
the existing CUE binary builder, not native JSON maps.

Private native index stages check candidates/commits, selected-driver recursion,
pending RHS and completion. Active index stages reject binary/cross-primary
operations; early return cannot expose a half-built postfix. Both drivers and
shared 64 source nodes/32 recursion levels/4096 frame steps stay unchanged. Each
postfix adds one AST node, index parsing one recursive expression invocation;
chaining is iterative. AST evaluation remains 4096 visits/64 active nodes.

Actual pre-index Pratt code stays byte-exact; compiled index-minimum100
control changes parsing. Independent trees/values/errors cover precedence/chain/metadata/stages,
resource bounds/selected drivers, ownership/64/empty/extrema/domain matrices,
error order and CLI atomicity, with pinned exact upstream source/AST probes.
JSON remains build-only, Rust committed, same six artifact-free seeds/features.
Runtime lists still exclude abstract/reference elements; nested values obey
separate depth/logical-node bounds. Go export,
validation/cursor/counter/lookup/builder/encoding/compiler boundaries remain;
no full CUE/bootstrap closure or measured performance claim.
