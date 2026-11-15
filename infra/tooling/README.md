# Development tooling

This infrastructure project owns Cuestacean's pinned CUE, Rust, and Shaka
environment and the disposable [bootstrap executor](../../bootstrap/executor/README.md)
package. The crate remains bootstrap scaffolding, not a CUE implementation.

From the repository root:

```sh
nix develop ./infra/tooling
shaka preflight
```

`project.cue` owns the generated `justfile` and GitHub Actions validation
workflow. Regenerate them from the repository root with:

```sh
shaka project generate-justfiles
shaka ci generate-workflows
```

The flake is intentionally handwritten: Shaka's infrastructure flake template
does not yet expose packaged Shaka as a development-shell dependency. Its
lockfile pins both FlakeHub inputs and upstream CUE through Nixpkgs.

Within this directory, `just validate` checks Nix formatting, flake evaluation,
whitespace, and the [bootstrap export](../../bootstrap/README.md). The flake's
`bootstrap-export` check builds the `exports` output: it exports each CUE
fixture twice, compares the results, and requires malformed expression trees
and evaluation-rule descriptions to fail export. JSON is not committed. The
`bootstrap-executor` check builds handwritten evaluator/scanner/AST/Pratt compiler seeds
without intermediate artifacts or the default `source` feature. It then uses
the fresh exports to regenerate Rust before compiling the final package with
the source frontend enabled. The final
stage runs formatting, Clippy, the complete library/CLI tests,
compiled generated evaluators, interpreter/generated CLI parity, and integer
boundary cases in the Nix sandbox. The [scanner DSL](../../bootstrap/scanner.md)
adds interpreted/compiled control-flow comparisons and cursor/fuel guard tests.
[AST builder](../../bootstrap/ast.md) checks cover typed expression lowering,
argument ownership, escaped names, and parser tree parity. [Pratt control](../../bootstrap/pratt.md)
checks compare both execution modes over the source corpus and exercise alternate
associativity, frame state, arithmetic and fuel guards.
The `bootstrap-codegen` check verifies deterministic Rust generation and rejects
committed-module drift. `nix build ./infra/tooling` from the
repository root builds that same package without a separate publishing job.
Build example JSON with `nix build ./infra/tooling#exports --out-link result-exports`.
`nix run ./infra/tooling#generated -- result-exports/addition.json` runs the
generated evaluator; the default entry point still runs the interpreter.
The devshell supplies the same export output as `CUESTACEAN_BOOTSTRAP_JSON`
for Cargo tests and the embedded grammar; a clean checkout needs no manual
regeneration. Re-enter the shell after editing CUE specifications.
`nix run ./infra/tooling#source -- bootstrap/source/valid/nested.cue` uses the
small source frontend. Its `bootstrap-source` Nix check differentially compares
all supported source fixtures against upstream CUE, requires both tools to
reject conflicts and incomplete exports, and checks unsupported-input cases.
Repo-wide `shaka preflight` also validates the CUE project schema and rejects
drift in generated files or unaccounted GitHub Actions workflows. `taplo.toml`
excludes direnv and Cargo caches from formatting, not project TOML sources.
