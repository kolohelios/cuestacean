# Bounded native string concatenation prerequisite (#276)

## Fresh scope and authority

Baseline main `3589d6a5b91f3e7acfd07604c019548135815d80` (#275).
Read the [exporter](native-spec-export.md), [lexical bindings](native-let-bindings.md),
[scalar projection](native-scalar-projections.md),
[literal-container projection](native-literal-container-projections.md),
[multiline values](native-instruction-module-text.md) and
[module](../bootstrap/module.md) contracts before changes.

The user approved generic concatenation ONLY: approximately1600, then1750 after
forecasts1315–1650/1450–1750, then a fresh one-off **approximately1900 changed
lines** after ENTIRE-contract forecasts1492–1852/1535–1865. Count ALL additions
AND deletions, natural formatting, docs, evidence and UNSPENT repair reserve.
No old exception transfers. Budget permission is NOT implementation GO: TWO fresh
ENTIRE-contract focused reviews and explicit coordinator GO precede code changes.
Pause FIRST credible overrun/identity defect/ambiguity/unresolved validation failure;
no midpoint substitution, proof compression/deletion, weakened assertions or silent
scope split. The prior six-step and seven-step proposals were alternatives, not
verified implementations. This contract selects the SEVEN-step schedule below.

Future need is the original `#InstructionModuleText.header + "\n"`. This slice
changes NO CUE owner/bridge/constructor/Go consumer/schema/composer/compiler/Cargo/
CLI/feature/provider ABI/Nix source cut/package/check. It removes NO production Go
edge. Module publication/full instruction policy/schema/PROGRAM/joined-seed closure
and broader #175 remain held. FIVE complete native policies/FIVE native compiler
binaries/six origins remain. No performance/full-CUE/self-hosting claim.

## Exact syntax, operands and document class

Private `Expr::Concat(Id,Id)` plus a distinct Document.concatenations counter.
Lex ordinary + as binary punctuation; parse LEFT-associative precedence3, above
&2/|1. Existing parentheses add no arena node and retain current parse-depth checks.
Include BOTH children in binary node height and binding dependency graphs. Keep
original Projection-to-definition edges through resolution/cycle admission.
New counter initializes0 in parser and existing defensive evaluator test fixture;
never repurpose the projections count or scan uncharged for dispatch.

A concat operand is an ORIGINAL string Literal (ordinary or admitted multiline),
a direct existing #Definition.field Projection selecting an ORIGINAL string
Literal, or another recursively admitted Concat. Parentheses may surround these,
not authorize new selector bases. No numeric addition/coercion, bool/container/
type/Ref/resolved Binding/field/And/Or/default/comprehension operands. A lexical
binding or ordinary definition may CONTAIN concat; using that binding/Ref as an
operand stays excluded even if it could evaluate to a string.

With projections, preserve the ENTIRE old whole-arena whitelist, extended ONLY
by Concat; And/Or/type/open-tail/etc remain Unsupported. Every projection target
remains an original direct Struct with its ENTIRE pure-literal unique-field tree
validated, including unselected siblings. Concat/Projection/Binding/Ref anywhere
INSIDE a target is excluded; folding must never sanitize a target.
Without projections, existing unmarked constraint composition around concat
results remains allowed: `"a"+"b" & "ab"` succeeds, mismatched literal conflicts,
ordinary existing disjunction readiness stays unchanged. This is static concrete
string computation before the SAME old constraint engine, not backward solving.
ANY mark/default or comprehension ANYWHERE in a concat document rejects BEFORE
rewriting, including unused declarations/empty comprehensions. No new interpolation,
label/inequality/template-expression syntax, arithmetic or default admission.

Missing definitions/fields retain Unknown(actual name/path); lexical collisions,
unused declarations and binding cycles retain their old admission/errors. Pure
recursive references retain old demand guards. Invalid operands use a precise
Unsupported("string concatenation operand"), not Conflict/Incomplete or a broad
error union. Mark/comprehension exclusion uses
Unsupported("string concatenation document profile"); historical projection
categories remain unchanged when reached. Existing selector grammar, LF separators/CR handling, escapes and
ordinary no-addition behavior remain unchanged except recognizing the + token.

## Original-arena preparation and unchanged ledgers

Keep preparation in existing projection.rs. No concatenations: execute the OLD
projection path/cost/order exactly; no projections either: skip preparation exactly.
With concat: after binding resolution, validate ALL original nodes/targets/operands
before ANY rewrite. One charged arena-order scan can record bounded metadata by
original Id: string lengths, selected original Id/kind and concat descriptors.
Children precede Concat parents; forward definition targets are available in the
complete original arena. Metadata contains lengths/Ids only, no cloned Expr,
strings, target trees, intermediate JSON, synthetic definitions or owned names.

Only after complete validation, specialize in place in child-before-parent order:
scalar Projection -> old Binding(original selected Id); container Projection ->
old SelectedContainer(original selected Id); Concat -> string Literal at SAME Id.
No new arena nodes/syntax refund. Preserve all existing selected-container fresh
per-demand closure groups/incoming groups/nested witnesses and outer openness.
Evaluator changes ONLY new defensive unexpanded-Concat rejection and counter0;
no evaluation/admission/readiness/reference/closedness algorithm or charge changes.
lib.rs/comprehension.rs remain unchanged; mixed comprehension admission is forbidden
so its separate preparation initialization cannot overwrite concat consumption.

For original N nodes, C concats, and whole original target field/list edges F_i/I_i:

`A = 1 + N + sum(4 + 2F_i + I_i) + 7C` successful preparation steps.

PRECHARGE each operation: entry1 before metadata allocation; each original node
inspection1; each projection descriptor1/lookup1/root-inspection1/replacement1,
field-name uniqueness1 plus child inspection1, list-child inspection1 as before.
Each concat: descriptor allocation1, EACH original operand kind/metadata/length
inspection1 (two), result allocation1, EACH append1 (two, even empty), replacement1.
Checked length addition and cumulative copied-byte reservation precede allocation.
Charge ENTIRE result byte length to preparation_strings BEFORE allocating it;
append each borrowed operand once, no unbilled clones/speculative joined buffer.
Descriptor/length validation precedes all rewrites; allocation/copy/replacement
charges occur in rendering order. Reusing metadata must not add uncharged scans.

Commit consumed preparation work/bytes to SAME document ledger; evaluator subtracts
preparation_steps and starts with preparation_strings. Count ALL intermediate
chain result copies, even undemanded ones, then existing final materialization/key
copies; no resets/refunds/per-expression allowances. All original work4096,
source/copy/encoded65536, tokens4096, syntax2048, depth32, width64, definitions64,
bindings64/materialized2048 caps remain. Logical bounds are not stack/time/allocator
hardening claims. Do not raise one guard to manufacture another reachable boundary.

Independent source-derived families, no extra Ref/let wrappers:

- Literal scalar chain C>=1: N=2C+1, A=2+9C, E=3, total T=5+9C.
- One flat width-n target, root #D.x+"": T=19+3n.
- List of m independently written #D.x+"": T=8+n+18m+2nm.
- Object of m distinct fields with that expression: T=7+n+21m+2nm.
- List of m literal chains each C concats: T=7+m(9C+5).

Assert T−1/T/recovery, both source orders. Hard4096 witnesses: n23/m63 list4063
succeeds, m64 mathematical4127 fails; or literal C30/m14 list3857 succeeds,
m15 mathematical4132 fails. Never use an allowance above4096 to admit a witness;
a separate4097 rejection control verifies the unchanged hard maximum.
For m independently demanded concat results each L ASCII bytes, copied2mL;
pretty list encoded3+m(L+6). With64 #D.x+"" expressions, L512 copies EXACT65536
and succeeds, L513 copied-string Limit. With L decoded LF bytes, encoded3+64(2L+6):
L508 gives65411, L509 gives65539 encoded Limit while copies stay below65536.
Use small independent byte witnesses first; UTF-8 counts BYTES, not characters.
Different associations can copy different intermediate lengths despite equal values.

## Independent evidence, ownership and release

Kernel ONLY spec-export/src/{model,parser,bindings,projection,eval}.rs; eval ONLY
counter initializer/defensive rejection+its assertion. No public Error variant/API.
Evidence ONLY NEW tests/string_concatenation.rs; ALL33 old integration files stay
byte-identical, all168 old groups retained. The existing defensive UNIT changes
only for the authorized counter initializer/new rejection assertion above.
Old export.rs1+1 remains rejected; ADD exact operand-category proof
in new file, do not delete old assertion. Interpolation v+v stays excluded. No old
positive/negative reclassification is necessary. Coordinator owns this contract,
checkpoint/exporter docs/integration/source/artifact/closure/release/CI proof.

FOUR complete new groups forecast, not a proof cap: independent Unicode/empty/
escape/multiline/chain/grouping/precedence values and ordinary constraint outcomes;
all operand/whole-arena/original-target anti-sanitization/Unknown/real binding-cycle
and CLI2 EMPTY/recovery phases; selected-container closedness alongside concat,
outer openness/repeated uses; exact work/shared-copy/encoded/source/token/syntax/
depth/width guards and preserved old costs. Unused invalid source must reject.
Both file orders/repeats/arbitrary names/unrelated filenames and required pinned
Go object-root comparisons use independently authored expectations, not captured
native/Go/golden output. Scalar/list native API accounting fixtures are not Go
package-embedding evidence. Tools absolute REGULAR/readable/executable BEFORE clear
env, owned HOME/unusable PATH. No setup failure relabeled semantic negative.
A generic header-shaped fixture is verification ONLY, never production publication.

Max TWO stable-main isolated workers/disjoint ownership. Immutable saved handoffs;
repairs NEW children, not amendment/rebase onto mutable coordinator. Integrate ONLY
owned paths; validation overlays stay live through Cargo but outside owned snapshots.
Format only owned files. Focused cross-reviews then TWO final immutable ENTIRE-tree
GOs; retire workers/panes/workspaces BEFORE PR or user-decision pause.
Measure full naturally formatted delta EARLY plus outstanding docs/UNSPENT repairs
BEFORE expensive validation. Approved1900 includes everything, not proof caps.

Expected unchanged30checks/27nondefault packages/57labels/56unique roots: verify
actual, capture ONCE from filtered ./infra/tooling, BUILD AND separately REBUILD
SAME explicit drv^* roots in real sandboxes. Audit exact FILE/DIRECTORY sets/EVERY
BYTE against frozen source (exporter10/checker/new34testfiles/full1354 forecast),
no targets/untracked/symlinks/extra empty dirs. ALL24 recursive/full retained
inventories refreshed:23 native constructors plus ONE Go-backed seed control.
Current Go CLI/source/ACTUAL linked producer/aliases/registered identities/schema/
bundle/executor/legacy+joined/BOTH oracle outputs+drvs/provider env/expected answers
excluded from native construction. FOUR genuine façade matrices plus FOUR separate
actual feature rebuilds/fresh metadata; core/unifier/Pratt generation native,
scanner27 PROGRAM/source-feature Go-backed VERIFICATION ONLY.

All249JSON/12builders/old native DATA/generated Rust/five cuts/six origins unchanged.
Exporter-dependent IDs may legitimately change; BOTH oracle PACKAGE IDs stable.
Full exporter168old+new EACH FOUR/strict Clippy4; executor775 default debug/release/
Clippy2; research135 FOUR; THREE boundary suites4 EACH FOUR/strict Clippy. Independent
raw-log source-case/block/completion audits retain real later-ok progress witnesses.
Pinned Shaka full preflight/whole whitespace/links; protected exact-head rebase
CI-gated auto-merge watched through MERGED, exact merged-tree equality/clean main.
Preserve failed/canceled/setup attempts and corrected NEW receipts independently.
Long jobs have independent nohup/PID/log/status receipts. Linuxx86_64 only.
No implementation GO is asserted by this contract yet.

## Contract review: held before implementation

Both independent ENTIRE-contract reviews examined immutable96bf3092 (182 lines)
and reconciled the selected seven-step ledger, five total-work formulas and
copy/encoding witnesses. Neither found a fundamental semantic-interface blocker.
Both issued budget NO-GO: complete naturally formatted forecasts1492–1852 and
1535–1865 count five production paths, independent evidence, all documentation and
UNSPENT repairs. The earlier1750 ceiling is not treated as authority for1865.
The larger forecasts reflect full evidence/release-document accounting, not a
new feature or reduced proof. No production code, Cargo tests or builds ran.
Implementation remains paused pending explicit budget/scope resolution, renewed
ENTIRE-contract approvals and coordinator GO. No old proof was compressed/deleted.
The held child also clarifies source identity: ALL33 old integration files remain
byte-identical; the existing defensive unit permits ONLY the already-authorized
counter/rejection assertion extension, not literal byte identity of that unit.
The user subsequently approved approximately1900 for THIS SAME prerequisite only.
Renewed ENTIRE-contract reviews and coordinator GO remain required before code.

## Fresh coordinator implementation GO

BOTH independent ENTIRE-contract reviews approve immutablefd88cf31 (202 lines),
including selected seven-step charges, original-arena validation, precise source
identity and full proof/release obligations. Complete upper forecasts1872/1885
include UNSPENT repairs under the user's1900 allowance. Coordinator GO authorizes
ONLY the five kernel paths/new generic test/docs above. Measure naturally formatted
full delta early; stop FIRST credible excess/ambiguity/identity defect/unresolved
validation failure. Cross/final/source/profile/sandbox/CI/exact-merge gates remain.

## Implemented / focused evidence (x86_64-linux)

Immutable kernel57012e9b (199 changed lines) and new evidenceeb51f2c4 (969 lines,
FOUR groups) passed both focused static cross-reviews. First actual compilation
failed101 on a Rust receiver-borrow ordering conflict; no tests ran in that attempt.
Reviewed NEW-child repair261b761b moves the existing parser match into a local
before node allocation, preserving semantics/charges and immutable original bytes.
Corrected integratedba74a299 passes all13 focused cases EACH FOUR profiles (52
executions, including16 new), strict all-target Clippy4, owned formatting and
before/after source hashes. Original101 is retained separately, not relabeled pass.
No old integration file or assertion changed. Full release gates remain pending.

A missing required auditor-prefix argument was caught while orchestration wrappers
were still waiting; neither the auditor nor Shaka had run. Those wrappers were
canceled and preserved. Their EXIT traps recorded the preceding successful sleep
as0 on SIGTERM: those zeros are explicitly NOT passing receipts. Reviewed NEW v2
wrappers supply the exact prefix/current capture, require real completed jobs,
use separate output/status paths and explicit TERM143/INT130 handling. Full Rust
and sandbox jobs were not canceled; the reviewed auditor itself was unchanged.

Full frozen-source Rust validation completes0: exporter172 EACH FOUR, executor775
EACH default debug/release, research135 FOUR, THREE boundary suites4 EACH FOUR and
14 strict Clippy completions. Independently audited source/script-derived totals:
2826 named successes/408 complete blocks/28 Cargo completions. All18 raw progress
notifications require later actual success in the SAME block, never count as
successes themselves. The known active semantics-cfg witness is retained twice.

Actual ONCE30checks/27nondefault packages/57labels/56roots BUILD AND separately
REBUILD using SAME explicit drv^* targets in real sandboxes. Independent execution
audit verifies all56 checking-output records once and original four capture hashes.
Exporter10/checker127 with34 integration files/full1354 AND legacy1354 match exact
FILE/DIRECTORY sets/EVERY byte against frozen source. All33 old integration files,
all CUE owners, flake and four façade cuts remain byte-identical; only FIVE allowed
production paths and ONE new test change inside bootstrap.

All20 old artifact trees remain byte-identical, including249JSON/12builders/oldDATA;
generated Rust and six actual binary origins are retained. Exporter-dependent
package IDs legitimately change25; BOTH oracle PACKAGE IDs remain stable.
All24 refreshed recursive/full retained inventories pass:23 native constructors
plus ONE Go-backed seed control. CURRENT actual CLI-linked Go-source producer,
valid aliases/registered-name-only witness, DB/SRI/version correspondence,
provider/answer exclusions and68 exact source witnesses are independently checked.
Manifest recomputation covers167 entries/168 total files/69 metadata files.
FOUR separate actual feature rebuilds match fresh stored metadata; core/unifier/
Pratt generation stays native, scanner27 source-feature Go-backed VERIFY ONLY.
Independent completed façade log audit verifies936 named executions/80 blocks/
24 strict Clippy completions, exact source-derived case/feature/profile sets and
fresh generated modules outside constructor source cuts. Earlier read-only
metadata-field inspection failed before that auditor ran; its receipt remains
separate, not a test/auditor pass. The reviewed auditor's first execution passed.
Actual pinned d07 Shaka full preflight13/13 (seven skips) and whole-tree whitespace
1437 files pass. Final documentation checks/TWO immutable ENTIRE-tree GOs/protected
CI through confirmed merge/tree equality remain release gates. FIVE/FIVE and
broader #175 stay held. No other platform or performance/closure claim follows.
