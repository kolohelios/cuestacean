# Cuestacean

A CUE implementation written in CUE, compiled to Rust. An ouroboros for fun.

Cuestacean is an experimental project to reimplement
[CUE](https://github.com/cue-lang/cue) with a CUE-hosted implementation DSL
and Rust as the execution target. The ambition is full self-hosting, not merely
using CUE to configure a handwritten Rust compiler.

There is no CUE compiler implementation yet. The [bootstrap fixtures](bootstrap/README.md)
define integer-literal and addition trees, plus proposed evaluation rules, in
CUE and export them to checked-in JSON using upstream CUE. A small
[bootstrap Rust executor](bootstrap/executor/README.md) interprets those rules.
An initial [rule-to-Rust generator](bootstrap/generated/README.md) lowers the
same rules to compiled evaluator modules. Nix builds a handwritten generator
seed, regenerates JSON and Rust from the CUE specifications, then compiles and
tests the final executables. The generated evaluator is runnable; CUE evaluation
still relies on the upstream Go tool.

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
There is no CUE compiler build yet. CI builds and tests the bootstrap executor,
checks the tooling environment and project declarations, compiles generated
evaluators, rejects generated-file drift, verifies reproducible bootstrap
exports, compares interpreted and generated CLI behavior, and rejects malformed
expression trees and rule descriptions.

Run the generated evaluator with its compiled-in rules:

```sh
nix run ./infra/tooling#generated -- bootstrap/addition.json
```

It prints `42`. The interpreter remains available through the default Nix entry
point for comparison.

- Nix pins the development environment and upstream CUE bootstrap tool.
- [Shaka](https://github.com/kolohelios/kolohelios/tree/main/tools/shaka)
  provides shared repository and validation tooling.
- Changes will arrive as small, focused pull requests for maintainer review and
  manual merging; no automatic merging.
- `.shaka/repo.cue` declares repository policy. Run `shaka repo audit` to check
  GitHub settings; `cuestacean validate` is the required CI check.

## License and upstream

Licensed under [Apache-2.0](LICENSE). Any material copied or translated from
upstream CUE will retain applicable copyright, license, and notice obligations.

Cuestacean is an independent experiment, not an official CUE project.
