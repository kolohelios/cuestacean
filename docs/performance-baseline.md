# First performance baseline

Measured 2026-11-22; [raw series](../benchmarks/results/baseline.tsv).
The contract below was reviewed before implementation; actual results follow it.

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

## Actual first run

Source/fixture/tool hashes were frozen before sampling. Both focused functional
reviews approved `618df21514567cb55c244fcd52beb92dea86ecdd`; that revision identifies
the measured harness. Production source remains baseline `01487845` above, with
no bootstrap-tree changes. Final report/results additions do not change the tools.
The two independent contract GOs covered `c650aeea408293b2db4ccfe27cf538c6d4327d93`.

Coordinator-only serial run: **2026-11-22 22:32:25–22:34:02 UTC**. Runtime completed
at 22:32:28. All validation jobs had finished; workers were quiescent and a 30-second
quiet interval preceded sampling. No exclusive-host or continuous-idleness claim:
this was a shared NixOS machine with background desktop applications.

Environment: x86_64 Linux 6.18.60; Intel Core i7-12800HX, 24 logical CPUs/affinity
0–23; MemTotal 65,536,920 kB; CPU0 governor `powersave`. No CPU affinity, frequency,
cache, root or sysfs changes. Runtime load averages before/after were
`[0.2334,1.1118,3.3022]` / `[0.2949,1.1099,3.2896]`; bootstrap ended at
`[1.5151,1.3179,3.1509]`. Nix 2.34.9, Python 3.13.17, GNU Time 1.10, CUE 0.16.1
(built with Go 1.26.10); production Rust 1.95.0, release build/default features.
Nix settings were cores=0 (autodetect), max-jobs=24, sandbox=true; the recipe also
explicitly disabled substitution and used offline mode. Bootstrap inherited the
ordinary coordinator Nix environment; runtime children used the controlled
contract environment, not ambient Go/CUE tuning. No build-daemon memory was sampled.

### Source CLI results

ALL 24 initial expected-value checks and every warmup/measurement output passed.
Five warmup rounds, 31 latency rounds and five separate RSS rounds, seed 1729:
1,008 successful child executions (24 validation +120 warmup +744 latency +120 RSS).
Round order is reconstructed by one Python `Random(1729)`, shuffling case-file
order × `source-cli,go-cue` each round, continuing across all three stages.

Latency columns are **median / IQR in milliseconds**; RSS column is **median KiB,
Cuestacean / Go CUE**. Quartiles are inclusive. UTF-8 subprocess text decoding and
Python launch/collection overhead are included; JSON parsing/comparison is outside.

| Case | Cuestacean ms | Go CUE ms | Peak RSS KiB |
|---|---:|---:|---:|
| literal-zero | 0.826 / 0.071 | 3.995 / 1.674 | 2556 / 20856 |
| arithmetic | 0.865 / 0.090 | 4.609 / 1.806 | 2556 / 20912 |
| chain16 | 0.880 / 0.101 | 4.225 / 1.795 | 2748 / 20916 |
| chain31 | 0.971 / 0.063 | 4.245 / 1.342 | 2748 / 22068 |
| bounded-integer | 0.840 / 0.115 | 4.058 / 1.209 | 2556 / 20788 |
| ascii-concat | 0.846 / 0.084 | 4.206 / 1.442 | 2620 / 20212 |
| ascii-length | 0.823 / 0.075 | 4.509 / 1.520 | 2556 / 20724 |
| list8 | 0.883 / 0.083 | 4.583 / 1.465 | 2556 / 21108 |
| list63 | 0.951 / 0.129 | 4.008 / 1.620 | 2808 / 21036 |
| nested8 | 0.873 / 0.085 | 4.823 / 1.417 | 2604 / 21556 |
| slice16 | 0.916 / 0.116 | 4.226 / 1.718 | 2620 / 20464 |
| record-selector | 0.856 / 0.123 | 4.364 / 1.617 | 2612 / 20404 |

Encouraging for these tiny CLI workloads: lower observed median latency and RSS.
This does NOT establish parser/evaluator throughput, large-input behavior, full-CUE
performance, compiler speed, allocation efficiency or a language-wide advantage.
Startup, grammar loading/validation, output formatting and Go runtime initialization
are all included; neither their individual costs nor their causes were isolated.
Wide Go CLI IQRs and a shared hybrid-core host limit extrapolation. No significance
claim, combined weighted score, optimization or regression threshold is introduced.

### Bootstrap results

These exclude flake evaluation, dependency acquisition and readiness realization.
All three targets already had available dependencies. Seven cached samples each;
three separately forced sandbox target rebuilds each. Times are parent-observed
Nix invocation wall time, not isolated constructor CPU time.

| Existing output | Cached median ms | Forced rebuild seconds, in order |
|---|---:|---|
| exports: full 249-JSON bundle | 52.772 | 22.949, 23.278, 23.504 |
| native-instruction-operand-cases: partial DATA | 53.913 | 0.122437, 0.120043, 0.120379 |
| native-codegen-core: compiler construction | 52.956 | 7.604, 7.553, 7.521 |

Nine actual forced-rebuild commands completed, three per root, plus 21 cached
samples and six untimed readiness/identity builds. Complete output/deriver/NAR,
derivation/capture/script/tool hashes were unchanged before/after. These different
products cannot support a native-versus-Go construction speedup ratio. Approximately
53 ms cached resolution is workflow/client overhead, not parser execution time.

### Durable provenance and reproduction

Exact derivations (prefix `/nix/store/`):

