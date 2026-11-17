# First Rust generation boundary

The evaluator modules are generated from rule JSON by `bootstrap-codegen`,
whose per-instruction expression lowering runs a [compiled CUE program](../lowering.md).
Checked rendering/cursor kernels and typed metadata projection remain native;
[complete driver source emission](../meta-lowering.md) is CUE-owned;
[module assembly](../module.md) and traversal are now CUE-owned too. Compiler-only seeds use the reference policy interpreter. This lowers our eighteen instruction primitives, not
arbitrary CUE source. The interpreter remains the default executable and a
reference for tests. The normal generated module also runs as a separate CLI:

```sh
nix build ./infra/tooling#exports --out-link result-exports
nix run ./infra/tooling#generated -- result-exports/addition.json
```

Run that command from the repository root; it prints `42`. The rules are baked
in, so this CLI accepts only a program path. The alternate and escaping modules
remain test-only.

`meta-lowering.rs` is the CUE-defined full driver compiler; fresh seeds interpret
its policy, then the compiled emitter regenerates its whole source byte-exact.
`lowering-meta-forced.rs` changes unchanged number-lowering IR's actual compiled
nonzero output to `forced` rather than `more`. `meta-driver-keys.rs` and
`meta-driver-previous.rs` compile unchanged keys-only module IR: the default emits
both quoted keys, while an alternate meta compiler emits only the first.

- `evaluation.rs` comes from the normal CUE-exported rules.
- `evaluation-left-twice.rs` comes from the alternate CUE-exported rules.
- `codegen-escaped-rules.rs` comes from an adversarial JSON fixture that tests
  escaping names containing quotes, backslashes, a newline, and Unicode. This
  fixture exercises the emitter, not the narrower CUE schema. It also changes
  the `true` rule to emit false, checking compiled boolean rule-data control.

`whitespace.rs` is generated from a different, executable [scanner DSL](../scanner.md):
its loop and newline branch come from shared CUE statements in
`scanner-fragments.cue`. `after-operand.rs` composes those statements with
state/EOF checks and rejection policy. The shared trivia loop now includes
CUE-owned `//` comments, UTF-8 scalar advancement and scoped breaks. `group-open.rs` recognizes the opener;
`group-close.rs` composes whitespace scanning with closing-token validation,
consumption, and diagnostics. `bound-prefix.rs` and `infix.rs` recognize symbol
spans; grammar lookup remains native. `decimal.rs` uses the
same DSL for optional-minus/digit scanning and lexical rejection; `identifier.rs`
supplies ASCII identifier start/continuation scanning. Checked i64 conversion,
grammar validation and keyword lookup remain native. The `scanner-horizontal`,
`scanner-stalled`, and `scanner-advance` modules are test-only algorithms that
exercise alternate branches, non-progress, and native cursor guards.
`scan-lowering.rs` now compiles scanner layout from the shared CUE backend;
`scan-unicode-advance.rs` changes the compiler policy, not scanner IR, and consumes
λ where native ASCII advancement rejects. `lowering-bindings.rs` pins lexical
rendered-value captures, escaping, equality and lazy boolean selection.
`scanner-decimal-permissive.rs` demonstrates alternate lexical policy;
`scanner-reject.rs` checks escaped reasons and rejection short-circuiting;
`scanner-state.rs` tests flag reads, stateful loops, and byte-presence checks;
`scanner-lookahead.rs` checks maximum lookahead and EOF handling.

The `ast-program`, `ast-atom`, `ast-integer`, `ast-binary`, and `ast-unary` modules come from
the separate [AST construction DSL](../ast.md). They replace native parser map
assembly. `ast-program-v2`, `ast-escaped`, and `ast-duplicate` are test-only
programs for changed policy, escaping, and repeated owned arguments.
`ast-lowering.rs` is their shared [CUE compiler backend](../ast-lowering.md);
`ast-forced-integer.rs` compiles the same literal-7 input with an altered integer
compiler case and returns 99. `lowering-numbers.rs` tests compiled signed extrema,
escaped joined separators and lazy zero branching.

