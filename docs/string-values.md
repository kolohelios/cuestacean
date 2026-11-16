# Owned string scalar design

This slice expands the **JSON AST evaluator**, not the source-language parser.
Concrete `string` nodes carry a string `value`; payload-free `stringType` nodes
represent abstract CUE `string`. Source string spelling/escapes/interpolation,
structs/lists, references, defaults and general solving remain outside the profile.

## Ownership boundary

`runtime::Value` stops being `Copy`. Concrete text uses `Text(Arc<str>)`; abstract
string uses `String`. Integer, boolean and bound payloads remain unchanged.
`Clone` is explicit and cheap for shared text, with content-based equality—not
pointer identity. Values own their text independently of the parsed JSON document;
no borrowed input can escape an execution call. No unsafe arena or global interner
is introduced. This does not settle graph/node identity or cycle ownership for
future structural values and references.

The public unification API still consumes two Values and returns an owned result.
Tag, endpoint and failure primitives inspect `&Value`; only an explicitly selected
input is cloned for a result. CUE compiler policy owns the emitted borrow/clone
spelling. The reference solver borrows its immutable input pair while traversing
control. Arithmetic continues consuming operands, rejecting non-integers after
left-first child evaluation. Existing copy-dependent tests must clone deliberately.

## CUE-owned semantics

CUE solver control owns string-type identity, equal-string selection, unequal-string
conflict, and mixed string/boolean/numeric-domain rejection before numeric identity.
Native kernels only expose tags, structural/content equality, errors and projections.
Strings have no integer endpoints; that absence is not permission for numeric
unification. Unicode data is preserved without normalization or coercion.

`execute_json` and both JSON evaluator CLIs emit exact JSON string scalars; legacy
integer `execute` rejects concrete strings. Abstract string output is incomplete.
Ordinary integer/boolean source behavior remains unchanged. No string concatenation,
comparison operator, source scanning or native source fallback is added here.

## Evidence and bootstrap

Independent expectations cover empty/Unicode/escaped/control-containing strings,
identity/equality/conflict, every mixed-domain pairing, incomplete output, ownership
across parsed-input lifetime and arithmetic error ordering. Reference and generated
rule execution plus compiled/reference CUE control are compared; known expectations
remain necessary because kernels are shared. Pinned Go CUE probes check supported
scalar semantics. Existing source differential fixtures stay unchanged.

Six fresh seeds still omit generated semantics/frontend; build-only CUE JSON and
committed Rust/drift policy remain. Existing unification goldens intentionally gain
borrows/clones and string-domain branches. This is an owned scalar foundation,
not full CUE compatibility, self-hosting, graph ownership or measured performance.
