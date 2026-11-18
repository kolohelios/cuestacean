# Bounded mixed closed collection values

## Ownership decision (review before edits)

Allow validated immutable closed lists and records as children of either container
through owned constructors and JSON listExpression/recordExpression evaluation.
Both raw literal codecs/schemas stay scalar-only. No source structs, open/abstract
collections, references, mutation or general solver is introduced.

Replace separate ListShape/RecordShape implementations with one private ValueShape
certificate (depth/nodes). Both opaque Arc-backed containers cache it; scalar and
empty-container shape is 1/1. Safe construction only accepts already-built immutable
children, so alternating kinds still form an acyclic shared DAG. No public fields,
metrics, setters, unchecked constructors or Deserialize path. Cloning preserves
certificates and shared storage; selecting a child does not retain its parent.

## Combined bounds, not per-kind resets

Existing width remains 64 direct entries/elements, with record names unique and
1..64 UTF-8 bytes. Value depth remains 32 root-inclusive levels, logical nodes 4096.
Every collection root and scalar occurrence counts; keys/text bytes do not. A
parent has depth = 1 + max(child depth), nodes = 1 + sum(child nodes), even across
alternating kinds and repeated shared Arcs. Never reset depth/node accounting at
a list/record boundary. Parent construction reads child certificates without
walking their subtrees.

Validation order stays capacity first; for each record entry name/duplicate, then
child type, prospective depth, prospective nodes. Lists check child type/depth/nodes
in element order. Parent kind selects existing ListDepth/ListNodes or RecordDepth/
RecordNodes diagnostics. Abstract children still fail Element before shape limits.
Element messages now require a `concrete scalar or closed collection`; collection
means concrete closed list or record. Raw codecs still reject array/object payloads
regardless of that broader constructor diagnostic.

Both checked frames validate all prospective invariants before consuming pending
state or changing buffers/maps/certificate/stage. A shape failure precedes later
callbacks; repeated failed append is transactional. Finish independently reconstructs
through the opaque constructor. Native representation checks do not replace CUE
admission. Callbacks and custom policies remain trusted, not sandboxed.

No AST/control/parser/compiler/scanner guard is raised: AST 4096 visits/64 active
nodes, each collection control 4096 steps/frame. Shape bounds do not bound total
string/key bytes, allocation, serde, wall time or every native stack frame.

## CUE policy and independent evidence

Default/reverse CUE controls explicitly admit both collection kinds, using the same
existing structured compiler/seed graph and checked frames. Preserve real prior
homogeneous list/record controls byte-identically as compiled alternates. Existing
scalar-only alternates must also stay byte-identical. No native policy fallback,
parser/compiler changes or new seed. Existing equality, closed unification, length,
logical concreteness, typed indexing and recursive exact output apply; root lists
and records remain distinct domains (no numeric identity or index coercion).

Require independent alternating 32/33 levels and mixed shared 4096/4097 logical
nodes with both possible root kinds; parent-specific errors, type/name/width/depth/
node precedence, repeated transactional append, shape-before-later-callback and
final-constructor defense. Check ownership/parent release across kinds, chained
typed indexing, exact MIN/MAX/Unicode output, equal/conflicting mixed contents,
unchanged flat codecs and shared AST/control budgets. Compare new default/reverse
with actual homogeneous/scalar historical controls. Pin CUE close at record nodes,
exact output/closed unification and semantic conflict failures. Full Rust/strict
Clippy, preflight/five real sandbox checks, six artifact-free seeds, feature profiles
and repeated generation/drift remain required. No bootstrap-closure/performance claim.