`string-literal.rs` supplies the [source string lexical subset](../strings.md);
`ast-text.rs` builds its owned string leaf. Native code only checks spans/stages and
decodes the already admitted escape subset. [Unicode escape control](../unicode-escapes.md)
adds four-digit scalars/surrogate pairs without native production Rust changes;
`string-no-unicode` is the compiled earlier escape policy on the same whole-string
template. `string-nested` preserves the prior 63-statement algorithm byte-exact;
`string-unrolled` preserves the subsequent 57-statement composed scanner;
production [bounded digit repetition](../scanner-repeat.md) uses 41.
`scanner-repeat`, `repeat-scopes`, `repeat-zero` and paired `repeat-fuel{,-over}`
fixtures pin compiled counts, break scopes and exact 256/257-step boundaries.
`string-no-wide` preserves the counted four-digit-only scanner byte-exact;
production additionally admits [eight-digit scalars](../wide-unicode.md). The
three earlier baselines explicitly select their previous wide-rejection policy.
`scanner-predicates` executes normal and inverted
compound conditions against fixed observable outcomes. All regenerate in fresh seed builds.

`pratt.rs` is the generated [expression control program](../pratt.md), containing
the operator loop, binding-power condition and associativity branch. The
`pratt-right`, `pratt-stalled`, `pratt-invalid-state`, and `pratt-overflow` drivers
are test-only programs. Frame operations remain native. `primary.rs` uses the
same compiler for [primary dispatch](../primary.md), group commitment, recursive
minimum and closing order. [Flat early-return control](../parser-return.md)
replaces nested dispatch; `primary-nested` retains the prior compiled algorithm
byte-exact. `primary-return`, `primary-return-empty` and `pratt-return-loop`
exercise tail/early return, completion/fuel guards and nested loop unwinding.
The CUE compiler selects footer and return spelling from checked native flow facts.
`primary-no-calls` preserves the pre-call flat primary byte-exact;
`primary-call-high-minimum` changes argument binding to 100; `call-open` owns
required opening/trivia/newline diagnostics. They use the existing scanner/compiler
and unary builder, not a native builtin algorithm.
`primary-no-atoms` and `primary-high-minimum` are
test-only policies, not production fallbacks. `primary-low-negation` weakens
the CUE-supplied unary binding power. `primary-bound-high-minimum` weakens only
threshold-group recursion; it shares `primary-fragments.cue` with production and
uses the existing group operations, without a new compiler opcode. `negation-prefix` is a generated scanner
that distinguishes signed decimal literals from expression negation. `primary-cross-phase` deliberately
attempts expression operations inside primary dispatch to exercise state guards.

