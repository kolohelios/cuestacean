# Bounded nested closed-record values

## Ownership decision (review before representation edits)

Extend the opaque immutable ClosedRecord value to contain concrete null/bool/i64/
text or already-validated closed records. Keep lists and abstract/reference values
excluded; lists still exclude all records. Only recordExpression evaluation and
owned constructors gain nested values. Raw `record {fields:[{name,value:rawScalar}]}`
codec/schema stays scalar-only. No source structs, open record merging, reference
identity, mutation or general solving is introduced.

Replace the private Arc map tuple with private immutable shared map plus certified
cached shape. Clones share the map and copy the certificate. Parent construction
only accepts completed immutable children: safe construction cannot insert a
parent into its descendants, so sharing forms an acyclic DAG, not Arc cycles.
No public fields, metadata getters/setters, unchecked constructor or Deserialize.
Selected child records retain their own storage, not their parent map. Mutable/
reference graphs still need a different ownership design.

## Separate value bounds and validation order

Existing capacity remains 64 input entries (including duplicates), unique 1..64-byte
UTF-8 names. New shape bounds match lists numerically but remain a separate record
profile: 32 root-inclusive value levels and 4096 logical value nodes. Scalar/empty
record shape is depth 1/nodes 1; a record has depth 1+max(child depth), nodes 1+sum(child
nodes). Charge each field occurrence, not each unique Arc allocation. Keys/text
bytes are not logical nodes; these bounds do not bound total bytes, allocation,
serde work, wall time or every native stack frame.

Constructor order: capacity first, then each name bytes/duplicate, concrete child
type, prospective depth, prospective nodes. RecordDepth displays `closed record
exceeds 32 value levels`; RecordNodes displays `closed record exceeds 4096 logical
value nodes`. RecordElement becomes `closed record field {field:?} requires a
concrete scalar or closed record`. The flat raw codec continues to reject object
payloads even though the broader constructor error names closed records.

Parent construction visits only direct fields, reading each child certificate
without walking its subtree. RecordFrame caches prospective accumulated shape;
append validates capacity/name/duplicate/type/depth/nodes before taking pending
name/value or changing map, certificate or stage. Failed append is transactional;
an earlier admission/shape error precedes later fields/callbacks. Finish invokes
ClosedRecord::new independently again. No unfinished or invalid value escapes.
Native certificates/checked construction do not replace CUE admission policy.
Callbacks/custom policies are trusted, not sandboxed.

AST budget 4096 visits/64 active nodes and record-control 4096 steps/frame remain
unchanged and independent; no parser/compiler/scanner limits are raised. A value
shape guard may reject before these budgets. No source/parser/compiler changes
or new compiler/seed are needed.

## Executable CUE and independent evidence

CUE record control explicitly admits record-valued children; equality, closed
unification, lookup, length, logical concreteness and exact recursive output use
existing policies/kernels. Ordering/arithmetic remain unsupported. Keep the real
previous scalar-only record control byte-identical as a compiled alternate, plus
existing reverse control and selected evaluator/shared context propagation.

Require independent 32/33 depth and 4096/4097 logical-node boundaries, shared-DAG
amplification, width/name/type/depth/node precedence, transactional failures,
shape-before-later-callback errors, final-constructor defense, selected-child
lifetime, nested output/equality/conflict/length/chained field lookup, historical
scalar admission versus default/reverse, unchanged flat codecs/list exclusions,
shared AST limits, exact pinned closed-CUE comparisons and all six fresh artifact-
free seeds/profiles/repeated drift/five sandbox checks. No full CUE/bootstrap
closure or measured performance claim.
