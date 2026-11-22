# First performance baseline

## Measurement contract (before implementation)

Production baseline: `0148784521f1f1e5e51cfdcaa979dae6721c594d` (PR #271).
This slice measures existing behavior, not new language capabilities or
optimizations. Implementation requires two independent focused contract reviews
and an explicit coordinator GO. The earlier, broader instrumentation proposal
is held; its estimates and any previous slice's budget exception are not authority.

### Scope and limits

- Compare the production Cuestacean source CLI and pinned Go CUE source CLI on
  twelve identical supported concrete expressions with independent expected JSON.
- Record end-to-end process latency and separate child peak RSS measurements.
- Measure cached realization and forced target rebuilds for three existing
  bootstrap outputs: `exports`, `native-instruction-operand-cases`, and
  `native-codegen-core`.
- Add only benchmark files, documentation/results and Python3/GNU time devshell
  tools. No production Rust, Cargo metadata, schemas, generated code, CUE policies,
  constructor/source cuts, features, guards, packages or checks change.
- Defer in-process parser/evaluator instrumentation, API throughput, pure startup,
  extra JSON CLI comparisons, dependency-footprint enumeration, cold bootstrap,
  optimization and automatic regression thresholds.

These tiny inputs exercise a bounded common subset, not application workloads,
full CUE compatibility, self-hosting or language-wide performance. Existing source
limits remain 64 AST nodes and 32 parsing levels. No performance advantage is
presumed. Literal zero is a minimal workload, **not** a pure startup measurement.

### Ownership and budget

Two isolated stable-parent workspaces, at most two workers:

- Harness worker: only `benchmarks/run.py`, `benchmarks/test_run.py`, and
  `benchmarks/cases.json`.
- Tooling/reviewer worker: only two devshell package additions in
  `infra/tooling/flake.nix`; independent fixture, protocol and execution audits.
- Coordinator: this document, `benchmarks/bootstrap.py` (explicit three-target
  recipe, not a general build framework), durable raw TSV and brief documentation
  links; integration, source/provenance checks and final validation/merge.

The complete independent reduced estimate is 1,046–1,260 changed lines, including
additions, deletions, formatting, contract/report, raw evidence, recipe, tests and
100–120 lines of repair reserve. Ordinary approximately 1,500-line bound applies.
Do not spend spare budget on deferred features. Pause at the first credible
budget overrun, ambiguity, source-identity defect or unresolved validation failure;
never delete proof, weaken assertions or golf source to fit. Saved handoffs are
immutable; repairs use new children and integration is owned-path only.

### Fixture and correctness boundary

Handwritten manifest schema: `{version: 1, cases: [{id, source, expected}, ...]}`.
Use exactly twelve unique safe IDs and exact UTF-8 source bytes:

1. Literal `0` → integer zero.
2. `(1 + 2) * (3 + 3)` → 18.
3. Sum of integers 1 through 16 → 136.
4. Sum of integers 1 through 31 → 496 (61 AST nodes).
5. `int & >=0 & <=100 & 42` → 42.
6. ASCII string concatenation → `"hello world"`.
7. Length of 26 lowercase letters, 26 uppercase letters and ten digits → 62.
8. Literal list of integers 0 through 7 → the same list.
9. Literal list of integers 0 through 62 → the same list (64 AST nodes).
10. Eight nested one-element lists containing 1 → that nested list.
11. Literal list 0 through 15 sliced `[4:12]` → integers 4 through 11.
12. Closed record with fields `a` through `h` valued 0 through 7, selecting `h` → 7.

Expected values are independently handwritten, never captured from either tool.
No defaults, references, abstract containers, Unicode-length mismatch or unsupported
Go-specific transformation. Both CLIs receive the **same** source file. Validate
ALL cases against BOTH CLIs before any recorded latency/RSS sample; also validate
every measured output. Recursive JSON comparison checks exact types (boolean is
not integer), object members and ordered list contents with exact integers.
Reject duplicate keys, nonfinite numbers, floats in this corpus, trailing/partial
JSON, unexpected stderr, nonzero exits and timeouts. Setup/fixture failures must
not produce a completed baseline or be relabeled semantic successes.

### Runtime harness interface and outputs

Invoke an explicit absolute pinned Python3 executable with `benchmarks/run.py`:

```text
--source-cli ABS --cue ABS --time ABS --cases ABS --output NEW_ABS_DIR
--revision FULL40 --harness-revision FULL40
[--samples 31 --warmups 5 --rss-samples 5 --seed 1729 --timeout 30]
```

Verify resolved provider paths are regular, readable and executable before use;
no ambient discovery, shell evaluation or stale-artifact fallback. Revisions are
explicit declarations independently checked by the coordinator, not VCS discovery
inside timing. Refuse existing output directories and invalid counts/revisions. Samples must be
2–1,000; warmups/RSS samples 1–1,000; seed 0–4,294,967,295; timeout finite,
positive and at most 3,600 seconds. Revisions are exactly 40 hexadecimal digits.
Success exits zero with empty stdout/stderr; failures exit two with contextual
stderr and no completed baseline. Help exits zero with argparse usage/options.
Run direct argv: `[source_cli, source_path]` and
`[cue, "export", "--out", "json", source_path]`. Materialize inputs outside timers.
Use an explicit child environment with owned HOME/XDG_CONFIG_HOME/XDG_CACHE_HOME/
TMPDIR, PATH=/nonexistent, LANG=LC_ALL=C.UTF-8 and TZ=UTC, plus owned cwd. Do not
inherit Go tuning, CUE experiments or credentials; record this environment without
modifying the user's configuration. Capture `cue version` and GNU `time --version`
outside timers; the source CLI has no version flag and must not be probed with one.

Clock `perf_counter_ns` immediately around direct subprocess execution/stdout
collection; JSON decoding/comparison occurs after stopping it. Latency includes
process startup, file reads, parsing/evaluation, encoding, output and process wait.
No builds or GNU time wrappers occur in latency samples. Five warmup rounds precede
31 recorded rounds; seed-shuffle all case/engine commands each round and retain
actual order. No outlier removal, startup subtraction, p99 or significance claims.

Use five **separate** GNU time invocations per case/engine for RSS, `%M` in a
separate owned file; require exactly one nonnegative integer and check child output.
Report Linux KiB peak child-process RSS, not Python's accumulated child maximum,
per-API allocation, live graph size or Nix daemon/build-tree memory.

Publish `raw.tsv` plus build-only `metadata.json` in the new output directory.
Write a completion marker in metadata LAST; failed/partial output is never complete.
TSV has seven columns: `track`, `case`, `engine`, `metric`, `unit`, `iterations`,
`samples`. Samples are ordered comma-separated integers, not a count. Runtime
engines are `source-cli`/`go-cue`, metrics `latency`/`peak-rss`, units `ns`/`KiB`,
iterations 1. Twelve cases produce 48 rows plus the header. Metadata records argv,
settings, actual schedule, hashes, versions, source/harness revisions, UTC, CPU,
kernel, RAM, affinity/governor when readable, clock details and load before/after.
Summaries use count, median, inclusive quartiles, min/max; derive from raw samples.

### Bootstrap recipe

Capture CURRENT named package derivations ONCE from repository-filtered
`./infra/tooling` after source freeze, never a broad `path:$PWD` source. Keep the
capture immutable. Select only the three named outputs above. Verify explicit
regular derivation files and single-output identities; record derivation hashes,
output paths, NAR hashes/sizes and actual input/source identities before timing.

The coordinator's explicit Python recipe receives absolute pinned Nix and the
captured roots file, realizes readiness outside timing, then uses the SAME explicit
`DRV^*` roots: seven cached `nix build --offline --no-link --json` samples and three
forced samples adding `--rebuild`. Time direct Nix child wall-clock, parse/check
outputs afterward, retain stderr diagnostics and actual order, and verify fixed
root/output/NAR identities before/after. Failure or drift aborts completion.
Publish six rows in the same TSV schema, track `bootstrap`, engine `nix`, metric
`cached`/`rebuild`, units `ns`, iterations 1, plus build-only provenance metadata.

These are **target rebuilds with warm dependencies**, not cold page cache, clean
store or bootstrap-from-zero. No GC, cache deletion, page dropping, root/sysfs
writes or network-fetch measurement. Wall time includes Nix client/daemon
coordination and target actions; do not claim isolated constructor CPU time or
build-tree memory. Full 249-export bundle, partial operand DATA and compiler
construction are different outputs, not like-for-like compiler speed comparisons.

### Validation, measurement and release gates

Before timing: focused Python tests cover strict JSON/type handling, invalid
paths/counts/revisions, all-case validation before sampling, timer boundaries,
seeded order, raw TSV/statistics, RSS separation and all fail-stop paths. Exercise
real pinned CLIs on the complete corpus. Independently review recipe identity
checks, actual captures and execution receipts, not just intended commands.

Verify all production source/generated artifacts remain unchanged and current
package identities against the merged baseline; retain source/derivation/binary
hashes and production release/default-feature evidence. Existing full pinned
Shaka preflight, Rust profile checks, real sandbox build AND rebuild, deterministic
generation and independent upstream expected results remain required. Do not call
cached realization a rebuild or pending checks passed. Exact-source audits compare
sets, directories and every byte, rejecting ambient targets/untracked/symlink inputs.

Freeze source and complete functional reviews/tests before coordinator-only serial
measurement, with workers no longer compiling. Record shared-host load; no exclusive
CPU, fixed-frequency or continuously monitored idle claim. Keep original failed
attempts and corrected runs separate. Commit readable method/results/limitations,
full durable raw TSV (55 lines including six bootstrap rows) and durable provenance;
JSON output remains build-only. No results are claimed until actually measured.

Two final independent exact immutable whole-tree GOs follow integration and actual
validation. Retire workers/panes/workspaces before PR. Complete focused reviews and
full local preflight, enable protected exact-head CI-gated rebase auto-merge, watch
required CI through confirmed merge, verify the merged tree and fetch clean main.
