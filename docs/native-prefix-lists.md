# Native typed prefix-list constraints

## Selection and review gate

After [#194 scanner extraction](scanner-schema-closure.md), extend the isolated
native specification exporter with boolean literals/types and required-prefix
open lists. These are two composable prerequisites of the unchanged scanner
schema, not production source expansion or native scanner construction. Bounds,
`uint`, regexes, optional fields and defaults remain unsupported; #175 stays held.
The full unchanged scanner-schema/group-open closure MUST still reject natively
as Unsupported with no JSON. Do not strip unsupported unused definitions.

Milestone [#196](https://github.com/kolohelios/cuestacean/issues/196).
Two focused contract reviews approved semantics/resource ownership and independent
evidence before explicit implementation GO. Read-only implementation cross-reviews
precede final validation/PR. Keep this slice below
approximately 1500 changed lines, including tests/docs/generated code.

## Language and semantics

- Admit `true`, `false`, and predeclared `bool` in the isolated native exporter.
  Boolean literals are distinct from integers, strings, objects and lists; never
  coerce them to 0/1 or strings. `bool` alone is Incomplete; a matching literal
  supplies a value; incompatible kinds/opposite literals are Conflict.
- Existing conjunction/disjunction rules apply: deduplicate identical complete
  results; opposite booleans or a compatible incomplete branch stay Incomplete.
  No new defaults, logical operators, negation, null or boolean inequality.
  Pinned CUE rejects `!=true`; keep it outside native admission.
- Resolve predeclared `bool` through the existing immutable lexical admission,
  after file-local bindings/field visibility and exact initializer exclusions.
  Preserve builtin shadowing/capture, field-reference rejection, alias cycles,
  independent instances and reserved `true`/`false` binding-name rejection.
- Admit `[A, B, ...T]` with one or more explicit prefix expressions and one final
  typed tail. Retain fixed lists and `[...T]`. Existing separators/precedence and
  depth checks apply; a tail must have a type/expression and be final. Trailing
  comma after the tail remains outside this bounded grammar. Pinned CUE also
  accepts untyped tails (`[1,...]`, `[...]`); those remain explicit native Syntax
  exclusions, not claimed upstream-invalid syntax.
- The minimum list length is its prefix length. Without a fixed-list conjunct,
  demand the longest known prefix, applying EVERY other prefix/tail constraint
  at each known position. Do not instantiate an infinite tail. `[1,...int]`
  exports `[1]`; `[int,...int]` is Incomplete. Empty required prefixes are not
  silently dropped. A fixed-list conjunct selects the exact length, which must
  be at least every open-list prefix and agree with other fixed-list lengths.
- Meet each position with each applicable prefix-or-tail constraint. Open-list
  conjunctions preserve their residual tails until the whole demand finishes;
  `[1,...int] & [1,"x"]` conflicts. Tail-only incompatible types can still
  export an empty list, as pinned CUE does. Required-prefix incompatible types
  conflict. Every definition/binding use instantiates independently; closedness
  remains inherited, with no global mutation or answer cache.
- Preserve Error categories, canonical JSON and error/no-JSON behavior. No field,
  filename or opcode gets privileged semantics. Do not accept unknown source
  syntax by dropping it, prune Unsupported/Limit as contradictions, or add a
  process/Go-CUE fallback. Production parser/evaluator source syntax is unchanged.

## Interface, ownership and resource boundary

Keep `Document`, `Expr::List(Vec<Id>, Option<Id>)` and public library/CLI APIs.
Add only `Type::Bool` to the private shared model; use existing JSON literals.
Prefix/tail nodes use existing arena IDs, heights, closure groups and cycle walk.
No new production modules or source-allowlist entries.

Kernel owner: ONLY `bootstrap/spec-export/src/{model,parser,bindings,eval}.rs`.
Evidence owner: ONLY new `bootstrap/spec-export/tests/prefix_lists.rs`, targeted
updates in `tests/{bindings,export}.rs` for newly admitted syntax, and the stale
unsupported-bool comment in `tests/scanner_boundary.rs`. Coordinator owns docs,
integration, comparisons, graph proofs and final validation. Agree extra files
before editing. Stable merged-main workspaces, at most two workers; immutable
handoffs, no worker rebase/amend onto mutable coordinator revisions.

All current hard caps stay unchanged: 64 KiB combined source, 4096 tokens/work,
2048 syntax/materialized nodes, 64 definitions/bindings/explicit prefix items,
32 syntax/output levels, 64 KiB copied strings/output. A tail is a constraint,
not another explicit prefix item; it still consumes syntax/token/depth/work.
Charge all prefix/tail traversal using the shared existing allowance; no reset
per branch, item, binding, retry or call. Keep lower-allowance/recovery tests.
No suppressions, raised guards or stack overrides.

## Independent evidence and construction

Pinned CUE source is 6d609d768f1686f9a3a2a20197cacdbb70e5c79d (v0.16.1);
Nixpkgs ac588ef1c914865dad517b337ea0eb9694a4021c. Author raw inputs and complete
expected JSON independently, then compare mandatory fresh pinned Go-CUE CLI
observations. Do not capture oracle output as expected answers. Cover booleans,
heterogeneous types, disjunction readiness, lexical capture/shadowing/exclusion,
recursive closed definitions, list minima/tail overlap/length conflicts, separate
instances, both orders/repeats/renaming, malformed tails, 64/65 prefix items,
syntax/output depths and cumulative work with post-error recovery.

Two-file oracle probes put non-struct expressions in an explicit data field:
a definitions-only CUE file contributes a struct root to the file build, not top.
Both implementations receive identical raw test texts and complete expected JSON.
Do not count a root-shape setup failure as a negative constraint observation or
claim broader non-struct file-build equivalence from expression tests.

Existing native constructor derivations legitimately CHANGE with implementation
source, but their artifacts must keep identical bytes: all 12 builders, policy
programs/four hashes, original atom output, full 249-file JSON bundle, committed
generated Rust and existing independent answers. Production source cut remains
exactly eight files; raw policy cut remains six. Observer identity/provider scope
must stay unchanged. No new native scanner artifact or extra Nix check required.

Run focused/full exporter and executor tests + strict default all-target Clippy
in debug/release, all retained graph/oracle/solver four-profile tests, scoped Go
tests, pinned preflight, all ten actual Linux Nix checks and six package builds
AND rebuilds. Preserve six fresh artifact-free compilers, feature isolation and
deterministic regeneration/drift. Independently compare recursive derivation and
full build graphs (`--requisites --include-outputs`) with verifier inventories;
exclude CURRENT JSON/exporter-consumer/CUE/source/observer from native
construction and observer from seed graph/provider environment. No Go-CUE-free
compiler-seed, minimal source-attestation, self-hosting or performance claim.

## Completed evidence

Two contract reviews approved before GO. Immutable kernel
`9a3598af62452e95debf5d091e340f98db43feae` and independent evidence
`98285639fc168fb676f75e17d01cd36457c5069a` both descend directly from merged #194
(`f3ee4663`). Read-only implementation cross-reviews approved both with no code
blockers; the coordinator integrated exact snapshots and retired both workspaces.

Review corrected oracle setup before accepting results: non-struct expressions
need identical explicit data fields in the two-file comparison. Applying this to
negative cases exposed upstream-accepted untyped tails, now independently tested
as native profile exclusions rather than mislabeled malformed CUE. No assertions
were loosened and no oracle output became an expected answer or producer input.

All 43 exporter groups (42 integration plus one unit), including seven new groups,
pass debug/release and default/no-default with strict all-target Clippy. Both full
Rust suites/default all-target Clippy pass debug/release; all 76 retained graph/
oracle and 14 solver groups pass their four feature/profile combinations with
strict scoped Clippy. Pinned offline observer Go tests pass. Preflight passes
13/13 (seven inapplicable skips); all ten actual Linux checks and six selected
packages build AND rebuild, retaining six fresh compiler binaries, feature
isolation, generation/drift and independent source/value evidence. Full repository
whitespace checks 1289 text files. Other architectures are omitted, not validated.

All native atom/family/policy artifact directories and all 249 full-bundle JSON
files compare byte-identical to #194. All four policy hashes remain unchanged;
no committed generated Rust or existing independent expected answers change.
Exporter/atom/family/policy/full-JSON constructors legitimately change to
`f75llf96…`, `6alqvb3a…`, `9w8w9z3s…`, `mz9zgmmk…`, `pvllxyzw…` respectively.
The observer remains `303m6ck2…`. Exporter source is exactly eight files at
`/nix/store/nbcavd76bfibb4cr81dmxv6njhm6ys9h-source`; the six raw policy files stay
at `/nix/store/06c9bq71vs4rldvajyh1wxis8aklcd08-source`.

Independent recursive derivation/full-build queries match all three native
verifiers' retained inventories: each has 1407 derivations/3392 build paths,
excluding CURRENT full JSON, executor, pinned CUE/source and observer. The seed
verifier independently matches 1436/3466, with observer absent from graph/provider
environment. This is not Go-CUE-free seed or minimal seed-source attestation.
The full unchanged scanner closure still fails natively as Unsupported/no JSON;
#175 and its separate prototype remain held.

Subsequent [#198 readiness work](native-container-readiness.md) found a gap that
these passing tests did not cover: `[1,...int]|[1]` has equal projected JSON but
different open/closed bounds, so pinned CUE remains Incomplete. The same
JSON-only deduplication gap predates this extension for closed/open structs.
#198 retains container bounds during branch comparison; this historical evidence
is not proof of general residual-constraint equivalence.
