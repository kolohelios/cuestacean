# Ordinary fields in explicit export inputs

## Implementation contract for #288

The existing expression frontend stays unchanged. `cuestacean export` adds a
separately selected entry profile for one explicit file or stdin. It admits one
or more unique visible ASCII labels matching `[A-Za-z][A-Za-z0-9_]*`, followed
by `:` and an existing expression. Exclude `package` and `import`. Literal
spellings `true`, `false`, and `null` are valid **labels** in this profile, not
valid arguments to `-e`. Do not admit declarations inside an expression.

A label and its colon may have spaces or tabs between them, but no intervening
line break or comment. Require an LF between declarations; CRLF is supported and
lone CR is not a separator. Keep existing line-comment termination (LF only)
and rejected comment characters. Permit blank lines, leading/trailing trivia,
and a trailing line comment. No commas or semicolons between declarations.
Empty/comment-only inputs retain the existing failure. The RHS grammar does not
acquire multiline operators or any new nested syntax. Ordinary nested bare
structs, references, imports/packages, embedded values, definitions, hidden or
quoted/computed labels, optional/required fields and duplicate unification are
outside the profile. A duplicate must fail, not overwrite or unify.

Pinned Go CUE v0.16.1 accepts newline-separated fields, operator-named fields,
LF between a label and colon, and literal-spelled labels. It rejects using lone
CR as the declaration separator. Cuestacean deliberately keeps the stricter
same-line colon rule. Use both explicit input forms and separate unsupported
upstream-success cases from differential passes.

## Ownership and boundary

- `parser::parse_file(source, grammar)` is the public parser entry used only by
  the user CLI. CUE scanner/control selects declaration mode by positive
  label/colon lookahead. The probe is noncommitting: nondeclaration input goes to
  the unchanged expression entry with its original offsets and diagnostics.
  After positive recognition, no retry as a standalone expression is permitted.
- CUE owns field iteration, separator syntax, and early return at a file RHS
  boundary. A separately generated Pratt variant, with the common fragments
  defaulting to old behavior, handles the boundary. No source splitting or
  synthetic `close(...)` input. Use the existing field/record builders and one
  program, not per-field evaluation or a native JSON output assembler.
- An LF boundary before more input preserves the original trivia offset until
  the file driver consumes it, so a recursive RHS returning cannot accidentally
  expose `div`, `mod`, `quo` or `rem` as an operator in its parent frame. Propagate
  boundary scope through precedence recursion (binary RHS, negation and not),
  and suspend it inside calls/bounds, groups, indices/slices, and list,
  conjunction and closed-record children. Keep old word-operator trivia,
  after-operand rejection, and nested parsers unchanged.
- One parser shares the 64-node count (record costs one), 32-level depth limit,
  frame fuel and scanner limits. Check staged progress and completion; retain
  existing record/field shape and evaluation limits. Full evaluation still
  precedes field selection: an unrelated incomplete or conflicting field fails.

## Evidence and budget

Cap: 2,100 total changed lines, including additions and deletions, generated
Rust, evidence and docs, with at least 150 lines reserved for repairs during
initial integration. Preserve old generated driver bytes, compiler construction
cuts and regeneration checks. Require compiled and independent reference
results, exact old trees/errors, offset/CRLF/comment fixtures, nested suspension,
operator-named fields after recursive RHS, exclusions and no retry, duplicate
and limit failures, deterministic regeneration, CLI file/stdin and differential
proof, full sandbox preflight, and protected CI. JSON remains build-only.
