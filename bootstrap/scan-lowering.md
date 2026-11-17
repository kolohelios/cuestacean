# Shared CUE scanner compiler backend

`scan-lowering{,-fragments}.cue` owns scanner module/statement layout, predicate
composition and source-branch simplification through the [shared engine](lowering.md).
Existing leaf control syntax and scanner goldens remain unchanged except the
production string scanner, intentionally compacted with [composable predicates](scanner-predicates.md).

## Explicit native boundary

Native validation still checks the entire original program, including unreachable
suffixes: version, duplicate/unknown fields, 64 statements, 16 levels, scoped breaks,
predicate sizes/lookahead and whole-program predicate trees (128 nodes/16 levels). The checked flow pass supplies loop depth, guaranteed
break facts and nearest-loop targets, pruning unreachable suffixes. This analysis
is still native, not claimed self-hosted. Cursor/fuel checks and the reference
statement interpreter remain native too.

CUE chooses real loops versus labeled once-blocks, label spelling, predicate
inversion, step placement, advancement/rejection calls and module layout. It renders
both conditional bodies once, caches them lexically, then merges equal branches
or selects inverted/ordinary condition layout. A merged branch still charges its
runtime condition step. Pure predicates make this valid; do not add side effects
to predicate reads without revisiting this contract. For `and`/`or`, native
metadata propagates the desired inversion bit to both children; CUE chooses
normal/De Morgan operators and operand grouping. There is no native boolean
source-emission map, and each bounded compound read still uses one test step.
[Repeat control](scanner-repeat.md) adds count/once/possible-break facts. CUE
chooses zero omission, for loops or once-blocks, adding labels only when needed;
repeat entry/iteration charges remain explicit without lint suppression.

`bytePattern` and `charPattern` are bounded native rendering primitives, not CUE
algorithms: they require 1..128 values, sort/deduplicate, and stream output through
the byte budget. Bytes must be ASCII u64s; runs of at least three compact into
inclusive decimal ranges. Characters must be single-scalar strings and use Rust
Debug character escaping. Target-source composition around these patterns is CUE.

## Scoped rendering expressions

- `bind {name, value, body}` evaluates one rendered value, then evaluates its body
  with a lexical binding. The initializer sees outer scope; nested shadowing works.
- `get {name}` copies the cached rendering through the shared byte budget.
- `ifEqual {left, right, then, else}` renders left then right, compares their text,
  and evaluates only the chosen branch. `ifTrue {field, then, else}` projects bool.

All templates, including unused branches, validate eagerly. Unknown bindings reject
statically. Binding names are 1..64 lowercase ASCII identifier bytes (first letter
or underscore). Bindings are local to a template, not dynamically inherited across
child opcode dispatch. Compiled closures and reference aliases share the same
checked primitives; there is no global Context binding state or budget reset.
Copies, comparisons' operands and enclosing results retain existing work/frame/byte
accounting. Pattern validation precedes rendering. Existing 512-node/32-level policy,
4096-step/128-frame/1-MiB output guards remain unchanged; the default policy uses
438 JSON values. Generation now has shared rendering limits as well as scanner
validation bounds, unlike the construction interpreter's execution-only contract.

## Pipeline and proof

`scanner::generate` selects `compiled_scan_lowering`; no-default-feature seeds
interpret fresh `scan-lowering.json`. `generate_with_policy` and optional
`bootstrap-scan-codegen <program.json> [lowering.json]` select a custom reference
policy after complete scanner validation. Exports remain independent of six seeds;
all generated Rust is deleted in staged seed builds, regenerated and drift checked.
The semantics-only profile physically removes this backend as well.

Tests retain independent scanner/parser expectations and byte-exact formatted old
goldens. A real compiled alternate backend emits `advance_char` for the unchanged
single-`advance` program: it consumes λ while reference ASCII advancement rejects.
This deliberate compiler-policy adversary does not relax the default ASCII cursor
contract. Compiled binding fixtures check captures, escaped selectors/strings,
lazy projection and fixed successful outputs. Native flow analysis, pattern
formatting, policy compilation and Go CUE export remain bootstrap boundaries;
this is not full compiler self-hosting or a measured performance improvement.
