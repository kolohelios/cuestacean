# Native AST specification export contract

Milestone [#169](https://github.com/kolohelios/cuestacean/issues/169), agreed after
#161. Two focused contract reviews preceded parser/evaluator implementation. The target is the unmodified `bootstrap/ast-schema.cue` plus
`bootstrap/ast-atom.cue`, not general CUE or replacement of the six compilers.
The [#171 family extension](native-builder-family.md) reused the initial
language/implementation for all 12 current builder specs with its own artifact and
proof. [#173 lexical bindings](native-let-bindings.md) extend this native subset
without changing those raw builder inputs or their JSON/Rust results. The
[#196 typed prefix-list extension](native-prefix-lists.md) adds boolean values/types
and required-prefix open lists without admitting the full scanner schema. The
[#182 demand-policy artifact cut](native-demand-policies.md) reuses that same
exporter/language for two experimental dispatch programs, with its own three-file
raw cut and proof. The original atom artifact and two-file attestation stay unchanged.

## Boundary and ownership

Use a separate `bootstrap/spec-export` Rust crate. Its production source fileset
contains only Cargo metadata and handwritten implementation sources: no exported
JSON, generated Rust, expected answers, existing compiler-policy modules or
`CUESTACEAN_BOOTSTRAP_JSON`. No external process execution/import/plugin interface.
The CLI accepts two raw source paths, reads bounded input, and publishes JSON only
after successful constraint evaluation and bounded encoding.

This is an explicitly native bootstrap component, not a CUE-hosted solver claim.
Reusable lexical/storage/budget/encoding primitives may remain native. Definition
instantiation, branch compatibility, meet and export-demand policy initially live
in the isolated native evaluator; moving one of those policies behind a checked
CUE graph-event interface is subsequent work, not hidden behind configuration.
Do not route production eager-evaluation failures here. No production syntax or
existing policy/resource limits change.

## Accepted source language

Both sources declare the same ASCII package name. Parse named `#` definitions,
one or more root embeddings, regular identifier/quoted field labels, structs,
fixed lists, typed open lists `[...T]` and `[A, B, ...T]`, parentheses, references to
definitions, predeclared `string`/`int`/`bool`, boolean literals `true`/`false`, scoped [let bindings](native-let-bindings.md), JSON-style
quoted strings, decimal i64 literals, conjunction `&`,
disjunction `|`, and literal inequality `!=`. `&` binds more tightly than `|`.
Comments are `//`; newline/comma separation must not accept missing separators.

Definition labels and quoted ordinary labels such as `"#Foo"` are distinct.
Definition bodies may refer forward to other definitions. Resolve names after
parsing both files; IDs are arena-scoped, never owning reference pointers. Store
finite syntax trees/arenas, not infinitely expanded recursive schemas. Each
instantiation has its own constraint collection and data path; specializing one
use must not mutate a shared template or another list item.

The separately reviewed [#218 tagged-default profile](tagged-default-admission.md)
admits exactly one marked tagged-struct choice with Top under its whole-document
guard. The separately reviewed [#224 scalar class](native-meta-lowering-policy.md)
also admits one concrete literal preference against literal/type/Top, without
relaxing the tagged-struct witness or shared exclusions. Neither class admits
general defaults; preference-free behavior stays unchanged. Full default/closure/
reuse compatibility remains tracked by #175.
The [#250 once-instantiated scalar definition](native-instruction-module-controls.md)
adds exactly one definition/one direct root Ref/one Ref total, a single-field Struct
path to one scalar preference, no siblings/And/lists/other declarations or uses.
It retains the original reference/closedness/evaluation machinery and explicitly
charged admission; all old inline costs/exclusions remain. No external context,
general default-bearing definitions, alias/reuse/closure model or #175 completion.

Explicitly reject unsupported syntax: imports, general defaults, optional/required field
markers, hidden fields, field-value references, general comprehensions/patterns/interpolation,
null literals, boolean inequality, other numeric forms, computed operations and open struct ellipses. Duplicate
named definition declarations are initially Unsupported (their closedness/merge
semantics need a separate contract). Duplicate ordinary fields are conjuncts, not
last-field-wins JSON. Out-of-i64 literals are UnsupportedRange, not CUE conflict.
There is no filename/definition/opcode-specific result construction. Label-length
failures are Limit; `_foo` is an unsupported hidden field while `#_Foo` remains a
definition identifier. Quoted labels are ordinary fields, including `"#Foo"`.

The separately reviewed [#234 finite table class](native-map-comprehensions.md)
adds exact-key field comprehension over one literal-string lexical let map with
pure finite templates. It is static specialization, not general interpolation or
loop evaluation; preparation shares existing work/copy budgets and aggregate syntax
includes all instantiated nodes. Marked/definition contexts remain excluded.
The [#248 multiline literal class](native-instruction-module-text.md) adds LF-only
triple-double-quoted VALUES with exact margin stripping, no backslash syntax/raw
BOM/CR or other controls beyond LF/TAB. Labels/inequality contexts remain excluded;
these are the same literal string nodes/work/copy charges, not general strings,
interpolation, concatenation or changed default admission.
The [#261 scalar-definition projection](native-scalar-projections.md) separately
admits one direct #Definition.field from an original flat, unique, literal scalar
map. Whole-arena exclusions and charged in-place specialization preserve old
no-projection costs; this is not field-value solving, container/default selection
or default/reuse admission. The separate [#260 producer/consumer slice](native-instruction-right-operand.md)
uses that unchanged capability to construct the original R wrapper from whole
independently native Right DATA. Its two-stage adapter and isolated consumer gates
do not widen the projection class or claim native per-use contextual semantics.
The fresh [#266 operand-case publication](native-instruction-operand-cases.md)
uses whole independently native R120 to construct unchanged L/eight cases as whole
4549-byte DATA. A sole-root companion/ordinary adapter needs no new language/CLI
admission. Canonical Go imports whole Group; raw contexts/gates keep SAME unresolved
owners. This is another partial instruction input, not a sixth policy/compiler.
The fresh [#268 literal-container projection](native-literal-container-projections.md)
extends original target validation to finite pure-literal trees. Selected object/list
subtrees inherit fresh recursive closedness; scalar flat-map costs/selector grammar
stay unchanged. No aliases/constraints/default target solving or production edge cut.
The separately reviewed [#276 string concatenation](native-string-concatenation.md)
adds static string-only `+` over original literals/direct eligible projections and
chains, preserving original-target validation and shared work/copy bounds. No
numeric/reference operands, defaults or comprehensions are admitted. This generic
prerequisite changes no production consumer or complete-policy/compiler count.
The [#278 module-program input](native-instruction-module.md) uses it to construct
whole module DATA from independently native Text/First/Step. Canonical Go consumes
a fixed native cohort through a zero-argument boundary; open contexts select raw
owners before argument assembly. Four isolated answer-free shape exports establish
completeness/types, not value coherence: actual constructor paths/bytes and closed
selection establish identity. No native language/schema/guard expansion or sixth
complete policy/compiler follows; full instruction closure and #175 remain held.
The [#280 complete instruction-policy DATA cut](native-instruction-lowering-policy.md)
uses unchanged literal projection/ordinary references for two placement programs,
then disjointly assembles five materialized components. Canonical Go construction
is skipped; conserved raw/schema/closed-cohort probes run in a separate verifier.
SIX complete policy DATA inputs/FIVE native compiler binaries remain, not schema,
PROGRAM/joined-seed closure or broader #175 admission. No language/guard expansion.

## Readiness and recursive schema semantics

Normalize the conjunction at a demanded data path into compatible shape/type/
literal constraints. Expand definition references only at that path, carrying
recursive closedness from a definition into nested inline shapes. Referencing a
definition does not globally refine it. An ordinary data struct is open; every
closed definition constraint restricts admitted fields. Check incompatibility
rather than throwing away a constraint that cannot yet produce a concrete value.

Merge struct fields by label and meet all conjuncts. Absent singleton fields may
be supplied (`formatVersion:1`); abstract string/int fields remain Incomplete.
An explicit value conflicting with a singleton is Conflict. Closed lists require
equal lengths; typed open-list constraints apply to each supplied element without
expanding the tail into infinitely many elements. Pinned CUE exports `[...string]`
as `[]` when no items are supplied: project the known prefix without converting
its retained open constraint into a permanently closed list. Required-prefix lists
select the longest known prefix if no fixed list supplies the length. Mixed
fixed/open lists use the fixed length, reject shorter-than-prefix lengths and
constrain every supplied item with every applicable prefix/tail.

Disjunction explores compatible branches with a shared budget. Eliminate
conflicting alternatives; deduplicate complete results only when exact JSON AND
canonical container bounds agree ([#198 readiness](native-container-readiness.md)).
Equal projected JSON can conceal open/closed alternatives. Different bounds,
distinct complete results, or a compatible incomplete alternative leave export
Incomplete; never choose a branch just because its JSON is already concrete. Unsupported/resource errors
are not alternative contradictions. Check shallow literal/type incompatibility
before recursive demands, and do not let one incomplete field conceal another
field's contradiction. No field named `op` has privileged semantic treatment.

A recursive template such as `#BuildExpression -> #BuildObject -> fields[].value`
is productive against the finite input object tree. Guard reference expansion at
the same data path to detect unproductive cycles separately; initially classify
such unsupported cycles explicitly rather than applying the arithmetic SCC rule
from #162. No general value-reference cycle or default solver is implied.

Keep residual constraints until the export demand succeeds. A returned partial
domain is not exportable JSON. Conflict, Incomplete, Unsupported syntax/semantics,
UnsupportedRange and Limit are separate errors; no JSON on semantic failure.
Within supported finite inputs and sufficient budget, declaration/field order
must not change values or classification. First detailed diagnostic/work count
need not be order-invariant.

## Proposed module interface and budgets

`parser::parse(schema:&str,program:&str)->Result<Document,Error>` returns a fully
name-checked arena plus root constraints. Public `export(schema,program)` evaluates
and returns deterministic pretty JSON (sorted keys, exact integers, preserved
array/string values, trailing LF). CLI errors go to stderr with nonzero exit.
Tests may expose an explicitly lowered work allowance, never raise hard caps.

Shared types in `model.rs` (crate-private except Error):

```rust
pub type Id = usize;
pub enum Type { String, Int, Bool }
pub enum Expr {
    Literal(serde_json::Value), Type(Type), NotEqual(serde_json::Value),
    Struct(Vec<(String, Id)>), List(Vec<Id>, Option<Id>), Ref(String),
    And(Id, Id), Or(Id, Id), Binding(Id), Top, Marked(Id),
    Name { scope: usize, excluded: Vec<usize>, name: String },
}
pub struct Document {
    pub nodes: Vec<Expr>, pub definitions: std::collections::BTreeMap<String, Id>,
    pub roots: Vec<Id>, pub marked: bool, pub lets: usize,
}
pub enum Error {
    Syntax(&'static str), Unsupported(&'static str), Range, Unknown(String),
    Conflict, Incomplete, Limit(&'static str),
}
```

`eval::evaluate(&Document,work_limit:usize<=4096)->Result<serde_json::Value,Error>`.
The library exports `export` plus `export_with_limit` for lowered-budget tests.
Derive Debug/Clone for syntax, Eq/PartialEq for Error and Copy/Eq for Type.
Parser and evaluator must agree exact Rust definitions before concurrent edits.
Use non-owning IDs for references; recursive evaluator frames are bounded by the
export-depth limit and syntax parsing by its own depth limit.

Initial isolated caps: 64 KiB combined source, 4096 tokens, 2048 expression nodes,
64 definitions, 64 let bindings, 64 fields/items per explicit container, 1..64-byte field/definition
labels (including the leading `#` for definitions), 32 syntax levels, 32
root-inclusive demanded output levels, 4096 evaluation steps, 2048 output value
occurrences across trial branches, 64 KiB copied value-string bytes and 64 KiB
encoded output. Charge branch/ref expansion, constraint
inspection and recursive demands cumulatively; no per-branch reset. Fail before
publishing output. These logical bounds do not establish total allocator, stack,
wall-time or adversarial-input safety. Some defensive caps can be dominated by an
earlier guard: materializing 2048 values requires more than 4096 charged evaluation
steps. Do not raise another guard merely to make that boundary reachable in tests. Existing production guards stay unchanged.

## Independent evidence and attested build cut

1. Capture pinned upstream behavior for open lists, disjunction incompleteness,
   repeated ordinary fields and recursive closedness before implementation.
2. Compare the real two-file export with handwritten expected JSON and Go CUE in
   separate tests. Mutate keys/names/types/operations, field/declaration order,
   nested object instances and incompatible/incomplete branches. Preserve the
   downstream boundary: an argument name `"missing"` is data and exports normally.
3. Build the exporter from the minimal production fileset above, with tests and
   expected answers excluded. A separate test derivation may include those inputs
   and Go CUE; it must not feed the production exporter/export derivations.
4. The native artifact derivation receives only that exporter and the two raw CUE
   sources; run with empty environment/unusable PATH, sandbox/no network and no
   ambient checkout. Inspect full build derivations for absence of Go CUE and
   existing `jsonExports`/policy derivations, not just runtime references. Rebuild
   actually rather than only substituting an output. No baseline/template cache.
5. A separate comparison/downstream derivation may depend on Go CUE and the existing
   AST compiler. Compare exact JSON/bytes, then feed native JSON into that compiler
   and compare generated Rust with the independently maintained builder golden.
   Do not call the downstream Go-dependent check an entirely native bootstrap.
6. Keep all five existing checks, fresh six artifact-free compiler binaries,
   feature isolation, repeated generation/drift and independent source/value
   checks. Add isolated crate formatting/Clippy/tests and native closure checks.

## Build and inspect the independent path

```sh
nix run ./infra/tooling#spec-export -- bootstrap/ast-schema.cue bootstrap/ast-atom.cue
nix build --no-link ./infra/tooling#native-ast-atom
nix build --no-link ./infra/tooling#checks.x86_64-linux.native-spec-tests \
  ./infra/tooling#checks.x86_64-linux.native-spec-closure
```

`specSources(false)` contains Cargo metadata and eight handwritten source files
(ten total after #261 adds bounded scalar projection specialization)
only. The test-source variant adds oracle fixtures/expected-answer tests, but is
not an input of `specExporter` or `nativeAtom`. The artifact receives a separate
source fileset containing exactly the two CUE files. `nativeBuildClosure` roots
`closureInfo` at both **derivation paths**, exposing their complete build graph.
`nativeVerification` rejects Go CUE derivation/source and existing JSON-export
inputs in that graph, then separately performs pinned oracle and downstream Rust
comparisons. Its `native-build-paths`, `native-build-registration` and
`native-build-roots` outputs retain the complete path/reference graph and exact
attested roots. The denylist checks pinned identities, not every conceivable
foreign dependency; retain full-graph/source-allowlist inspection too. These checks
intentionally have different dependency closures; do not attest the
oracle/check package as Go-CUE-free.

Review findings closed: a JSON slash escape initially rejected by the parser is
now accepted and compared upstream. Targeted tests distinguish copied-string,
encoded-byte and output-depth limits, cumulative alternatives, declaration/field
ordering, recursive instance isolation and explicit versus merged container width.
Native production code contains no process execution or pre-exported policy load.
Milestone #169 validation on x86_64-linux: nine integration tests pass in debug and release,
including pinned upstream comparisons; both crates pass full tests and strict
all-target Clippy. Preflight passes 13/13 (7 inapplicable checks skipped). The five
existing Nix checks remain, alongside two native checks; actual native build/test/
artifact/verification and existing sandboxes pass, with explicit deterministic
rebuilds of the independent path and JSON export. The inspected native graph has
1407 derivations/3392 store paths, excluding pinned Go CUE and policy-export inputs.
These repository checks are distinct from scratch/read-only reviews;
other architectures were omitted, not validated.

This original milestone achieved one useful native specification export only.
Subsequent scoped capabilities are linked above. General references,
open structs/defaults, wider builtins, full self-hosting and performance remain
deferred; subsequent changes require a new bounded contract and evidence.
