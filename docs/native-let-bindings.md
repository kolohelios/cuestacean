# Scoped native specification `let` bindings

## Reviewed contract

Milestone [#173](https://github.com/kolohelios/cuestacean/issues/173), after #171.
Two focused contract reviews preceded implementation. Next prerequisite: admit lexical `let` bindings in the isolated native
exporter. Real `ast-lowering-fragments.cue` uses `Copy`/`Owned` template bindings;
`identifier.cue` uses file-local bindings for byte tables. This slice implements
neither imports/builtins nor defaults/patterns/optional fields, so **does not close
those complete specification exports**. Existing production source syntax stays
unchanged. The native semantic policy boundary remains explicitly native.

### Identity and lookup

- Each input file has its own lexical root scope; nested structs introduce child
  scopes. Definition names remain package-wide as before. A definition captures
  the lexical context of its declaration, not that of a later referencing file.
- Admit `let name = expression` declarations at file/struct scope. They add no
  exported field. Names are ASCII identifiers, 1..64 bytes; `_name` is allowed as
  a binding, unlike a hidden data field. Reject reserved declaration keywords.
- Collect all declarations before resolving names, so forward references to other
  bindings work. Resolve a name to an immutable binding/body ID, not a caller-time
  string lookup. Different uses meet constraints independently; no global template
  mutation or values cached across export invocations.
- A binding is excluded from its own initializer, including nested expressions;
  an outer binding of the same name may still resolve. Nested initializer contexts
  retain all such exclusions. Other forward bindings remain visible.
- Search lexical bindings before predeclared `string`/`int` (and `bool` since
  [#196](native-prefix-lists.md)). Other predeclared types remain Unsupported
  when not shadowed. Thus `let string=1`
  shadows the predeclared type, while `let string=string` may use the outer builtin.
  Quoted field labels do not declare identifiers or collide with binding names.
- Unquoted field references are still Unsupported, not a new value-reference
  solver. Reject visible alias/unquoted-field collisions across related lexical
  scopes, as pinned CUE does. Initializer exclusions also apply to collision
  admission: `let x={x:1}` is valid because its own binding is not visible there.
  Do not suppress collisions with other still-visible bindings of the same name.
  Never resolve a field accidentally as an outer let
  or builtin. Do not treat a quoted `"x"` field as an identifier declaration.
- Duplicate bindings in one scope and unused bindings reject explicitly. Static
  reference uses inside initializers/definitions count, not just executed paths.
  Undefined identifiers and cross-file direct binding access reject before export.
  Definition references can legitimately carry a file-local binding into another
  file; the binding itself is not package-global.

### Open values, closed definitions and cycles

A binding reference does **not** itself close a struct. `let X={a:1}; X&{b:2}`
exports both fields. Recursive closedness inherited from a containing definition
still applies at the use site: binding expansion carries existing closure groups
but does not create a definition-closure group. Preserve residual conjunctions,
disjunction readiness, separate instances and all existing error precedence.

Initially reject any dependency cycle involving a binding as Unsupported at
admission, including cycles through definitions or nested data. Pure recursive
definition templates remain governed by the existing demand-depth/productivity
contract. This is deliberately narrower than CUE: pinned CUE accepts a productive
`let A=#D; #D:{children:[...A]}` with an empty child list. Do not label all such
cycles contradictions or claim general let recursion support. Direct self-reference
without an outer/builtin binding is undefined, not a fallback to itself.

### Implementation interface and resources

Add a crate-private `bindings` module for scope collection/name resolution. Keep
`Document {nodes,definitions,roots}` and public export/error APIs; use the existing
Unknown category for undefined references and distinguish Unsupported/Limit from
Conflict/Incomplete. Parser/evaluator must agree the exact temporary/resolved node
variants before concurrent edits. The shared additions are:

```rust
// Expr additions; Id is an existing arena node index.
Name { scope: usize, excluded: Vec<usize>, name: String },
Binding(Id),
// bindings.rs collection types (crate-private):
struct Scope {
    parent: Option<usize>, bindings: std::collections::BTreeMap<String, usize>,
    fields: std::collections::BTreeSet<String>, // unquoted identifiers only
    excluded: Vec<usize>, // active initializer binding IDs, not name strings
}
struct Binding { name: String, scope: usize, body: Option<Id> }
// resolve(&mut Document, &[Scope], &[Binding]) -> Result<(), Error>
```

Fields on these collection structs are `pub(crate)` for parser construction.
Resolve Name to Binding(body ID) or Type, preserving initializer exclusions;
count static uses and reject unsupported collisions/cycles/unused declarations.
After resolution no unresolved name may reach evaluation. Binding references expand their body under the current demand path,
without adding closure. Source/evaluation arenas own finite nodes; references are
non-owning IDs, never cyclic owning pointers.

Keep all current source/token/node/syntax/output/evaluation/copy limits unchanged.
Add a separate hard cap of 64 binding declarations per document. Scope count is
bounded by parsed structs plus the two file roots; ancestry by syntax depth.
Resolve/cycle-check using bounded ID arrays/iterative traversal, not unbounded
recursive expansion. No raised guards, stack overrides, per-binding budget resets
or silent Go CUE fallback. Newly needed implementation sources must be explicitly
allowlisted in the production Nix fileset; oracle tests remain outside it.

### Independent evidence

Pinned CUE probes established forward binding resolution, lexical capture and
shadowing, initializer exclusion/outer capture, file-local rather than package
binding visibility, unused/redeclared errors, open let structs, inherited
closedness and quoted-label behavior. Tests must compare independent expectations,
native results and pinned upstream, including negative classification boundaries.

The normal devshell builds the production JSON bundle, which now depends on the
exporter. During incomplete exporter edits, use the pinned toolchain without that
bundle dependency (this is development tooling, not an artifact fallback):

```sh
nix shell --inputs-from ./infra/tooling nixpkgs#cargo nixpkgs#rustc \
  nixpkgs#rustfmt nixpkgs#clippy nixpkgs#cue --command \
  cargo test --manifest-path bootstrap/spec-export/Cargo.toml
```

Exercise representative `Copy`/`Owned`-style instruction trees without pretending
the whole lowering policy can export. Test declaration/field/file permutations,
independent uses and refinements, builtin shadowing, forbidden field references,
unknown/cyclic/unused bindings, 64/65 declarations and cumulative work/copy budgets.
Include the productive binding/definition cycle that CUE accepts but this profile
explicitly rejects. Semantic failures publish no JSON.

Rebuild both native artifact cuts/full dependency proofs and all twelve existing
builder exports; require unchanged JSON and generated Rust. Run both Rust suites,
debug/release, strict Clippy, all eight Nix checks, six artifact-free compiler
binaries, feature isolation/regeneration/drift and upstream evidence. This is a
bounded prerequisite toward identified closures, not general CUE scope solving,
completed policy/import closure, full self-hosting or measured performance.

## Validation and review findings

Review exposed a collision-admission bug: initializer lookup excluded its binding,
but collision checking initially did not. Pinned `let x={x:1}` and nested/predeclared
name cases formed a failing regression before the per-scope binding-ID exclusion
fix. Other visible bindings still collide, and field references remain unsupported.
Nested initializer coverage requires two simultaneous exclusions, not just the
nearest binding. Cyclic/undemanded-binding limitations remain explicit negative
profile tests even where pinned CUE exports successfully.

On x86_64-linux, 21 integration tests plus one defensive unit test pass in debug
and release, including nine binding tests and all previous native resource/builder
cases. Both Rust crates pass full tests and strict all-target Clippy. Pinned
preflight passes 13/13 (7 inapplicable checks skipped); all eight Nix checks and
both independent artifact paths pass actual sandbox builds and explicit rebuilds.
Fresh six compiler binaries, profiles, regeneration/drift and upstream comparisons
are retained. The full production JSON tree remains byte-identical to #171 and
all generated Rust remains unchanged. The production exporter source contains
exactly eight allowlisted files (Cargo metadata plus six Rust files), excluding
expected answers/tests/policies. Both rebuilt native build graphs exclude pinned
Go CUE/source/JSON-export identities and retain their full graph evidence.
Incompatible aarch64 systems were omitted, not validated.
