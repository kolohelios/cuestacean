# Cuestacean

A CUE implementation written in CUE, compiled to Rust. An ouroboros for fun.

Cuestacean is an experimental project to reimplement
[CUE](https://github.com/cue-lang/cue) with a CUE-hosted implementation DSL
and Rust as the execution target. The ambition is full self-hosting, not merely
using CUE to configure a handwritten Rust compiler.

There is no general CUE implementation yet. The [bootstrap fixtures](bootstrap/README.md)
define scalar values and constraints, inclusive bounds, arithmetic and unification trees, plus evaluation
rules, in CUE and export them to build-only JSON using upstream CUE. A small
[bootstrap Rust executor](bootstrap/executor/README.md) interprets those rules.
An initial [rule-to-Rust generator](bootstrap/generated/README.md) lowers the
same rules to compiled evaluator modules. [Instruction-to-Rust lowering](bootstrap/lowering.md)
now compiles CUE-owned recursive expression programs into Rust drivers, with a
reference interpreter for seeds and custom policies. The checked renderer,
typed decoding, canonical metadata projection and safety guards remain native.
The [complete lowering-driver emitter](bootstrap/meta-lowering.md) now lives in
CUE and regenerates its whole source implementation. [Module assembly](bootstrap/module.md)
now also runs CUE-owned boilerplate and rule-traversal statements. The same
[lowering machinery now compiles Pratt/primary control](bootstrap/pratt-lowering.md),
with no native per-control-opcode emitter. [Unification-control compilation](bootstrap/unify-lowering.md)
now uses another policy through that same engine. [AST-builder compilation](bootstrap/ast-lowering.md)
now uses CUE layout/clone/move policy with native checked binding/use-count facts.
[Scanner compilation](bootstrap/scan-lowering.md) also uses shared CUE source/branch
layout, with explicit native flow facts and bounded pattern rendering.
Nix builds native generator/VM seeds, regenerates JSON and Rust from the CUE specifications, then compiles and
tests the final executables. A [source frontend](bootstrap/source/README.md)
now handles a tiny checked-integer subset (decimal and 0b/0o/0x/0X literals) with addition, subtraction,
multiplication, integer division/remainder, unary negation, `true`/`false`, `bool`/`int` constraints,
inclusive bounds, unification, parentheses, and Unicode `//` comment trivia
without Go at runtime.
The first [executable scanner DSL](bootstrap/scanner.md) now supplies the
whitespace, decimal, and ASCII identifier scanning as CUE-authored control flow
lowered to Rust. Parenthesis token handling, decimal lexical validation, the
conservative newline-after-operand rule, bound/infix token recognition, and their
diagnostics also live in CUE. Checked i64 conversion, grammar validation/lookup,
and frame safety remain native. The [Pratt expression loop](bootstrap/pratt.md)
and [primary dispatch](bootstrap/primary.md) now run from CUE control statements,
including [grammar-selected unary calls](bootstrap/unary-calls.md) such as
`len("é🙂")` (UTF-8 byte length 6),
including binding-power, associativity and grouping-minimum decisions. Bootstrap specification evaluation still relies on the upstream Go tool. [AST construction](bootstrap/ast.md)
now uses separate CUE-defined builder expressions compiled to typed Rust functions;
the native parser no longer assembles JSON maps. [Scalar unification](bootstrap/unify.md)
executes CUE-defined identity, interval-intersection, scalar equality and typed conflict policy;
[exclusive integer bounds](bootstrap/exclusive-bounds.md) now use CUE admission,
extrema guards and unit offsets without wrapping;
[owned string scalars](docs/string-values.md) now work in JSON evaluation and the
[bounded single-line source-string profile](bootstrap/strings.md).
[Closed scalar lists](docs/closed-lists.md) now add immutable owned JSON arrays,
CUE content unification/equality and list length. [Source lists](bootstrap/source-lists.md)
now accept computed scalar elements and `len([1,2])`; [indexing](bootstrap/list-indexing.md)
adds `[10,20][1]` through CUE postfix/range control. [Nested expression values](docs/nested-list-values.md)
now support `[[10,20],[30]][0][1]` with immutable sharing and bounded shape.
Open/abstract lists and general solving remain absent.
[Closed scalar records](docs/closed-records.md) now add bounded immutable JSON AST
values, CUE closedness/field lookup/length policies and exact object output.
[Computed JSON record fields](bootstrap/record-expressions.md) now use CUE-owned
iteration with checked scalar admission and the shared evaluation context;
source structs and general open-record solving are not yet supported.
[CUE-owned addition control](bootstrap/sum.md) selects checked integer addition
or concrete string concatenation. [Scalar equality](bootstrap/equality.md)
adds `==` and `!=` with CUE-owned concreteness guards and boolean result selection.
[CUE ordering](bootstrap/ordering.md) adds `<`, `<=`, `>` and `>=` for matching
concrete integers or text, preserving inclusive prefix bounds.
[Logical selection](bootstrap/logic.md) adds `&&` / `||`, preserving eager child
errors and selecting results/types in CUE. [Unary `!`](bootstrap/logical-not.md)
uses checked one-operand CUE control and source-prefix dispatch.
[Null and top](bootstrap/lattice-values.md) add concrete `null` and abstract `_`,
with CUE-owned identity, null-domain and concreteness guards.
[Checked integer dispatch](bootstrap/integer-control.md) also routes negation,
subtraction and multiplication through CUE over native checked primitives.
[Division policy](bootstrap/division-control.md) extends this to zero checks,
Euclidean/truncating primitive selection and the remainder unit-divisor case.
[Output admission](bootstrap/output-control.md) now selects concrete domains and
incomplete diagnostics in unary CUE control, with native extraction/JSON encoding.
[Unicode escapes](bootstrap/unicode-escapes.md) add `\uXXXX` and valid surrogate
pairs through CUE-only scanner policy. [Eight-digit scalar escapes](bootstrap/wide-unicode.md)
now use CUE admission with a checked native bridge to JSON conversion.
[Prefixed integers](bootstrap/radix-literals.md) use CUE sign/prefix/digit control
and checked i64 conversion, including signed MIN and inclusive thresholds.
[Computed bounds](bootstrap/computed-bounds.md) add parenthesized expression
thresholds, evaluated eagerly before the existing i64-only admission policy.
[Bound construction](bootstrap/bound-control.md) also delegates threshold admission
and inclusive endpoint layout to checked left-only unary CUE control.
Value representation and low-level primitives remain native.

## CUE all the way down

CUE will describe the parser, evaluator, constraint solver, and code generator.
Ordinary CUE evaluation is not a general-purpose programming runtime: we will
need to define executable meaning for the algorithmic constructs of our
CUE-hosted DSL. Designing that DSL is part of the experiment.

Handwritten Rust will provide the initial bootstrap machinery. As the DSL grows,
we intend to express that machinery in CUE and generate its replacement. Any
permanently native boundary, such as operating-system integration, should be
small, explicit, and justified.

Compiling our implementation to Rust is separate from evaluating users' CUE
files; those files need not themselves be compiled to Rust.

## Bootstrap path

1. Use a pinned upstream Go CUE implementation to evaluate our CUE sources into
   a deterministic intermediate representation.
2. Use an initial handwritten generator and runtime to build a Rust executable
   from that representation.
3. Move implementation machinery into the CUE-hosted DSL in small, testable steps.
4. Replace upstream CUE in the build with Cuestacean, then verify that successive
   self-builds produce identical canonical generated output under pinned inputs.

Bootstrap closure and CUE compatibility are separate milestones. Rebuilding
ourselves does not establish compatibility with the whole language. Differential
tests against pinned upstream CUE versions will guide compatibility work.

## Why?

For fun, for the delight of building the tool with the tool, and to explore
whether Rust and different implementation choices can make CUE faster.
Performance is a hypothesis to benchmark, not a promise.

## Development

Enter the pinned CUE and Shaka environment, then run the same gate as CI:

```sh
nix develop ./infra/tooling
shaka preflight
```

With direnv, `direnv allow` enters this environment from the repository root.
See [development tooling](infra/tooling/README.md) for regeneration commands.
CI builds and tests the bootstrap executor and small source frontend,
checks the tooling environment and project declarations, compiles generated
evaluators, rejects generated-file drift, verifies reproducible bootstrap
exports, compares interpreted and generated CLI behavior, and rejects malformed
expression trees and rule descriptions. Supported source fixtures are compared
with pinned upstream CUE.

Run the generated evaluator with its compiled-in rules:

```sh
nix build ./infra/tooling#exports --out-link result-exports
nix run ./infra/tooling#generated -- result-exports/addition.json
```

It prints `42`. The interpreter remains available through the default Nix entry
point for comparison. To evaluate the supported source subset:

```sh
nix run ./infra/tooling#source -- bootstrap/source/valid/nested.cue
```

This also prints `42`; see the [explicit subset limits](bootstrap/source/README.md).

- Nix pins the development environment and upstream CUE bootstrap tool.
- [Shaka](https://github.com/kolohelios/kolohelios/tree/main/tools/shaka)
  provides shared repository and validation tooling.
- Changes arrive as focused pull requests with reviews, full local validation
  and CI-gated auto-merge. Required checks remain enforced; no administrative
  bypass. See [development guidelines](AGENTS.md)
  and the [bootstrap checkpoint/continuation guide](docs/bootstrap-status.md).
- `.shaka/repo.cue` declares repository policy. Run `shaka repo audit` to check
  GitHub settings; `cuestacean validate` is the required CI check.

## License and upstream

Licensed under [Apache-2.0](LICENSE). Any material copied or translated from
upstream CUE will retain applicable copyright, license, and notice obligations.

Cuestacean is an independent experiment, not an official CUE project.