Generated evaluator code contains node-kind dispatch and lowered rule expressions.
CUE now emits per-invocation [AST evaluation context](../evaluation-budget.md)
creation, checked entry and private dispatch body; recursive `evaluate` calls
share the current context. Both normal and altered/reversed/escaped-rule modules
retain the same native visit/depth guard, independently of compiler/module fuel. It
shares the handwritten `bootstrap_executor::runtime` primitives for JSON
validation, field access, integer/type values, inclusive bounds, unification,
and checked integer arithmetic, including `difference`/subtraction. Unification delegates to `unify.rs`, generated
from the [unification DSL](../unify.md); `unify-right` and `unify-scoped` are
test-only policies. Addition now delegates to `sum.rs` through the same finite
binary-control DSL ([contract](../sum.md)); `sum-right` is test-only.
`equal.rs` and `not-equal.rs` implement [scalar equality](../equality.md) through
that same compiler; `equal-false` is test-only. Four [ordering policies](../ordering.md)
use it too, with `less-false` as a compiled test-only alternate.
`logical-and.rs` / `logical-or.rs` use [CUE logical selection](../logic.md);
`logic-false` is test-only. Both child instructions still evaluate eagerly.
`list-control` uses the same structured control compiler with the list target:
CUE selects traversal, per-item scalar admission and construction order for
[expression lists](../list-expressions.md). `list-control-reverse` observably
reverses output; stalled/unfinished variants pin fuel and completion guards.
No native production element-evaluation loop or new seed is introduced.
`list-length-control` adds [closed scalar list](../../docs/closed-lists.md) length;
`list-length-zero` is an actual compiled alternate. Native bounded literal codecs
supply owned arrays; CUE owns list isolation/equality/logical/output admission.
No source or general/nested/expression-list support is implied.
`string-length-control` adds [UTF-8 byte length](../string-length.md) through
CUE concrete-text admission and a borrowed checked kernel; `length-zero` is an
actual compiled input-independent alternate. [Unary source calls](../unary-calls.md)
now connect `len(...)` through grammar-selected constructors and CUE grouping.
`logical-not.rs` has a checked one-Value signature; `not-true` is an actual
compiled alternate and `control-default` preserves omitted-function binary arity.
`negate-control` / `difference-control` / `product-control` now own integer
operator dispatch; `difference-as-product` is a test-only alternate compiler mapping.
The four division/remainder `*-control` modules add CUE zero/unit-divisor policy;
`divide-as-quotient` compiles unchanged Euclidean control to truncating math.
`integer-literals` checks actual compiled signed extrema in both literal roles.
`json-output` / `integer-output` own [output admission](../output-control.md);
`output-zero` is an actual compiled alternate producing JSON 0 instead of false.
`lower-bound` / `upper-bound` use [unary bound control](../bound-control.md);
`bound-reversed` flips endpoint placement and `unary-bound-endpoints` executes
left-only choose/literal/empty/projection in compiled code.
Intermediate values may be abstract `bool`, `int` or
bounds; JSON evaluation also supports owned strings and abstract `string`.
Public `execute_json` requires concrete JSON null/integer/boolean/string;
abstract top reports `non-concrete value`. [Lattice policy](../lattice-values.md)
selects top identity and null-domain behavior in CUE;
legacy `execute` remains integer-only. The normal module is exposed as
`bootstrap_executor::generated` under `semantics`; both compiled CLIs reuse it. Tests compare values
and errors against the interpreter and known expectations. Sharing primitives
means this is not an independent test of their correctness or of CUE conformance.

Regenerate from the repository root in `nix develop ./infra/tooling`. Re-enter
the shell after editing CUE rules so its export directory reflects those edits:

