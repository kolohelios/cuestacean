# CUE-owned scalar equality

`equal.cue` and `not-equal.cue` reuse finite binary control for `==` and `!=`.
The shared CUE equality program checks that both operands are concrete null, i64,
boolean or text values, then chooses a boolean using the existing structural
`same` primitive. Concrete mixed types compare unequal, not type-conflict, as
pinned Go CUE confirms. Text compares by content without Unicode normalization
or coercion. Abstract types and bounds fail `comparison requires concrete operands`
even if structurally identical, including top `_`. This is scalar equality, not structural/reference
equality, general CUE comparability or propagation. [Ordering](ordering.md) has
its own matching-domain CUE policy, unlike equality's mixed-type behavior.

Both children evaluate left-first before operand checks. Source `==` and `!=`
use existing binary AST builders and Pratt control, with precedence 8 (above `&&` at 7, `||` at 6 and `&`
at 5, below `+`/`-` at 10) and left associativity. CUE infix control recognizes
only complete two-byte tokens; bare `=` or `!` do not advance. No boolean unary
not, assignment or regex operators are added here; the separate ordering slice
now shares this comparison precedence. Existing source limits
and conservative LF policy remain unchanged.

## Compiler and native boundary

The binary-control DSL adds boolean result construction and comparison failure.
The entry-name enum adds `equal` and `notEqual`; CUE maps the latter to Rust
`not_equal`. Entry metadata chooses only the compiled symbol, never runtime
semantics or arbitrary identifiers. CUE owns concreteness classification and
result selection; Rust still supplies owned values, tag/content inspection,
checked frames, JSON/output and the reference VM. Production runtime facades call
generated policies under `semantics`, with no native equality dispatch fallback.

Rule instructions `equal`/`notEqual` evaluate their children once and delegate
to these policies. Native typed decoding and CUE instruction/compiler policies
are extended together. Six fresh seeds still generate semantic/frontend Rust
without committed artifacts; JSON remains build-only. Binary-control 512-node /
64-level and all compiler/parser/scanner budgets are unchanged.

Independent tests pin equality truth tables, mixed domains, ownership/content,
Unicode distinction, abstract rejection, eager validation, left-first errors,
ASTs, token boundaries, precedence, CLI output and failure atomicity. An actual
compiled alternate policy returns false for unchanged equal operands where
production returns true. Source and JSON probes compare with pinned Go CUE;
boolean output stays byte-exact, without numeric/JSON normalization. This is
not full CUE support, bootstrap closure or measured performance.
