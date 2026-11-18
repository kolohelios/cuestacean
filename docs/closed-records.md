# Bounded closed scalar records

## Decision and deliberately separate profile

Add opaque immutable `Value::Record(ClosedRecord)`, backed by shared
`Arc<BTreeMap<String,Value>>`. Capacity counts input entries, including duplicates,
before checking uniqueness. A record has at most 64 unique fields; names are
nonempty UTF-8 strings of at most 64 bytes. Field values are concrete
null/boolean/i64/text **only**. No list/record/abstract/reference children,
mutation, unchecked construction, Deserialize bypass or public storage.
This finite flat profile cannot form cycles; it does not extend nested-list
admission/shape limits or decide general struct/reference graph ownership.

`ClosedRecord::new(Vec<(String,Value)>)` checks capacity first, then for each
entry in order: name bytes, duplicate name, scalar value. Duplicate names must
not be silently overwritten. Immutable inspection/len is safe; clones share the
map. Selected text shares its Arc and survives record/input drop. JSON output
uses deterministic byte-ordered field names without rounding integers. Text
value bytes remain unbounded; field width/name bytes do not bound all allocation,
serde parsing, wall time or arbitrary native work.

## Wire and executable policy

JSON AST `record {fields:[{name:string,value:rawScalar},...]}` / instruction
`record {field}` stores literal data, not field-expression ASTs. The entry array
preserves duplicate names until checked. It consumes one AST visit; native
bounded field validation/decoding is a representation codec, not hidden child
AST traversal. Source braces/field declarations/selector syntax and expression
fields were not added by the literal slice. [Record expression control](../bootstrap/record-expressions.md)
now additionally evaluates `{name,value:AST}` entries through the same immutable
scalar-only constructor, shared AST context and checked per-field admission.

Binary AST/rule `recordIndex {left,right}` eagerly evaluates record/key in that
order with the shared context. CUE requires Record/Text, tests field presence,
then chooses checked extraction or missing-field failure. Native always-available
record_get/record_has defensively validate operands; record_count counts fields.
Feature-gated record_index delegates generated control. Same compiler/seed set,
no native field-policy fallback. A compiled record-index-zero control proves
name/arity alone do not provide behavior.

Unary recordLength and generic length/len CUE control admit record_count;
stringLength/listLength stay domain-specific. Generic LengthOperand display now
requires a concrete string or closed collection. CUE top identity precedes
null/list/record isolation before numeric identity. Two closed scalar records
unify only when field names/content match, else RecordConflict: unequal field
sets cannot merge because these are **closed** records, not ordinary open CUE
source structs. Equality/logical/output concrete-domain policies include records.
Ordering/arithmetic remain unsupported. Pinned CUE close({...}) is the oracle.

## Errors, limits and evidence

Errors: RecordCapacity (`closed record exceeds 64 fields`), RecordName {index}
(`closed record field name at index {index} must contain 1..=64 UTF-8 bytes`),
RecordDuplicate {field} (`duplicate closed record field {field:?}`),
RecordElement {field} (`closed record field {field:?} requires a concrete scalar`),
RecordConflict (`conflicting closed record values`), RecordLength
(`length requires a concrete closed record`), RecordIndexOperands
(`field lookup requires a concrete closed record and string`), RecordMissing
{field} (`missing closed record field {field:?}`). Existing Array/Text/MissingField
errors apply to malformed literal payloads as appropriate.

Require reviewed construction/ownership before editing, independent empty/64/65,
UTF-8 byte-name boundaries/duplicates/type/error order, exact MIN/MAX/key ordering/
Unicode, shared text/map lifetime, full mixed-domain semantics and eager-child
errors, actual compiled alternate and typed/unused/arity validation, pinned exact
closed-CUE record/lookup/len/equality/unification comparisons. Same six fresh
artifact-free seeds, build-only JSON/committed Rust, profiles/drift and sandbox
checks; all existing parser/evaluator/compiler/scanner/value limits unchanged.
No general open/abstract/recursive struct solving, source syntax, full CUE/
bootstrap closure or measured performance claim.
