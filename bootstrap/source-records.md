# Bounded closed-record source expressions

## Design and scope (review before implementation)

Admit `close({"name": expression, ...})`, including empty bodies, explicit/trailing
commas, trivia, nested closed records/lists and existing postfix indexing. Labels
are quoted strings using existing CUE scanner/checked string decoding; no bare or
computed labels, bare/open structs, declarations, references or field solving.
Each record node lowers directly to recordExpression {fields:[{name,value:AST}]};
close is not a second AST/evaluation node. Preserve duplicate entries until checked
runtime construction. Existing width/name/shape/admission errors remain; parsing
the entire source still precedes evaluation, so later syntax errors may precede
earlier semantic errors. Ownership/mixed certificates are unchanged.

Optional grammar record:{token,kind,fieldsField} selects the constructor. Default
token close, kind recordExpression, fieldsField fields. Validate identifiers and
field names eagerly, including absent-use inputs and token collisions with atoms/
calls. Absence disables source construction without fallback; custom token/kind/
field metadata remains trusted grammar data, not a CUE-compatibility claim.

## Executable CUE / checked bridge

After scanIdentifier, primary CUE checks hasRecord and invokes record, then returns.
The record operation only dispatches a separate record-primary CUE driver using
existing parser-target control/compiler. Do not embed a native field loop or grow
every primary artifact with the full record algorithm. Record CUE sequences:
advanceRecord, openRecordCall, recordWhitespace, openRecordBody, recordWhitespace,
probeRecordEnd, then while hasRecordField:
requireRecordLabel, parseRecordLabel, recordLabelTrivia, recordColon,
parseRecordValue(minimum0), appendRecordField, advanceRecordTail, and optional
recordWhitespace/probeRecordEnd for a comma/trailing close; finally closeRecordCall
and buildRecord. CUE owns loop, minimum, scanner acceptance and diagnostics.

Reuse call-open (including newline-after-name policy), whitespace, after-operand
for label trivia (rejects newline), string-literal and group-close scanners. Five
small new CUE scanners own required `{`, optional `}`, quote lookahead, required
colon and comma/brace tail; no native character/token acceptance. Group-close
scanner owns trivia/closing parenthesis after `}`. Values retain conservative
existing newline-after-operand policy; no general implicit comma insertion.

Native opaque source-record stages check spans/progress, candidate metadata,
pending decoded key/child, bounded buffering and completion. Field wrappers use
a new CUE AST builder with name/value parameters; final record uses the existing
unary builder over a buffered array. Native JSON-map assembly is forbidden.
Incomplete driver returns/cross-phase operations never expose partial records.
Driver dispatch consumes the same frame fuel, not a new context or budget.

Add a selected record engine alongside existing expression/primary engines;
default APIs preserve signatures and choose production/interpreted record control.
Explicit three-driver hooks allow alternate record control and propagate all
engines through every nested field/group/list/RHS. A high-minimum record alternate
must change field parsing even through nested contexts; no silent default fallback.

## Limits and evidence

Existing source 64 AST nodes/32 recursive expression levels/4096 steps per frame
remain. Labels/field-entry wrappers are data, not extra AST nodes; record costs 1,
so 63 single-node fields fit (runtime capacity 64 remains separate). Existing shared
AST 4096 visits/64 active nodes and mixed value 32-level/4096-node bounds remain.
No larger stack/guard, new compiler/seed, value representation or native emitter.

Retain actual old primary byte-identical with record support disabled. Require
independent complete trees/values/errors, empty/nested/mixed cases, UTF-8 escaped
labels, delimiters/trivia/explicit comma rules, metadata/collision/disabled cases,
selected-engine propagation, stage/progress/fuel/unfinished return, node/depth
boundaries and atomic CLIs. Pin Go CUE close at each record node, exact i64-safe
outputs and semantic conflicts. Full Rust/strict Clippy, preflight/five real sandbox
checks, six artifact-free seeds/profiles/repeated drift remain required. JSON stays
build-only; committed Rust never seeds generation. No full CUE/bootstrap closure
or measured performance claim.