```sh
cargo build --manifest-path bootstrap/executor/Cargo.toml --no-default-features \
  --bin bootstrap-codegen --bin bootstrap-scan-codegen \
  --bin bootstrap-ast-codegen --bin bootstrap-pratt-codegen --bin bootstrap-unify-codegen \
  --bin bootstrap-lower-codegen
set -o pipefail
for policy in tests/valid/lowering-identifier lowering pratt-lowering unify-lowering ast-lowering scan-lowering meta-lowering tests/valid/meta-driver-keys tests/valid/lowering-bindings tests/valid/lowering-numbers tests/valid/lowering-select tests/valid/lowering-left-twice tests/valid/lowering-escaped tests/valid/lowering-reverse tests/valid/lowering-module-escaped tests/valid/lowering-collections; do
  name=$(basename "$policy")
  bootstrap/executor/target/debug/bootstrap-lower-codegen "$CUESTACEAN_BOOTSTRAP_JSON/$policy.json" \
    | rustfmt --edition 2024 > "bootstrap/generated/$name.rs"
done
bootstrap/executor/target/debug/bootstrap-lower-codegen "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/lowering-numbers.json" \
  "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/meta-lowering-forced.json" \
  | rustfmt --edition 2024 > bootstrap/generated/lowering-meta-forced.rs
bootstrap/executor/target/debug/bootstrap-lower-codegen "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/meta-driver-keys.json" \
  "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/meta-driver-previous.json" \
  | rustfmt --edition 2024 > bootstrap/generated/meta-driver-previous.rs
for rules in "$CUESTACEAN_BOOTSTRAP_JSON/evaluation.json" \
  "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/evaluation-left-twice.json" \
  bootstrap/executor/tests/fixtures/codegen-escaped-rules.json; do
  name=$(basename "$rules" .json)
  bootstrap/executor/target/debug/bootstrap-codegen "$rules" \
    | rustfmt --edition 2024 > "bootstrap/generated/$name.rs"
done
bootstrap/executor/target/debug/bootstrap-codegen "$CUESTACEAN_BOOTSTRAP_JSON/evaluation.json" \
  "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/lowering-reverse.json" \
  | rustfmt --edition 2024 > bootstrap/generated/evaluation-reverse.rs
for machine in call-open integer-literal tests/valid/string-no-wide tests/valid/string-unrolled tests/valid/scanner-repeat tests/valid/repeat-scopes tests/valid/repeat-zero tests/valid/repeat-fuel tests/valid/repeat-fuel-over tests/valid/string-nested tests/valid/scanner-predicates tests/valid/string-no-unicode whitespace after-operand after-word-operator group-open group-close negation-prefix not-prefix bound-prefix infix decimal identifier string-literal \
  tests/valid/scanner-horizontal \
  tests/valid/scanner-stalled tests/valid/scanner-advance \
  tests/valid/scanner-decimal-permissive tests/valid/scanner-reject tests/valid/scanner-state \
  tests/valid/scanner-lookahead tests/valid/scanner-char-break; do
  name=$(basename "$machine")
  bootstrap/executor/target/debug/bootstrap-scan-codegen "$CUESTACEAN_BOOTSTRAP_JSON/$machine.json" \
    | rustfmt --edition 2024 > "bootstrap/generated/$name.rs"
done
bootstrap/executor/target/debug/bootstrap-scan-codegen "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/scanner-advance.json" \
  "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/scan-lowering-unicode.json" \
  | rustfmt --edition 2024 > bootstrap/generated/scan-unicode-advance.rs
for builder in ast-program ast-atom ast-integer ast-text ast-binary ast-unary tests/valid/ast-program-v2 \
  tests/valid/ast-escaped tests/valid/ast-duplicate; do
  name=$(basename "$builder")
  bootstrap/executor/target/debug/bootstrap-ast-codegen "$CUESTACEAN_BOOTSTRAP_JSON/$builder.json" \
    | rustfmt --edition 2024 > "bootstrap/generated/$name.rs"
done
bootstrap/executor/target/debug/bootstrap-ast-codegen "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/ast-literal.json" \
  "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/ast-lowering-forced.json" \
  | rustfmt --edition 2024 > bootstrap/generated/ast-forced-integer.rs
for control in list-control tests/valid/list-control-reverse tests/valid/list-control-stalled tests/valid/list-control-unfinished \
  primary tests/valid/primary-no-calls tests/valid/primary-call-high-minimum tests/valid/primary-nested tests/valid/primary-return tests/valid/primary-return-empty tests/valid/pratt-return-loop \
  tests/valid/primary-bound-high-minimum tests/valid/primary-no-atoms tests/valid/primary-high-minimum \
  tests/valid/primary-cross-phase tests/valid/primary-low-negation \
  pratt tests/valid/pratt-right tests/valid/pratt-stalled \
  tests/valid/pratt-invalid-state tests/valid/pratt-overflow; do
  name=$(basename "$control")
  bootstrap/executor/target/debug/bootstrap-pratt-codegen "$CUESTACEAN_BOOTSTRAP_JSON/$control.json" \
    | rustfmt --edition 2024 > "bootstrap/generated/$name.rs"
done
bootstrap/executor/target/debug/bootstrap-pratt-codegen "$CUESTACEAN_BOOTSTRAP_JSON/pratt.json" \
  "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/pratt-lowering-no-increment.json" \
  | rustfmt --edition 2024 > bootstrap/generated/pratt-no-increment.rs
for control in list-length-control tests/valid/list-length-zero string-length-control tests/valid/length-zero lower-bound upper-bound tests/valid/bound-reversed tests/valid/unary-bound-endpoints json-output integer-output tests/valid/output-zero divide-control modulo-control quotient-control remainder-control tests/valid/integer-literals negate-control difference-control product-control unify sum equal not-equal less less-equal greater greater-equal logical-and logical-or logical-not tests/valid/not-true tests/valid/control-default tests/valid/unary-conditions tests/valid/logic-false tests/valid/less-false tests/valid/equal-false tests/valid/sum-right tests/valid/unify-right tests/valid/unify-scoped; do
  name=$(basename "$control")
  bootstrap/executor/target/debug/bootstrap-unify-codegen "$CUESTACEAN_BOOTSTRAP_JSON/$control.json" \
    | rustfmt --edition 2024 > "bootstrap/generated/$name.rs"
done
bootstrap/executor/target/debug/bootstrap-unify-codegen "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/unify-equality-probe.json" \
  "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/unify-lowering-permissive.json" \
  | rustfmt --edition 2024 > bootstrap/generated/unify-permissive-bool.rs
bootstrap/executor/target/debug/bootstrap-unify-codegen "$CUESTACEAN_BOOTSTRAP_JSON/difference-control.json" \
  "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/integer-kernel-alternate.json" \
  | rustfmt --edition 2024 > bootstrap/generated/difference-as-product.rs
bootstrap/executor/target/debug/bootstrap-unify-codegen "$CUESTACEAN_BOOTSTRAP_JSON/divide-control.json" \
  "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/division-kernel-alternate.json" \
  | rustfmt --edition 2024 > bootstrap/generated/divide-as-quotient.rs
```

