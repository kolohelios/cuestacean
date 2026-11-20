# Native instruction module text / bounded multiline literals (#248)

## Selected boundary after #246

Five existing compiler binaries construct independently; instruction/evaluator
policy DATA still needs Go. Its literal Rust module header uses CUE multiline
syntax, separately from Binary/reused-R and two module-control defaults. Select
ONLY the real header/footer DATA and the literal syntax needed to construct it.
No sixth complete policy/compiler, module-array/default admission, general string
interpolation/concatenation, schema/PROGRAM/seed closure or #175 completion.

Read-only probe refactors original quoted header/footer into one concrete helper
without changing either payload. The header's existing +"\n" remains in the Go
consumer, not a new native concat feature. Standard pinned `cue import -p bootstrap
-l '#InstructionModuleText:'` places native JSON under the helper definition in the
GO CONSUMER. Canonical, raw-helper and imported-helper policies match byte-exact,
including BOTH actual left-twice/reverse overrides. This is feasibility ONLY, never
a captured constructor seed/expected answer. No raw text helper feeds canonical Go
construction once the native JSON supplies it; raw source remains an oracle input.

## Exact bounded syntax / resources

Kernel/build owner ONLY spec-export/src/parser.rs, infra/tooling/flake.nix, NEW
bootstrap/instruction-module-text.cue and instruction-module-text-export.cue,
existing lowering-fragments.cue. No model/eval/comprehension/bindings/lib/Cargo/
executor/façade/CLI/schema/composer/guard changes; exporter selected source stays9.

Admit ordinary STRING VALUES in LF-only triple-double-quoted form. Opening `"""`
must be immediately followed by LF; closing unescaped triple quote must be first
non-space/tab text on its own closing line. Closing indentation is an exact byte
prefix of every non-empty body line; remove it, preserve further/trailing literal
spaces/tabs, permit truly empty lines. Reject short/mismatching nonempty indentation
rather than guessing/tab-expanding. Discard opening LF and the LF before closing
as CUE does; empty-body/blank-line cases explicit. UTF-8/scalar quotes (one/two raw
quotes) are literal DATA. Any raw triple quote inside a non-closing body position
rejects. CR/CRLF, raw U+FEFF (pinned CUE rejects an interior byte-order mark), and
raw ASCII controls other than LF/TAB remain outside this class.

NO backslash syntax AT ALL in a multiline body in this slice: escapes, raw \(x),
line continuation and escaped delimiter forms Unsupported, including unused/
empty-comprehension syntax. Existing ordinary strings retain all current escapes/
interpolation handling unchanged; no decoded data is scanned again. Hash/bytes/
single-quote/raw delimiters/general interpolation unchanged Unsupported.

Use a PRIVATE origin-distinguishing Token::Multiline(String) (or equivalent) so
only ordinary expression values/pure literal comprehension TEMPLATE values accept
it, producing the SAME Expr::Literal/Template::Literal string. Do NOT admit field
labels, computed keys, literal-inequality operands or new binding names by accident;
unsupported multiline contexts reject explicitly. Definition/let use and existing
marked-value profile constraints remain unchanged; this is alternate literal syntax
for already-supported string DATA, not new default semantics/admission.

Lexing prevalidates closing/indentation/characters and checked decoded byte length
before ONE bounded decoded allocation; no unbilled speculative clones/line-joining
temporary. Original combined source64KiB bounds decoding/scanning; output cannot
exceed raw source. It is ONE token/ONE scalar syntax/template node and old literal
work/copy behavior EXACTLY, including shared4096 work/2048 cumulative syntax/32depth/
64items/64KiB copied strings+encoded output/source guards. No per-line counter reset,
extra evaluator steps/refunds, raised guards, stack/recursion/lint/cfg suppression,
new public API/error variant/provider environment or Go fallback.

## Canonical raw owner / Go consumption

NEW instruction-module-text.cue owns ONLY concrete #InstructionModuleText with
header (original literal payload unchanged) and footer (original ordinary string
unchanged). One reviewed cue fmt relocation exception may uniformly change the
common body/closing margin from three tabs to two; every payload byte AFTER exact
margin removal, all blank lines and the footer literal stay byte-identical. NEW export selector embeds that definition
for native export. Exactly TWO raw source inputs; no root helper fields/version/
policy cases/aliases/defaults/assembly/source scraping/preexpanded captured Rust.

lowering-fragments.cue declares only the required helper SHAPE
#InstructionModuleText:close({header:string,footer:string}), with no default values.
Explicit close is REQUIRED: merging another declaration into the SAME definition
can extend an otherwise automatically closed definition.
Its module first value becomes #InstructionModuleText.header + "\n", last value
#InstructionModuleText.footer. Preserve all other module statements/defaults/array
order, Binary/comprehensions/reused-R/unary requirements/helpers/cases EXACTLY.
Unknown-name/schema collision probes required. Missing helper/both slots/type-only
shape => non-concrete/no JSON with cue eval success; actual header diagnostics may
say `non-concrete value string in operand to +`, not literally `incomplete value`.
Each header/footer slot independently required, not supplied by old owner. Wrong
types/extra fields fail schema (not ordinary incomplete-value negatives). Original canonical/
left-twice/reverse and all six/evaluator/escaped/alternate controls stay meaningful.

