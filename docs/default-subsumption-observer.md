# Default-sensitive subsumption observations

## Reviewed research contract

Follow [the public default observer](default-observer.md). Investigate another
public observation before designing a new native calculus: can `Value.Subsume`
distinguish incomplete values whose Default flags/export categories coincide?
Milestone [#192](https://github.com/kolohelios/cuestacean/issues/192). Two focused
contract/evidence reviews preceded implementation and approved observation only.
No native solver,
readiness certificate, production source, policy or held #175 prototype changes.

Pinned public API `v.Subsume(w)` reports nil when w is an instance of v. Its
`cue.Raw()` option requests low-level subsumption taking defaults into account.
Without Raw the implementation can select defaults on the subsumed operand.
Both operands MUST come from the SAME build/context. Do not use `.Default()` or
`.MarshalJSON()` before comparison, or assume Raw means ignoring defaults merely
because the similarly named Syntax option suppresses simplification.

A nil API result is an observed acceptance, not a general semantic proof. A nonnil
result can include unsupported/inexact comparisons: do NOT label it definitive
mathematical non-subsumption or turn it into native Conflict. This is a bounded
public-API study, not a new logical decision procedure or CUE compatibility claim.
CUE API and CLI share the same pinned source; they are not independent upstream
implementations. Keep every old independent observation/assertion unchanged.

## Protocol extension (existing v1 modes unchanged)

Extend the existing Go observer with two modes:

```text
cuestacean-default-observer subsume FILE SUPER SUB
cuestacean-default-observer raw-subsume FILE SUPER SUB
```

Reuse the identical reference-free source/selector/setup profile, byte limits,
shadow exclusions, no imports/ambient loader, package handling and absolute
provider from #190. Scalar lookup/existence checks precede the API call. No
Default/MarshalJSON projection or global root.Err rejection. Inspect/unify/
default-unify retain EXACT v1 wire bytes and behavior, including all null fields.

For new modes emit exactly three ordered fields, compact JSON plus one LF:

```json
{"formatVersion":2,"mode":"raw-subsume","accepted":true}
```

`mode` is exactly subsume or raw-subsume, `accepted` is exactly whether the public
call returned nil. No omitted/null/error/status/value fields; no diagnostic text
in successful observation JSON. Rejection is still a successful observation
(exit zero, empty stderr, accepted=false), NOT a setup failure. Invalid args,
paths, AST admission, read or provider errors stay fatal with no JSON; no fallback.
Full ordinary-mode rejection and acceptance examples (each followed by LF):

```json
{"formatVersion":2,"mode":"subsume","accepted":false}
{"formatVersion":2,"mode":"subsume","accepted":true}
```

Raw-mode rejection is exactly:

```json
{"formatVersion":2,"mode":"raw-subsume","accepted":false}
```

Never add v1 observation fields to these v2 results.

Each mode/order uses a fresh process/context and the same two source operands
within that build. Source fields are independently authored literal/type/And/Or
expressions, never field-reference conjunctions or inputs derived from native
reports. Do not copy raw observation dumps into test goldens or compare a probe
with outputs it generated as the only evidence.

## Independent evidence

Use literal handwritten public-API expectations for BOTH directions and BOTH
profiles, separately from fresh v1 inspections. Required probes:

- Plain unions versus marked defaults with the same ordinary alternatives.
- Flat conflicting defaults versus unmarked alternatives: same false/incomplete
  Default observation does not establish semantic equality.
- A forced concrete value with/eliminated default versus its plain literal form.
- Multiple defaults, abstract `*int|1`, plain int/top, scalar exact i64 extrema,
  concrete false/null and ordinary conflict.
- Known #176/#186/#188 genuine failures and separately pinned asymmetric source
  forms. Preserve relation observations individually; no operand-order imitation.
- Reflexive/obvious unmarked scalar inclusion controls. Mutual API acceptance
  is an observation, NOT a confluence/equivalence certificate for arbitrary CUE.
- Renamed files/selectors, repeat exact bytes with cleared env/unusable PATH,
  new-mode wrong arity and all selector/provider/cap rejection boundaries.
- Reject unrelated source references/shadows/imports even if a requested relation
  would otherwise be meaningful; the observer is intentionally not general CUE.

Existing v1 192 handwritten observations, setup tests and all native regressions
must remain unchanged. Add relation evidence additively in the existing Rust
target rather than deleting/replacing old expectations. No new Cargo dependency.
Use existing mandatory CUESTACEAN_DEFAULT_OBSERVER only, never PATH/CLI/native
fallback. The old CLI comparison remains an explicitly separate mandatory oracle.

## Ownership and construction independence

After BOTH reviews and explicit GO: implementation owner ONLY existing Go
main.go/main_test.go under bootstrap/oracles/default-observer; evidence owner ONLY
bootstrap/executor/tests/default_observer.rs; coordinator docs/integration/validation.
Stable merged main workspaces; integrate exact immutable commits, never rebase
workers onto mutable coordinator changes. At most two workers; no suppression.

Observer constructor/output legitimately change with the added public API call.
All FIVE native/export constructor identities, four policy hashes and exact
8/6-file cuts remain unchanged. Provider stays exclusively in downstream
executor/devshell, not base rustPackage or compiler seed. Existing tenth check
must re-attest its NEW observer .drv exclusion in full seed construction graph;
all native graph verifiers likewise exclude the new oracle. Preserve source/vendor
pin and offline Go build, fresh six compilers/features/drift/twelve builders and
independent source/value evidence. No Go-CUE-free seed/source-minimality claim.

Require focused new/old tests debug/release x default/no-default with strict
Clippy, scoped Go tests, both full Rust suites/default all-target Clippy in both
profiles, pinned preflight, TEN actual sandbox checks and six native/export/oracle
package builds/rebuilds. Inspect recursive derivation/build graphs, not just
runtime refs. Pinned CUE source 6d609d768f1686f9a3a2a20197cacdbb70e5c79d (v0.16.1),
Nixpkgs ac588ef1c914865dad517b337ea0eb9694a4021c; Linux x86_64 only. Stay near
1500 changed lines. No raised guards, native/default/bootstrap/performance or
readiness claim, regardless of how many sample relations agree.

## Findings and interpretation

Independent pinned observations show why API profiles must stay separate:

- For A=`1|2`, B=`*1|2`, ordinary API accepts both directions; Raw accepts
  B.Subsume(A) but rejects A.Subsume(B). Do not infer relation direction from
  export success, the Default flag, or a plain-set subset intuition.
- Opposite marked preferences have ordinary acceptance both ways and Raw
  rejection both ways. Rejection still means only nonnil API result.
- Flat conflicting preferences and an unmarked union have the same false/
  incomplete v1 observation AND mutual acceptance under both relation profiles.
  Even Raw does not supply a native Absent/Bottom classifier or equality proof.
- Forced/eliminated-default source forms and their plain literals can mutually
  pass both profiles. That does not authorize erasing source recipe identities.
- The #176 genuine forward/reverse source states both remain incomplete, yet
  their relation observations are ordinary true/true and Raw false/true.
  Other source asymmetries likewise require individually pinned directional data.

These are public API observations of the pinned implementation, not a formal
pair-subsumption calculus, a normalization certificate or a corrected lifetime
algorithm. Existing native results, conservative gates and #175 hold remain
unchanged. No relation result is fed back into a native solve or production build.

## Review and validation results

Two focused reviews preceded implementation, using independently authored raw
source and public-API probes against the pinned source/vendor closure. They
confirmed same-build operands, Raw's default-sensitive profile, exact three-field
wire forms and the limitation of nonnil results. Cross-review verified operation
order before Default/MarshalJSON, direction-specific source cases and unchanged
v1 behavior. No blocking findings remained. All workspaces were retired.

Immutable inputs, both parented on merged `b4ae7848`:

- Go `3df8871bee043d4447f2268cf55493c1de0c0dbb` (two existing source files).
- Evidence `b20c82a798722c349371fc94f940e9dfe40f011d` (334 additive Rust lines).

The original 599 Rust evidence lines are byte-identical; all 192 old v1 expected
observations remain intact. Eighteen controls plus twelve independently authored
forward/reverse source-state pairs add 120 directional/profile relation
expectations, each repeated in a fresh process. Additional ambiguity/renaming,
setup/provider and inclusive-cap tests keep observation and admission separate.

All fourteen observer tests and strict focused Clippy pass debug/release ×
default/no-default. Together with 62 retained groups, all 76 focused groups pass
in all four combinations. Both complete Rust suites/default all-target Clippy
pass debug/release; scoped Go tests pass in the sandbox package. Pinned preflight
passes 13/13 (7 inapplicable skips); all TEN checks and six native/export/oracle
packages were actually built and sandbox-rebuilt. Fresh six compilers, profiles,
twelve builders, drift and independent source/value evidence remain intact.
Linux x86_64 only; #175 remains held.

New observer constructor:
`/nix/store/303m6ck2yx0dx5p2ljs1pxypxyds5nzn-cuestacean-default-observer.drv`.
Output: `/nix/store/kziagf7r5z1vzjymrxsfwsqm5h1qimzl-cuestacean-default-observer`.
Pinned source/vendor/offline Go construction are unchanged from #190. No Nix
source change or provider-placement change was required.

The five native/export constructor identities, four policy hashes and exact
8/6-file cuts are unchanged. Fresh native construction proof: 1407 derivations /
3392 full build paths, excluding current observer/CUE/source/full JSON/executor.
Fresh seed proof: 1436 derivations / 3466 paths, excluding the NEW observer
executable/derivation with no provider environment. Independent full
`--requisites --include-outputs` queries match retained verifier path sets.
These are not Go-CUE-free seed or minimal seed-source claims; the oracle/check/
consumer closures remain intentionally separate from the attested native cuts.

Reproduce the relation controls with:

```sh
nix develop ./infra/tooling --command cargo test \
  --manifest-path bootstrap/executor/Cargo.toml --test default_observer \
  handwritten_relation_controls_both_directions_and_profiles
```
