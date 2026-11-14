# Cuestacean

A CUE implementation written in CUE, compiled to Rust. An ouroboros for fun.

Cuestacean is an experimental project to reimplement
[CUE](https://github.com/cue-lang/cue) with a CUE-hosted implementation DSL
and Rust as the execution target. The ambition is full self-hosting, not merely
using CUE to configure a handwritten Rust compiler.

There is no implementation yet. This repository starts with the plan.

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

## Development plan

- Nix will pin the development environment and bootstrap toolchain.
- [Shaka](https://github.com/kolohelios/kolohelios/tree/main/tools/shaka) will
  provide shared repository and build tooling, extended where necessary.
- GitHub Actions will test the build once build machinery exists.
- Changes will arrive as small, focused pull requests for maintainer review and
  manual merging; no automatic merging.
- Repository policy will be declared in `.shaka/repo.cue` and audited with Shaka
  in a follow-up. Required build checks will be enabled when those checks exist.

The first pull request deliberately contains only this README.

## License and upstream

Apache-2.0 is the intended license. A license file will follow before
implementation code is introduced. Any material copied or translated from
upstream CUE will retain applicable copyright, license, and notice obligations.

Cuestacean is an independent experiment, not an official CUE project.
