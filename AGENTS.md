# Cuestacean development

Read [the bootstrap checkpoint](docs/bootstrap-status.md) before selecting the
next slice; consult the linked component contracts before changing them.
The checkpoint is anchored to a merged milestone, not a promise of full CUE
compatibility or self-hosting.

## Workflow

- Use Jujutsu (`jj`) for version control and Conventional Commits.
- Deliver coherent vertical slices, normally at most approximately 1,500 changed
  lines including generated Rust. Prefer capability/architecture progress over
  repeatedly splitting tiny lexer changes into separate PRs.
- Open PRs, complete focused reviews and full local preflight, then enable
  CI-gated auto-merge. Watch required CI through confirmed merge; fix failures
  rather than bypassing checks or using administrative merge overrides.
- After a confirmed merge, work may continue from fetched `main@origin` without
  waiting for a maintainer prompt. Keep slices bounded; pause for ambiguous
  design/scope, security-sensitive changes or unresolved validation failures.
- Use pinned Nix tooling and Shaka. Run full preflight and real sandbox builds;
  do not bypass failing checks or claim a pending check passed.
- Keep CUE-generated JSON build-only. Keep generated Rust committed for now,
  regenerate deterministically, and retain independent expected results and
  upstream comparisons. Seeds must not depend on committed generated artifacts.
- Preserve Apache-2.0 and applicable upstream notices for copied/translated work.

## Parallel work

- Use Herdr for visible coordination when available in the managed session.
  Check `HERDR_ENV=1` before control and consult the installed CLI/skill.
  Keep at most two workers active; Terra is acceptable for bounded tasks.
- Agree interfaces and distinct file ownership first. Give concurrent editors
  isolated `jj` workspaces. Verify actual cwd and returned pane IDs; do not infer
  IDs or rely on UI focus. Avoid concurrent formatting of others' files.
- One coordinator integrates, reviews, regenerates artifacts and owns final
  sandbox/CI validation. Reuse workers immediately for focused review when useful.
- Retire workers and close only the panes created for them by PR creation,
  unless immediately reusing them. Forget/remove their workspaces once changes
  are safely integrated. Do not leave unused workers running.
- Durable knowledge belongs in visible project docs, not private memory or
  temporary worker prompts.
