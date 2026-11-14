# Development tooling

This infrastructure project owns Cuestacean's pinned CUE and Shaka environment.
It contains no language implementation or Rust scaffold.

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
`bootstrap-export` check exports each CUE fixture twice, compares both exports
with its committed JSON artifact, and requires malformed expression trees to
fail export. Repo-wide `shaka preflight` also validates the CUE project schema
and rejects drift in generated files or unaccounted GitHub Actions workflows.
