# Bounded native table comprehensions / Pratt tables (#234)

## Actual dependency and delivery boundary

Remaining Go-built Pratt/instruction policies use finite map comprehensions. Isolating
the unification compiler instead requires runtime/error coupling changes; do not
silently widen the existing kernel/interface cuts. Select the actual Pratt maps:
MinimumCalls/SelectorCalls and THREE existing field comprehensions. Deliver a native
pratt-lowering-tables.json module consumed by the still-Go-built complete policy.
Four COMPLETE policy inputs/THREE isolated compilers stay the count; no fifth/full
Pratt policy, fourth compiler, schema/general-comprehension/default or #175 closure.

Read-only pinned Go exports the raw relocated module to2779 bytes, identical to the
original policy's aliases plus selectorAction/parseMinimum cases. The historical
preimplementation #232 binary rejects it (unsupported string escape/no JSON).
This is a regression target,
not captured expected data or production seed. Authoritative two literal maps and
three executable CUE comprehensions relocate unchanged; NEVER hand-expand tables,
move their semantic projection into jq, or give names/opcodes/filenames privileges.

## Exact new specification-language class

Ordinary, preference-free documents only. Recognize a struct whose SOLE member is:

    {for key, value in Table {"\(key)": TEMPLATE}}

Table is a normal lexical name resolving through existing scope/initializer rules
to ONE immutable let body that is a syntactically explicit, duplicate-free struct
of required regular fields with literal STRING values. No alias chains, definitions,
field references, And/Or, types/Top, optional/pattern/open fields, computed sources,
lists, imports/builtins or unresolved values. Empty maps allowed; at most64 fields,
existing nonempty1..64-byte labels. Source name resolves OUTSIDE iterator scope.
Both declared iterator names are distinct normal ASCII binding identifiers; value
may be blank `_` (cannot be referenced), key may not. Iterator value need not be
used. Reject iterator names colliding with visible ordinary lets or related-scope
unquoted fields, including declarations collected later; source lookup uses the
outer environment. Pinned CUE rejects `for T,v in T` when outer let T exists, so
this class does NOT claim iterator shadowing of ordinary bindings.

Exactly ONE `for`/two identifiers/ONE exact key interpolation label and ONE yielded
field. No clauses/guards/else, nesting, sibling fields/comprehensions or list yield.
Key interpolation raw spelling exactly `"\(identifier)"` with the KEY binder, not
arbitrary string interpolation/prefix/suffix/expression. Ordinary escaped backslash
strings/comment text remain ordinary; using a key-template token elsewhere rejects.
TEMPLATE is finite literal string/bool/i64 data, static-label duplicate-free structs,
fixed lists and references ONLY to the key/value iterators. No outer references,
declarations/definitions/types/Top/constraints/marks/nested comprehension/other
expression. Reject unquoted template fields colliding with either iterator name
at any nested scope; quoted labels remain data. Eagerly validate even empty-map
body and unused/nonselected syntax. Definitions/marked syntax anywhere in a document
with comprehensions remain Unsupported, preserving all existing marked-profile
exclusions. Ordinary documents without comprehensions retain old behavior/budgets.

This is a finite static specialization of CUE's for-clause binding/key interpolation:
known required field set, concrete string key/value, pure body, unique output labels.
Each row receives independent immutable key/value substitution; output iteration
order is unobservable apart from deterministic serialization. No mutation or cache
across source instances/invocations, source-name or expected-answer dispatch. It
is not general scope/graph/default solving. Preserve normal lexical let capture,
initializer exclusions/unused/cycle/field-collision behavior for the source name.