NEW native-instruction-module-text package/check constructs JSON from raw2/exporter9
ONLY with repeated exports/publication64KiB cap, no Go/schema/fullbundle/executor/
compiler/oracles/expected/generated construction inputs. Tools absolute REGULAR/
readable/executable before env-clear, owned HOME/unusable PATH. JSON build-only.

Canonical jsonExports consumes that independently native JSON. In the existing
GO-backed consumer, pinned cue import places it under #InstructionModuleText in a
fresh build-only CUE file; full export loads that helper, existing native unary DATA
and all existing unresolved owners BEFORE contexts. This structural Go import is
consumption, NEVER a native-constructor dependency. No selected-default late retry,
filename/opcode privilege or wrapper that fills required text. Raw oracle helpers
load original raw text definition instead. Update ALL affected full-policy Go
call sites (including old unary module verifier/tests) without broadening their
raw2 constructors. Tests-only source inventory gains actual helper/selector inputs.

## Independent evidence / ownership

Evidence owner ONLY NEW spec-export/tests/multiline_strings.rs and
instruction_module_text.rs; old instruction_unary_tables.rs ONLY raw-oracle helper/
include dependency updates necessitated by moved text. Independent complete literal
values/header-footer tree and complete compiler Rust predictions, NOT native/Go/
scratch/golden captures. Existing109 tests retained; both feature settings/debug/
release (four profiles), strict all-target Clippy. Mandatory Cargo native CLI and
existing pinned cue/jq discovery/setup validation before cleared invocation, no new
compiler/provider ABI. Setup failures are not semantic negatives.

Literal cases: exact empty/open/close/newline treatment, empty lines, zero/space/tab/
mixed exact margins/further/trailing whitespace, Unicode/raw single-double quotes,
source/body malformed/unterminated/bad indent/inline close/CR/control/backslash/
interpolation/unused/empty-map negatives, value-only contexts and all old forms.
Pin scalar-equivalent work/copy/syntax/depth/items/token/source/output boundaries
under SAME ledger; no invented per-line budget. Compare accepted examples to actual
pinned CUE; do not relabel invalid setup/resource cases semantic mismatches.

Actual raw2 module: both file orders/repeats/definition rename, complete independent
header/footer tree, source-only header/footer changes → native JSON → standard Go
import/full policy → COMPLETE independently predicted tiny evaluator Rust. Header
suffix +"\n" and module control defaults stay Go-owned. Schema wrong type/missing2
slots/no output, all raw left-twice/reverse policies, all evaluator/six-driver/
adversarial goldens and actual compiled semantic controls retained. New admission
not inferred from source emission. No assertions weakened or expected text captured.

## Integration / release gates

Coordinator owns docs/source projection/integration/full source/artifact/graph/
validation/CI. Two independent focused lexical/resource/semantic and consumer/
source/evidence reviews plus explicit GO BEFORE implementation. At most2 stable-main
isolated workers/distinct ownership; immutable handoffs/new-child fixes/overlays,
owned-path integration, format only owned files. Retire workers/workspaces BEFORE
PR; independent implementation and final exact-tree reviews.

Strict source projection preserves original header payload after only that uniform
cue fmt margin3→2, exact footer literal + Go newline, and ALL retained body/default/
helper/module order tokens; source edits only helper ownership/references/required
shape. No generated artifacts used as seeds. All249
JSON/12builders/old DATA/generated Rust unchanged. Exporter9 CODE identity and its
native dependent constructor IDs legitimately change; retain exact raw cuts and
all FOUR existing façade source inventories. BOTH Go oracle PACKAGE IDs unchanged.

23 distinct checks/20 NON-DEFAULT packages actually BUILD AND REBUILD. Fresh17
recursive derivation AND full --requisites --include-outputs retained inventories;
native constructors exclude current Go CLI/source/source drv/fullbundle/exec/
legacyjoined/BOTHoracle outputs-drvs/provider envs; native feature core/unifier/
Pratt stay native/scanner source-feature positively Go-backed VERIFICATION ONLY.
New text native DATA is partial instruction input, not sixth complete policy.

Full executor/default all-target strict Clippy debug/release, exporter/four profiles,
research135/four/both old boundary4groups EACHfour; ALL FOUR genuine façade matrices/
fresh modules+separate explicit drv^* feature rebuilds. Pinned preflight/whitespace/
relative links; protected exact-head rebase auto-merge/required CI through confirmed
MERGED. Linuxx86_64 only; no performance/minimality/full CUE/self-hosting/default claim.
Target coherent <=1500 additions+deletions incl CUE moves/tests/docs. Pause for scope/
semantic/consumer ambiguity or unresolved validation; never compress/delete proof.

Implementation GO: BOTH independent focused lexical/resource/consumer/schema and
independent literal/source/evidence reviews approve the exact interfaces above.
Coordinator GO permits ONLY those owned paths, raw U+FEFF exclusion and explicit
Go-only close shape. Canonical/raw/imported helper + both actual overrides match
originals in both orders; missing slots are non-concrete/no JSON with eval success,
wrong types/extras schema failures. Optional exhaustive Go permutations timed out:
not passed, not a semantic failure and not required. These observations are only
interface feasibility, never native constructor seeds/expected captures. Immutable
cross/final reviews and ALL implementation/resource/full release gates remain.