- Production: `2nsqix06mckydld41yhsjdw05jjba0qx-bootstrap-executor-0.1.0.drv`.
- Exports: `f1c19r2qs44rmarikii2rlkdp63h11av-cuestacean-bootstrap-json.drv`.
- Operand DATA: `6xb8jkh3j2w5rbi806xxw0fsrcdqx7vc-cuestacean-native-instruction-operand-cases.drv`.
- Core compiler: `w92pbwvnrxqln1g2k200wkcckrva20cs-cuestacean-native-codegen-core-0.1.0.drv`.

Measured providers and SHA256 (all paths relative to `/nix/store/`):

| Provider | Store path | SHA256 |
|---|---|---|
| source CLI | `q9qmv6llmw0y2s5rvh024dng0vfypjgj-bootstrap-executor-0.1.0/bin/bootstrap-source` | `b40f354b7f84a9cfe1b46a98ccad7dd0c1fac7bf2391e2c66f72fb683c9152bf` |
| Go CUE | `nzipa1yaw7f0zbdzkvqmqii100bzf5m2-cue-0.16.1/bin/cue` | `e67cd476bb3e54cab81b60a3a041fdc969377d659001984368bfe28a77ec294a` |
| GNU time | `3xrn42w74kwf5il7qgs7fmpqz3380zq1-time-1.10/bin/time` | `18b67e22f2611d5f77102756a39b80a6b4d5151ba5eaf0e48613707f029e6c9e` |
| Python | `kp6p38x6zb21n7gdlnp9nsm2aagq7sg2-python3-3.13.17/bin/python3.13` | `dda1d953a0b486f62e83f9a23bc1de48940ea2d6dbd74ad20896a4d75d2c3089` |
| Nix | `wh4dksj5jz315f46g5rrn65mfyhjm23h-nix-2.34.9/bin/nix` | `126360e93520157ba98fc010c51e14f812d0f3ac3e79be417cb142b5a87c9ee6` |

Scripts SHA256: runtime `bda9915b33b45c8d821f5c34cfc72c9942284a83bc3cabb0937023390cbe9261`;
bootstrap `e0a31e959304193de44312a22b48233c953e6ee14f256f102b4b444f44c9379f`;
fixtures `8454b72e7ee37e1a83ec42dfb95159b8200435c1836c87d33e9a1d8226bc8298`.
Raw durable TSV SHA256 `afa818ca3db1ad1347d88c277433dd120c368cb4ae323902fefbe21a40485cc7`.
Original runtime TSV was copied, not modified, before appending six bootstrap rows.
All 54 raw series retain every sample; summaries above are rounded displays only.

Build-only local metadata receipts are `/tmp/cuestacean272-{runtime,bootstrap}-baseline/metadata.json`:
runtime SHA256 `457fe0fc3d101b15b465aebbd4c4bee287de2879dadb932b1d618ae3a0f79489`;
bootstrap `b1719d4c788e3e4d5634c660c9ad100d32f2f6a021e4c338fff3ab797bbe917f`.
They contain actual schedules/argv/host/provider details; durable inputs, source
identity, protocol and full raw samples are retained here and in version control,
not solely in those disposable receipts. Lockfile pins were unchanged.

For a new run, enter `nix develop ./infra/tooling`, run
`python3 -B -m unittest discover -s benchmarks -p test_run.py -v`, capture package
and check maps ONCE, to new absolute output files:

```sh
nix eval --json ./infra/tooling#checks.x86_64-linux \
  --apply 's: builtins.mapAttrs (_: v: v.drvPath) s' > "$CHECKS"
nix eval --json ./infra/tooling#packages.x86_64-linux \
  --apply 's: builtins.mapAttrs (_: v: v.drvPath) (builtins.removeAttrs s ["default"])' > "$PACKAGES"
```

Realize explicit roots outside timers; obtain provider outputs from
those captured derivations, verify their bytes/source, then use the runtime argument
protocol above. Invoke `python3 -B benchmarks/bootstrap.py ABS_NIX ABS_PACKAGES_JSON
NEW_ABS_DIR FULL_HARNESS_REV`. Use new output directories; do not resume, silently
replace providers, recapture named roots during sampling or assume old store paths
are locally available. Copy runtime TSV and append bootstrap TSV without its header.

### Completed local validation and limitations

Before timing: 11 focused Python tests and 24 real untimed CLI checks; both exact
functional GOs; current 29-check/26-package identities equal baseline (55 names,
54 roots). All 54 roots were realized, then separately forced rebuilt successfully.
Fresh exact-set/directory/EVERY-byte audits covered production/legacy 1,351 files,
four compiler cuts, 423 export CUE files and two raw operand files; no ambient
targets, symlinks, untracked inputs or extra empty directories. All 249 JSON outputs,
generated Rust, five native policies/binaries and six origins remain unchanged.
Actual Rust matrix: executor 775 twice; exporter 164 FOUR profiles; research 135
FOUR; three boundaries four groups EACH FOUR; 14 strict Clippy completions,
2,794 named passes/400 complete blocks. Independent receipt audits passed.

Correct flake-pinned Shaka preflight passed 13/13 (seven skips). An earlier attempt
used a stale store helper and failed; it was retained separately, not waived.
An initial receipt parser rejected the exact pre-Cargo dirty-tree setup warning;
a separately reviewed corrected parser classified only that witnessed setup line,
retaining all Rust warning/error/count/source checks. Neither issue changed repository code,
checks, fixtures or results. Both measurement stages exited zero on their first
attempt, without retry, discarded sample or partial-result promotion. Broader #175
holds remain.
