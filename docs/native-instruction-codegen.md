# Native instruction compiler construction (#282)

The final legacy compiler constructor built `bootstrap-codegen` against the
full Go-backed export bundle even after the [instruction policy DATA cut](native-instruction-lowering-policy.md)
made its canonical policy native. This slice changes who builds the existing
binary, not how it parses instructions or emits Rust.

## Constructor and consumer

The internal `bootstrap/native-instruction-codegen` Cargo façade uses the same
`bootstrap_executor` library name for the unchanged CLI. Its production source
set is exactly seven files: the façade manifest, lockfile and library, and the
existing CLI, typed compiler, lowering kernel and module assembly source. The
façade exposes the real compiler error and entry points. It does not import
executor runtime code or supply a stub. The existing executor API remains owned
by the executor crate.

The constructor uses empty features and exactly two whole native policy files:
`lowering.json` and `meta-lowering.json`. It does not consume the full bundle,
committed generated Rust, expectations, oracle binaries or verifier outputs.
The joined seed selects the isolated `bootstrap-codegen` alongside the five
existing binaries. The old bundle-backed compiler remains a verification-only
comparison; its canonical policy already comes from native DATA, so agreement
with that compiler alone is not independent Go construction evidence.

## Verification boundary

The feature derivative adds one test file. The native core lower compiler
regenerates both driver modules from native data before the four debug/release
and empty/compiled-lowering test and strict Clippy combinations. Evidence covers
complete independently predicted output, actual CLI error ordering and empty
stdout on failure, and real compiled driver execution and recovery.

The separate sandbox verifier compares complete default, left-twice and reverse
source with the legacy CLI and committed formatted goldens. It also replays all
17 retained source-effect cases from the policy verifier, with both native and
raw Go policy inputs and complete independent output expectations. Those inputs
are verification evidence, never seeds. Existing executor regeneration and
compiled alternate/adversarial execution remain in place.

Six native compiler constructors and their actual production use do not by
themselves establish a Go-free joined closure. Audit that closure separately
from the feature and constructor closures. Fixture PROGRAM and schema exports,
full executor bootstrap, general defaults and upstream CLI/API compatibility
are outside this slice.