`integer-literal.rs` adds [prefixed numeric scanning](../radix-literals.md) for
source primaries and bounds; `decimal.rs` remains byte-identical after shared
fragment factoring. Checked signed i64 conversion stays native; no new compiler
or AST/rule wire is introduced.

## Fresh Nix builds

The pinned formatter defines the Rust artifact's byte layout. Nix builds in two
stages, clearing leftover JSON and generated Rust from each staged source copy
first. This never deletes files in the working tree. CUE JSON exports are an
independent Nix output built only from CUE sources, not committed artifacts.

1. Export fresh CUE JSON, including evaluator and Pratt compiler policies. Compile the
   native evaluator-generator VM, lowering-policy, scanner, AST, Pratt and unification compiler
   seeds with `--no-default-features` and that JSON export directory. This excludes generated frontend and
   semantic dependencies; `source` normally enables `semantics`. Seed tests need generated modules,
   so they run in the final stage rather than creating a bootstrap cycle.
2. Use the fresh JSON output and the seeds to regenerate lowering drivers,
   evaluator, scanner, AST-builder, Pratt-control and unification Rust. Compile
   all binaries with default `source` and `compiled-lowering` features enabled and
   run the complete formatting, Clippy, and test suite against fresh artifacts.
   Normal rules plus the alternate `lowering-left-twice` policy must reproduce
   the alternate-rule golden exactly; that freshly policy-produced module is
   included in the final compiled tests. Drift checks run the final compiled
   default generator and compare its evaluator output to the interpreter-seeded
   artifacts. The shared CUE backend also reproduces every existing Pratt/primary
   driver. Unification emission now uses another shared backend while preserving
   its previous goldens and scoped tuple semantics; a deliberately permissive
   compiled unifier checks alternate compiler-policy behavior. AST-builder emission
   likewise preserves all old goldens while native checked binding/use facts feed
   CUE-owned layout/clone/move rules; a compiled fixed-integer builder tests altered
   compiler policy without changing builder IR.
   `pratt-no-increment` compiles an alternate compiler policy without
   modifying parser-control IR. Normal, alternate and escaped/zero-argument lowering drivers also
   have repeat/drift checks. Reverse traversal additionally regenerates and
   compiles `evaluation-reverse.rs` while normal evaluator goldens stay unchanged. The semantics-only feature check temporarily removes
   every generated lowering driver to prove its independence.

`lowering-identifier.rs` actually executes identity, snake and upper-first
rendering against fixed lexical/budget oracles. The meta driver emits the fourth
identifier kernel argument in every affected driver and self-regenerates; existing
solver/evaluator/frontend output remains byte-identical. See the
[identifier contract](../identifier-rendering.md).

Committed Rust modules remain reviewable golden files: the Rust generation
check still rejects drift. JSON has repeatability and schema checks, not a
committed byte baseline. CLI tests compare results, errors, and
exit statuses with the interpreter. Neither the generator nor the shared
runtime is fully self-hosted yet; generic seeds remain Rust and CUE exports still
require the upstream Go implementation.
