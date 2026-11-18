# CUE-controlled closed-record selectors

## Bounded capability (review before implementation)

Source `operand.name` / `operand."quoted label"` selects existing closed-record
fields and chains left-associatively with bracket indexing above unary/binary
operators. Unquoted labels use ASCII letters followed by letters/digits/underscore;
leading underscore, definitions and Unicode identifiers remain excluded. Quoted
labels reuse the source-string scanner/decoder and can include Unicode, punctuation
or leading underscores. Pinned CUE probes distinguish regular quoted `_x` from
hidden unquoted `_x`; do not conflate them. CUE accepts keyword-spelled selectors
such as .true/.for/.int and normal uppercase identifiers.

Lower directly to JSON AST `select {operand:Expression,label:string}`. One selector
costs one AST node; label is literal data, not another child AST. Evaluation rule
uses existing binary recordIndex instruction with evaluate(operand) and text(label),
so no new instruction/kernel/value type or dummy operand. The selected evaluator
and shared context enter operand first; malformed label decoding follows operand
errors, then existing strict CUE record/presence policy. Runtime label lookup is
byte-exact; no normalization/coercion. Lists/text/scalars fail typed record lookup.

Optional grammar selector:{kind,operandField,labelField} selects the AST fields;
validate kind/nonempty distinct fields (neither kind) eagerly. Absence disables dot
syntax without native fallback. Existing index metadata and typed APIs stay intact.
Before dot, retain conservative operand-newline rejection; after dot, existing CUE
whitespace permits trivia before the label. No implicit comma insertion or source
open/abstract structs/references/solver is introduced.

## Executable control and checked stages

CUE Pratt control probes index first; only a non-index candidate probes selector.
A native hasPostfix fact reports either candidate, not token/precedence policy.
CUE loops and chooses index versus selector using hasIndex. Selector sequence:
advanceSelector, selectorWhitespace, scanSelectorString, if hasSelectorString then parseSelectorString
else parseSelectorIdentifier, buildSelector; then afterOperand and re-probe. New
CUE dot and ASCII selector-identifier scanners own acceptance/diagnostics; existing
string/trivia scanners are reused. No native character dispatch or postfix loop.

Private selector stages defend candidate/commit/label/completion and cross-index/
primary/binary phase mutations. Finished selector clears both pending postfix
states and uses the existing CUE binary builder with a JSON string label (literal
data), never native map assembly. Early returns cannot expose unfinished postfixes.
All three selected engines and existing frame fuel propagate unchanged.
The native reference VM groups selector primitive dispatch in a helper to avoid
adding per-arm Result temporaries to every recursive VM frame. This preserves the
existing debug-build 32-level record boundary without stack overrides or weaker
tests; it is not a comprehensive native-stack or measured performance guarantee.

Keep actual old bracket-only Pratt control byte-identical as pratt-no-selectors;
pre-index Pratt stays byte-identical too. Existing index-minimum100 and right-
associative alternates must propagate selectors without fallback. Preserve all
512-value/32-level/64-case compiler guards; factor existing templates only if
needed, never raise bounds or add a compiler/seed/native emitter. The expanded
unit-method map would exceed 64; validated selectorAction metadata splits it into
58/7 arms. Existing parse-minimum aliases retain byte-identical source and keep
that revision at 497 original values under 512. [Source slices](source-slices.md)
subsequently replace ordinary method selection with checked CUE snake rendering,
reaching 330 values while keeping actual prior driver sources byte-identical.
Internal tags/fields still reject on wire.

## Independent evidence

Pin complete trees, mixed selector/index precedence and closed-record values,
quoted/escaped/keyword/case labels, hidden/Unicode-identifier exclusions, missing/
malformed labels, metadata/absence, child error priority and selected-evaluator
propagation. Test node/parse/frame budgets, stage/fuel/progress/unfinished return,
shared AST 4096 visits/64 active nodes and parent-release ownership through existing
lookup. Records retain existing value bounds. Pin exact i64-safe CUE source/AST
comparisons, atomic CLI failures, old compiled controls and live alternate policy.
Full Rust/strict Clippy, preflight/five real sandbox checks, six artifact-free seeds,
feature profiles and repeated generation/drift remain required. JSON stays build-
only; committed Rust never seeds generation. No full CUE/bootstrap closure or
measured performance claim.
