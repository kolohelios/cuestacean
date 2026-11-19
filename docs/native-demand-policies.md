# Native construction of experimental demand policies

## Reviewed contract

Milestone [#182](https://github.com/kolohelios/cuestacean/issues/182). Two focused
Nix/boundary and independent-evidence reviews preceded implementation. The initial
successful comparison probe is feasibility evidence, not sandbox attestation;
actual builds/full graph inspection remain required below.

After [#180 executable dispatch](demand-policy.md), close the **policy artifact
construction** cut for the two existing test-only programs. The unmodified native
exporter already exports both raw schema/program closures byte-identically to
pinned CUE at current limits. No parser/evaluator/DSL/guard expansion is needed.
The conservative graph dispatch remains test-only; nested preference lifetime is
unresolved and [#175](https://github.com/kolohelios/cuestacean/issues/175) stays held.

This is native bootstrap export of CUE-authored represented instructions, not
CUE-hosted specification evaluation, a production graph/default implementation,
closure of the six compiler policies, full self-hosting or a performance result.

The [#184 projection-policy extension](preference-projection-policy.md) expands
this artifact family explicitly to six raw files/four programs and re-attests its
new producer. The historical #182 two-policy contract and hashes below remain
evidence; the original two dispatch JSON byte results stay unchanged. This does
not widen the original AST cuts or the exporter production fileset.

## Exact raw input and artifact closure

Add `native-demand-policies`, receiving only the independently built exporter
and a separate source fileset of exactly three unchanged files:

- `bootstrap/spikes/demand-policy-schema.cue`
- `bootstrap/spikes/demand-policy.cue`
- `bootstrap/spikes/demand-policy-defer.cue`

Each process loads the schema and **one** program. Produce `demand-policy.json`
and `demand-policy-defer.json` twice in separate empty-environment/PATH-unusable
processes, compare bytes and publish build-only JSON. No Go CUE executable/source,
pre-exported policies, expected answers, generated Rust/JSON, graph test/runtime
or ambient checkout is a construction input. No native failure may retry in CUE.
Do not widen the original atom/family raw cuts or the exporter’s eight-file
production source allowlist. All four current native exporter/atom/family/full-
JSON derivation identities must remain unchanged.

The artifact’s whole build dependency graph is independently attested from both
exporter and artifact **derivation paths**, not output references alone. Retain
build paths, registration and exact roots. Deny pinned Go CUE/source/full JSON and
downstream executor identities; inspect the full graph and allowlisted source
rather than treating that identity denylist as a universal foreign-code proof.

## Native provider and separate oracle

Expose mandatory `CUESTACEAN_DEMAND_POLICIES` in the devshell and Rust test package
pointing to this artifact directory. The graph policy integration test loads the
native JSON and admits it through the existing strict Policy::from_json. Missing
environment/artifact is a setup failure, not permission to export via Go CUE or
use a committed/cached answer. Kernel/VM code still does no filesystem/process IO.

Canonical and alternate graph behavior must now execute **native-built policy
JSON**. Preserve all policy mutation, certificates, old hypothesis/readiness,
work accounting and independent semantic expectations. Pinned Go CUE remains
mandatory in separate raw-source comparison and semantic-oracle tests, never as
the policy provider or input to artifact construction. Its source inputs are raw
CUE, not the native result. Do not claim the entire check package is Go-CUE-free.

Add a separate `native-demand-policies` Nix verification check beside the existing
eight checks. It may depend on Go CUE and jq to compare fresh raw-source exports
with native bytes, while retaining the full native graph proof. The existing
Rust sandbox supplies downstream execution evidence from the mandatory native
provider; do not make the artifact depend on its verifier or consumer.

```text
minimal exporter sources -> native exporter ----+
exact raw schema + two programs ----------------+-> native policy JSON
                                                      |
                                                      v
                                           strict decoder + checked graph VM
                                                      (test-only consumer)

native policy JSON -> separate Go CUE/raw-source verifier + full graph evidence
```

The Rust package/seed derivations may legitimately gain the **native artifact**
as a test-environment dependency, not the Go-CUE comparison derivation. The six
fresh artifact-free compiler binaries and profile isolation must remain intact;
this does not establish their Go-CUE independence or embed test policy JSON in
those binaries. Production specification export membership stays unchanged.

## Evidence and agreed ownership

Before implementation complete two focused contract reviews. Nix owner:
`infra/tooling/flake.nix` only, including explicit raw filesets, producer, graph
proof/verifier, package/devshell/test environment and spec-export test fixtures.
Evidence owner: `bootstrap/executor/tests/demand_policy.rs` and new
`bootstrap/spec-export/tests/demand_policies.rs`. Coordinator owns docs, immutable
commit integration, source/cut inspection and final validation. Use at most two
stable-main workers with no rebase onto a mutable coordinator parent.

Required independent tests, not generated/serialized native answers as inputs:

- Handwritten expected policy trees for canonical dispatch and alternate defer;
  compare exporter library, native CLI and pinned CUE to those expectations.
- Exact source boundaries, input order, repeat bytes, arbitrary temporary file
  names, cleared CLI environment/PATH and mutated valid/invalid actions/fields.
  The exporter must not recognize a policy filename/opcode/result specifically.
- Lowered shared work failures, failure recovery, explicit incompatible or
  incomplete policies and CLI semantic failures publishing no partial JSON.
- Existing graph integration runs the native artifacts, retains handwritten
  action/demand expectations and canonical-vs-reference parity, and demonstrates
  alternate policy behavior without recompiling Rust. No consumer fallback.
- Separate pinned raw CUE comparison and semantic probes remain mandatory. Pin
  source/version honestly: CUE language v0.16.1/source
  `6d609d768f1686f9a3a2a20197cacdbb70e5c79d`; `(devel)` is not source attestation.

No code under either crate’s production src/, raw production/test-policy CUE,
existing hard guards, Cargo dependencies or generated Rust should change.
`specSources(false)` remains exactly Cargo metadata + six Rust files; only its
separate test variant gains these three raw fixture files. Generated JSON remains
build-only; no expected-answer/native output enters the producer.

Run focused debug/release × default/no-default consumer tests, native exporter
and both full Rust suites/strict Clippy, full pinned preflight, all **nine** Nix
checks plus explicit native artifact builds/rebuilds. Retain six fresh seeds,
feature isolation/regeneration/drift, all twelve builder exports and independent
source/value checks. Compare complete production JSON and inspect all native
build graphs/filesets. Only x86_64-linux is validated. Keep the slice around
1500 changed lines including docs/tests/Nix; no raised guards or lint suppression.
A later contextual preference policy/production proposal still requires review.

## Reproduce

```sh
nix build --no-link ./infra/tooling#native-demand-policies
nix build --no-link ./infra/tooling#checks.x86_64-linux.native-demand-policies
nix develop ./infra/tooling --command cargo test \
  --manifest-path bootstrap/executor/Cargo.toml --test demand_policy
nix develop ./infra/tooling --command cargo test \
  --manifest-path bootstrap/spec-export/Cargo.toml --test demand_policies
```

The devshell supplies the mandatory native artifact path; direct consumer tests
outside that environment must fail setup rather than reconstruct a policy.

## Findings and validation

Both complete policy trees now come from native construction. The graph consumer
requires the artifact directory and loads native JSON through strict admission;
it no longer invokes CUE to obtain policy inputs. Two isolated child probes remove
the environment or supply an empty directory with unusable PATH, confirming setup
failure before any CUE provider/recovery path. The separate raw-source comparison
and scalar semantic oracles remain mandatory and are not construction inputs.

Five new exporter tests compare full handwritten trees with native library/CLI
and pinned CUE, reverse input and declaration/field order, rename schemas/files,
repeat exact bytes in cleared CLI environments, and mutate both valid vocabulary
and invalid/incomplete/conflicting fields. A schema mutation from `defer` to
`pause` exports generic data successfully: VM validation remains downstream, not
a filename/opcode rule hidden inside the exporter. Lowered-budget failures recover
without cached answers and semantic CLI failures publish no partial JSON.

On x86_64-linux those five tests pass debug/release with strict Clippy; all ten
native-provider graph-policy test groups pass debug/release × default/no-default
with strict focused Clippy in all combinations. Existing handwritten behavior,
alternate-policy effects, certificates/shared work, static falsification and
conservative nested deferral remain unchanged. Both full Rust suites pass debug/
release and strict all-target Clippy. Final pinned preflight passes 13/13 (7
inapplicable checks skipped); all nine Nix checks and all native artifacts pass
actual sandbox builds and explicit rebuilds with the integrated consumer tests.
Six fresh artifact-free compiler binaries, feature isolation, all twelve builder
exports, regeneration/drift and source/value evidence remain intact. Other
architectures are omitted, not validated.

The new native construction graph has **1407 derivations/3392 build paths**.
Its retained roots are the exporter derivation
`a92mcym225xgv8lspwravw2w1pq5rraa-cuestacean-spec-export-0.1.0.drv` and artifact
`af1xs4sa35df8lpcj4a4mh3i19s41hfm-cuestacean-native-demand-policies.drv`.
Full recursive derivation inspection, exact three-file raw-source inspection and
retained paths/registration/roots exclude pinned Go CUE/source/full production
JSON/downstream executor identities. This is the construction cut, not the
separate Go-dependent verifier/consumer check closure.

The rebuilt artifact has these SHA-256 bytes (not seed inputs):

```text
demand-policy.json       d37e6537492c9ab43fdcc4631e564783951b9ff44dc0cd316000822629f62276
demand-policy-defer.json 873ab27094b14372adcce54685d13b0c2cf85d60aa6f86755cda68b0e419511a
```

All four original exporter/atom/family/full-JSON derivation identities remain
unchanged from merged #180, as do raw policy files, production JSON and generated
Rust. The exporter’s production fileset is still exactly eight files; the new
native artifact’s raw fileset is exactly three. Original atom/family graph proofs
retain 3392 build paths and are not widened. Separate verifier/test/seed derivation
identities legitimately change: the native artifact is an explicit consumer test
environment dependency, while the Go CUE comparison derivation is not a native
construction input. Compiler seed binaries do not embed the test policy JSON.

Decision: the two existing **experimental dispatch-policy export closures** are
closed natively under the current exporter profile. Dispatch semantics/kernel
ownership is unchanged; this is not closure of the six compiler policies or of
nested preference/default solving. #175 and its separate prototype remain held.