The subsequent [#238 string-template class](native-instruction-unary-tables.md)
adds exactly one key/value string placeholder with decoded literal affixes in
TEMPLATE VALUES only. Exact key labels and all source/scope/global exclusions stay;
its two extra append charges share the same work/copy budgets. This is not general
interpolation or runtime expression evaluation.

## Concrete private interfaces and accounting

One kernel/build owner prevents parser/model/expansion drift. Proposed interfaces:

- model.rs: Template::{Literal(Value),Key,Value,Struct(Vec<(String,Template)>),
  List(Vec<Template>)} and Expr::Comprehension{source:Id,template:Template,scope:usize,
  key:String,value:Option<String>}. Retain iterator declaration context for eager
  collision checking; Parser
  classifies iterator names into Key/Value locally; templates contain no unresolved
  scope lookups. Document retains comprehension count, preparation steps/strings
  and aggregate syntax count. These are private; public export/Error ABI unchanged.
- parser.rs: exact-key token, bounded template parser, sole-member comprehension
  parse. Source Name uses existing scope/exclusions. Parse counts BOTH original
  Expr AND template nodes against2048; carries conservative template height into
  existing32 syntax-depth checks. Recognize reserved/blank binders deliberately.
- bindings.rs: original name resolution/usage/cycle checks remain; reject iterator
  collisions against collected ordinary scopes/bindings before specialization,
  respecting active initializer binding-ID exclusions retained in scope context.
  Comprehension graph edge is source Id only; template has no lexical body IDs. Cycles reject
  BEFORE expansion, including indirect source dependencies/unused declarations.
- NEW comprehension.rs: after existing resolution, lower ALL admitted occurrences
  to ordinary Struct nodes by instantiating the finite template per source field.
  Validate ALL sources against the ORIGINAL resolved AST BEFORE replacing ANY
  comprehension; an earlier expansion must not make a computed source look literal.
  Resolve source only as immediate Binding->literal-string Struct. No interpreting
  general Expr to discover source rows or cloning arenas/source maps/templates
  speculatively before charging. Use moves/bounded ID descriptors, not unbilled
  string snapshots. Validate the full class before publishing;
  preserve original comprehension signal/lets for whole-document exclusions.
- lib/parser call expansion with export_with_limit's allowance (hard max4096).
  Evaluator starts remaining=limit-preparation_steps and strings=preparation_strings;
  no per-row/binding/path reset. Unexpanded variants reject defensively. No loop
  reaches ordinary evaluator or activates general interpolation/variable semantics.

Keep EVERY existing cap. Aggregate syntax cap2048 includes original Expr/template
nodes PLUS every freshly instantiated Expr, even after dropping templates; no arena
reuse refund. The generated OUTER Struct replaces the existing comprehension node
in place; every freshly instantiated template Expr is additionally charged.
Generated containers/items cap64, depth32. Copy strings/labels through
shared64KiB cumulative copied-string allowance, then ordinary evaluation continues
charging it; final encoded output<=64KiB. Source/token/bindings/definitions unchanged.

Precise preparation work: 1 per comprehension +1 source-binding inspection +1 source-
struct inspection; EACH row costs1 field inspection +1 literal-value inspection +1
emitted field, plus1 per instantiated template node. Charge BEFORE the operation.
Thus per comprehension3 + rows*(3+template_nodes), including empty-map cost3. Body
validation/syntax limits still eager; all row/string allocation has checked bounds.
Generated label/key/value/literal/static-field strings charge bytes on each copy,
including copies later charged again during ordinary evaluation. Same lowerable4096
ledger covers preparation AND evaluation. No guard raises, stack overrides, missed
nonselected/empty work or expected-cost discovery used to choose success semantics.

## Canonical raw owner and actual consumer

New bootstrap/pratt-lowering-tables.cue holds UNCHANGED MinimumCalls/SelectorCalls
literal lets and aliases/selectorAction/parseMinimum instruction bodies containing
all THREE for clauses. Explicit brace root, no defaults/schema/version. Companion
native-policy-package.cue unchanged. No committed/captured JSON/generated Rust.

Existing pratt-lowering-fragments.cue removes those two local tables/three generated
maps and two full case bodies; retains L/R/ALL OTHER original cases/default add
unchanged. Go #PrattLowering still #LoweringPolicy, now requires selectorAction,
parseMinimum and all seven original alias fields via `_` (not filling defaults).
Go production/full-policy and no-increment override supply native table JSON before
context; schema/remaining policy still evaluated by Go. Missing provider must be
Incomplete/no JSON. Separate raw oracle loads unresolved raw table CUE before context.
These table values are fixed DATA; unlike default artifacts, no unresolved preference
is moved across the module boundary. Do not freeze or alter raw add preference.

Nix new native-pratt-tables package/check, raw cut exactly TWO CUE files; exporter
production cut deliberately grows8->NINE with new expansion module. Constructor
receives only selected exporter/raw source, repeats export with cleared environment,
unusable PATH/owned HOME/absolute readable executable tool. Bounded output before
publication; no Go/full bundle/executor/compiler/oracle/golden/expected inputs.
All constructor source files explicit; test sources/fixtures admitted only tests=true.
All older native DATA/249 JSON/12 builder outputs/generated Rust stay byte-identical;
exporter-dependent derivations/compiler ELF identities legitimately change. Full
Go/typed comparisons remain verification, never feed native construction.

## Independent evidence / gates / ownership

Evidence owner ONLY new tests/map_comprehensions.rs and tests/pratt_tables.rs. Derive
complete arbitrary-table/key/value/template output and exact target module trees
independently, not captures. Mandatory Cargo native CLI + existing pinned cue/jq
PATH discovery before clear-env; no new compiler/artifact provider ABI. Nix verifier
owns actual consumer/full-policy/golden/source-effect/typed no-Rust evidence.

Require pinned CUE both orders/renaming/capture/source-name collision rejection/string escapes/
Unicode and byte labels/empty maps/unused value/blank binder/three comprehensions/
independent reuse. Exact-limit/recovery tests for TOTAL preparation+evaluation work,
syntax/materialized depth/nodes/items/copy/output; lowered work must not reset and
empty map must not hide bad body. All excluded contexts, unknown/cyclic/unused source
and collisions remain exact setup/Unsupported/Limit vs Conflict/Incomplete categories.
Marked documents containing comprehension anywhere must never become supported.

Target complete seven aliases/two case trees; unmodified compiler policy/full JSON
and all Pratt/primary goldens/no-increment compiled adversary unchanged. Independent
source key/method/template variants -> native module -> Go policy -> actual Pratt
compiler -> COMPLETE predicted Rust. Wrong table/body syntax fails native class;
malformed materialized DSL rejected schema/typed consumer with no Rust. Retain all
old88 exporter/four profiles and135 research/four profiles, both full Rust/default
all-target strict Clippy debug-release, BOTH genuine façade feature matrices/fresh
modules/separate rebuilds. EIGHTEEN checks/FIFTEEN packages actual build AND rebuild.

Fresh recursive derivation AND --requisites --include-outputs inventories exclude
current Go CUE/source/full bundle/legacy+joined/current executor/BOTH oracle outputs-
drvs/provider env from native construction. Raw table2/exporter9/core8-9/scanner7-8/
experimental6/other raw cuts precise. Original core feature path native, scanner
feature path Go-backed verification only. Six fresh compilers/provenance/isolation/
drift/source/value, pinned Shaka/whitespace/links, two immutable final reviews,
retire workers BEFORE PR, exact-head protected rebase CI through confirmed MERGED.
Only validated architectures; coherent approximately<=1500 changed lines, no goldens.

Kernel/build owner ONLY spec-export/src/{model,parser,bindings,eval,lib,comprehension}.rs,
flake.nix, new pratt-lowering-tables.cue and pratt-lowering-fragments.cue. No executor/
Cargo/dependency/composer/schema/shared runtime edits. Coordinator docs/integration/
source/baseline/full graphs/validation/CI. Two stable-main isolated workers max,
immutable handoffs/new children for fixes/overlays, no saved amendment/rebase onto
mutable coordinator. Format only owned files. Preserve Apache/upstream notices.

Implementation GO: BOTH independent focused semantic/interface/resource and
source/consumer/evidence reviews found no blockers and agreed the exact static
class/private interfaces, original-AST validation before ANY specialization,
initializer exclusions, collision boundaries, no speculative unbilled copying,
in-place outer Struct, cumulative syntax/work/string accounting and distinct
ownership. Coordinator GO now permits ONLY this contract; immutable final reviews
and all resource/source/consumer/release gates remain mandatory. Pause for authority/scope ambiguity,
wrong lexical semantics or unbounded/uncharged work—not a filename/opcode exemption.
#175 remains OPEN. Not full Pratt policy, fourth compiler closure, general CUE/loops/
interpolation/schema/defaults, self-hosting, performance or mathematical minimality.

## Completed local evidence

96 exporter tests/four profiles; full Rust/default all-target strict Clippy both
profiles; retained135 research groups/four profiles. Eighteen checks/fifteen packages
actually built AND rebuilt; BOTH genuine façade matrices and their separate feature
derivatives rebuilt. Pinned preflight13/13 (7 skipped, cached Nix discovery0) is not
substituted for those executions. All249 JSON,12 builders/old DATA/generated Rust
unchanged. Twelve fresh recursive/full inventories match retained inventories;
original core feature graph Go-free, scanner source-feature graph Go-backed only.
Exporter9 source `/nix/store/6pc3icx3g1lj5jgpfkcsky9q0dyibhl0-source`, raw2 source
`/nix/store/mlg3w4xyqfi043fxykfiy64fg9xddigw-source`; canonical module2779 bytes.
Cross-review corrections keep exact Unsupported for extra clauses/single iterator/
reserved source tokens, Missing Unknown, and nested-empty total40 (preparation9+
evaluation31 including three retained-bound checks). No successful admission widened.
