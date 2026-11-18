# CUE-defined AST construction

The parser's JSON-object assembly now comes from eight CUE builder programs:

- `ast-program.cue`: the versioned program wrapper.
- `ast-atom.cue`: a payload-free node with a grammar-supplied kind.
- `ast-integer.cue`: integer and bound leaves with a dynamic payload field.
- `ast-text.cue`: owned string leaves with grammar-selected kind/payload names.
- `ast-binary.cue`: nodes with grammar-supplied kind and child field names.
- `ast-ternary.cue`: nodes with three ordered children and grammar-supplied names.
- `ast-unary.cue`: unary nodes with a dynamic kind and operand field.
- `ast-record-field.cue`: a decoded quoted name and expression child; the wrapper
  is data, not an additional AST evaluation node.

`bootstrap-ast-codegen <builder.json> [lowering.json]` compiles these pure
construction expressions through the shared [CUE backend](ast-lowering.md) into
typed Rust functions. The parser calls the generated builders,
not the interpreter; it no longer assembles JSON maps itself. No Go process or
builder JSON file is needed during source execution.

These expressions define construction, not control flow; the separate
[Pratt DSL](pratt.md) now supplies the expression loop and binding-power policy.
The [primary control program](primary.md) supplies primary dispatch and grouping
policy. Grammar validation/lookup, checked i64 conversion, frame safety, and
native builder validation/use-count analysis and the reference construction
interpreter remain handwritten. CUE owns Rust layout and clone/move selection.
The [native builder-family exporter](../docs/native-builder-family.md) now supplies
all 12 normal/alternate builder-specification artifacts to the production JSON
bundle. Go CUE remains an independent oracle and exports compiler/lowering
policies; neither the full parser nor its compiler is self-hosted.

## Expression contract

`#BuildProgram` in `ast-schema.cue` contains `formatVersion: 1`, ordered `parameters`,
and a `result` expression. Parameters have unique nonempty names and one type:
`string`, `integer` (i64), or `json` (any JSON value, not a validated AST).

| Expression | Meaning |
| --- | --- |
| `argument` | Read the parameter named by `name`. |
| `string` | Construct a literal string from `value`. |
| `integer` | Construct a literal i64 from `value`. |
| `object` | Evaluate ordered `fields`, each with `key` and `value` expressions. |

Keys must be statically string-valued. Object values may nest expressions;
repeated keys use last-field-wins semantics, matching the former native maps.
There are no loops, calls, dynamic argument lookup in generated code, or
literal array/bool/null expressions. Arbitrary JSON can pass through a `json`
parameter. This is a small experimental construction language, not general
CUE evaluation or complete JSON-language support.

The compiler validates argument references, key types, at most 16 parameters,
64 expression nodes and 16 nesting levels before emitting source. Generated
functions accept owned `String`, `i64`, and `serde_json::Value` arguments in
parameter order and return a JSON value. A reference interpreter checks
positional JSON arguments dynamically.

Metadata names never become Rust identifiers: emitted parameters use positional
names such as `_arg0`. Literal strings/keys are escaped as Rust strings. Owned
arguments are cloned only when reused, then moved on their final use. Normal
parser builders use every argument once, avoiding subtree copies. Compilation also
obeys the shared lowering string/work/frame/output bounds; see the backend contract.

## Checks and bootstrap

Tests compare compiled builders, the interpreter and handwritten JSON
expectations, including customized field names, i64 extrema, escaped metadata
names/keys, and repeated arguments. A version-2 wrapper program demonstrates
that construction policy is data rather than a native format-version constant.
Existing parser tests also compare full trees against CUE-schema exports and
retain upstream source comparisons and exact error checks.

```sh
nix develop ./infra/tooling --command cargo test --manifest-path bootstrap/executor/Cargo.toml --test ast
```

Nix exports builder JSON natively as build-only data and adds the handwritten AST
compiler to the bootstrap seeds. The independent producer runs each specification
twice with no Go CUE on its input/build path; the separate family verifier compares
fresh upstream exports and regenerated normal/alternate/forced Rust goldens. With the default `source` feature disabled,
seeds require no generated builders or scanners. The final stage regenerates
all Rust, enables the source frontend, and runs the complete suite. Committed
Rust modules remain drift-check goldens. See [regeneration](generated/README.md)
for local commands; no generated JSON is committed.
