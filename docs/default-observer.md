# Pinned CUE default observer

## Independent evidence, not another solver hypothesis

Follow [the failed contextual hypothesis](contextual-preference-spike.md).
Instead of guessing another readiness rule, expose pinned CUE's PUBLIC
`Value.Default` / `Value.Unify` observations in an explicitly Go-dependent,
test-only oracle. Milestone [#190](https://github.com/kolohelios/cuestacean/issues/190).
Two focused contract reviews preceded implementation, resolving exact protocol,
selected-error classification, reference-free AST admission and recursive shadowing.
No native algorithm, old assertion, policy, certificate, production source or
held #175 prototype changes. This does not make the Go oracle an independent
implementation of CUE CLI semantics: both use the SAME pinned CUE source, and
both remain independent of our native hypotheses.

Pin cue-lang/cue `6d609d768f1686f9a3a2a20197cacdbb70e5c79d` (v0.16.1), Nixpkgs
`ac588ef1c914865dad517b337ea0eb9694a4021c`. Preserve upstream source/license files;
our probe is independently written against public APIs, not copied evaluator code.
Use the pinned package's source/vendor closure, not a floating module download,
installed host Go/CUE or network resolution during builds.

## Trusted test protocol under review

New Go oracle sources under `bootstrap/oracles/default-observer/`. Build as an
additional command in the pinned CUE source tree using its module/dependencies.
No committed copy of the upstream go.mod/go.sum/vendor tree and no Cargo dependency.
The observer is NOT a native construction input or production executable.

CLI modes, each one fresh process and CUE context (nonempty selectors, exactly
256 UTF-8 bytes accepted, >256 rejected):

```text
cuestacean-default-observer inspect FILE SELECTOR
cuestacean-default-observer unify FILE LEFT RIGHT
cuestacean-default-observer default-unify FILE LEFT RIGHT
```

Read exactly one explicit raw CUE file (65536 bytes accepted, >65536 rejected),
with no import/module
loading via an ambient working directory. Parse/build this file with public
parser/cuecontext APIs. Selectors are bounded explicit CUE paths; missing or
invalid selectors fail setup. Reject imports explicitly; there is no ambient
loader. Parser errors fail setup. Do NOT reject `BuildFile`/root.Err globally:
an unrelated field can conflict while the selected field remains observable.
Only selected missing paths/invalid ParsePath fail setup; selected semantic Err
is an observed conflict. Unresolved source references are setup failures, not
semantic Conflict. To avoid diagnostic-string classification, this literal/type
observer rejects ALL source value references (including resolved field references)
at AST admission. Exact grammar: file/nested StructLit containers with ordinary
identifier/quoted-string field labels; expression BasicLit, predeclared identifiers
`_`, `int`, `bool`, `string`, `bytes`, `number`, `float`, parentheses, binary AND/OR,
and unary default-star/numeric +/- only. Reject every other expression/declaration
before BuildFile. To prevent predeclared names masking resolved references, reject
EVERY field whose decoded label is one of those seven names at EVERY struct depth,
quoted or unquoted. This is a deliberate oracle profile, not general CUE admission.
Tested semantic profile is scalar literal/type/
And/grouped marked-Or probes, not full CUE or untrusted wire-input hardening.
Invalid args/read/parse/path errors exit nonzero, stderr only, no partial JSON.
Do not conflate malformed setup with observed semantic Conflict.

- inspect: look up the value; observe its Default result.
- unify: look up two values and call LEFT.Unify(RIGHT), then observe Default.
- default-unify: select each operand's Default first, then Unify, then observe
  Default. This is an explicit counterfactual experiment, not an authorized
  evaluation strategy, fallback or implementation of source conjunction.

One deterministic JSON object per successful observation, trailing LF:

```text
{formatVersion:1, mode:"inspect"|"unify"|"default-unify",
 leftDefault:null|bool, rightDefault:null|bool,
 defaultSelected:bool, status:"concrete"|"incomplete"|"conflict", value:JSON|null}
```

Field spelling is `formatVersion`, NOT `version`. Emit EVERY field in exactly
the order shown, compact JSON with no additional fields/omissions and exactly
one final LF. Enum strings are lowercase as shown. `value` is null for incomplete
or conflict; concrete carries raw JSON (including null if the actual value is
null). Operand flags are explicit null outside default-unify, never omitted.
Complete inspect example (followed by LF):

```json
{"formatVersion":1,"mode":"inspect","leftDefault":null,"rightDefault":null,"defaultSelected":true,"status":"concrete","value":1}
```

Example shape for an incomplete unmarked union (followed by LF):

```json
{"formatVersion":1,"mode":"inspect","leftDefault":null,"rightDefault":null,"defaultSelected":false,"status":"incomplete","value":null}
```

Operand default flags are null unless explicitly selected in default-unify.
`defaultSelected` is EXACTLY the bool returned by the final public Default call;
it is NOT our Preference::{Absent,Bottom,Values} and must not be renamed to one
of those states. False cannot establish Absent versus conflicting preferences.
A nonconcrete selected default can still have defaultSelected=true. Record actual
API observations, never infer a flag from export success or expected answers.

Observe Default BEFORE marshaling. Report semantic status from the selected value:
definite Value.Err failure -> conflict; otherwise MarshalJSON failure -> incomplete;
success -> concrete with raw JSON, preserving exact i64 rather than float conversion.
Confirm these classification assumptions on the bounded profile during reviews.
MarshalJSON itself invokes Default; document this, not as an ordinary-constraint
projection. API values may be lazily normalized: use separate fresh processes for
mode/order comparisons, never hide a prior observation changing an operand.

## Fixture/evidence contract

Independently author raw CUE fields for left, right, inline conjunction and reverse
inline conjunction; no source conjunction through field references masquerading
as the inline expression. API Unify of independently built operands is a DIFFERENT
operation from source conjunction. Do not assert equality unless independently
pinned; preserve differences as observations, including order asymmetries.

Mandatory controls before broader observations:

- Simple *1|2, unmarked1|2, conflicting flat preferences, multiple defaults,
  abstract default *int|1, ordinary conflict, exact i64 extrema.
- Existing both-order failures and source-order discrepancies from #176/#186/#188.
- Left/right Default versus unify/default-unify versus fresh inline source exports;
  record full handwritten protocol expectations, not captured golden generation.
- Selected default can remain incomplete; selected=false is not a native
  preference-state classifier. No blanket API/CLI/native equality claim.
- Renamed files/fields/selectors, direct invocation with cleared environment and
  unusable PATH, missing provider/path failure, malformed args/source/selectors,
  byte/selector caps and deterministic repeat output. No ambient env mutation.

Tests use the mandatory absolute executable supplied as
`CUESTACEAN_DEFAULT_OBSERVER`; absent, relative, unreadable or unexecutable
providers fail setup. Execution/read errors are fatal. Never try
PATH, CUE CLI or a native reference after setup/semantic failure. Existing native
policy provider remains mandatory and unchanged. Native solvers never execute
or import this probe; only test orchestration does.

## Construction separation, ownership and validation

Add an explicit `default-observer` package/check. Existing nine checks remain;
run TEN actual checks with the new oracle check. Supply the test-only executable
to the devshell and downstream executor/check environment ONLY, not the base
rustPackage inherited by codegenSeed. Keep the observer executable/derivation out
of all six fresh compiler construction graphs; test-only inheritance must not
silently turn it into a seed dependency. Verify the seed's recursive derivation
graph separately. The observer's Go source/dependency closure is explicit, not a native/bootstrap
independence claim. Separate oracle outputs and handwritten expectations must
never seed production JSON, Rust generation, native producers or graph kernels.

All five existing constructor/export derivation identities and four native policy
hashes must remain unchanged. Keep exact eight-file exporter/six-file raw policy
cuts. Explicitly reject the observer derivation in native constructor full graph
verification alongside pinned CUE/source/full JSON/downstream executor. Inspect
recursive derivation/build paths, not only runtime refs. Verifier/consumer closure
MAY depend on Go CUE/observer and is not attested as Go-CUE-free.

Ownership after both reviews/explicit GO: tooling owner ONLY new Go oracle source
files and infra/tooling/flake.nix; evidence owner ONLY new executor integration
target `default_observer.rs`; coordinator docs, integration/final checks. Stable
merged-main workspaces, immutable commits, no coordinator-parent rebases; at most
two workers. No new private spike API/dead-code workarounds are needed.

Require focused debug/release × default/no-default plus strict focused Clippy,
Go oracle tests, both full Rust suites/default all-target Clippy debug/release,
pinned preflight and all ten actual Nix checks plus native/oracle builds/rebuilds.
Keep fresh compilers/features/drift/twelve builders/source/value evidence. No
raised guards, stack overrides, suppressions or production fallback. Linux x86_64
only; roughly <=1500 changed lines. Observation/evidence progress is not default
readiness, full CUE/default/bootstrap closure or a performance result; #175 stays held.

## Findings and validation

The observer records a strictly richer observation than export success alone:

- Unmarked `1|2` and conflicting flat preferences both have final Default=false
  and incomplete export. That flag cannot distinguish native Absent from Bottom.
- Multiple defaults and `*int|1` can have Default=true yet remain incomplete.
- The #176 genuine both-order failure remains incomplete in both orders while
  its public Default flags differ (false forward, true reverse). Do not collapse
  those flags into an order-independent inferred preference state.
- Selecting operand defaults before Unify can conflict where source conjunction
  exports a value, or concretize where direct Unify remains incomplete. It is an
  explicit counterfactual, not an approved native evaluation strategy.

Both contract reviews preceded implementation and settled exact wire formatting,
reference-free AST admission, all-depth decoded-label shadow rejection and
selected-value versus root error classification. Read-only implementation reviews
closed rejection of ordinary package clauses, then verified package/nested-selector
positives and unrelated-field conflicts. A fresh independently authored scratch
probe was regenerated in a unique directory before accepting review evidence;
no scratch corpus or captured output is a runtime expected-golden input.

Immutable inputs, both parented on merged `a8864faa`:

- Go/Nix `1d7169f3ba2584bbd6910f93ab1b0da699608015` (three owned files).
- Evidence `109c2643216789558feefada27b7f47590538f5c` (new integration target only).

All created workspaces were retired after integration. Six new plus 62 retained
Rust groups pass debug/release × default/no-default with strict focused Clippy.
The new table pins 24 cases × eight API observations (192 complete handwritten
wire expectations), each repeated in a fresh process, plus separate CLI exports
and setup/AST/provider/cap tests. Both full Rust suites/default all-target Clippy
pass debug/release. Scoped Go tests pass in the sandbox-built observer package.
Pinned preflight passes 13/13 (7 inapplicable skips). All TEN checks and six native/
export/oracle packages were explicitly built and sandbox-rebuilt; six fresh
compilers, profiles, twelve builders, drift and source/value oracles remain intact.
Linux x86_64 only; no new semantic/readiness approval.

Observer constructor `/nix/store/kn42sxf8kjmm4hkndkc0sxshd9iqjg93-cuestacean-default-observer.drv`
produces `/nix/store/zhcg4i3bdl7x4s86jkhmlayd0xh4vzd4-cuestacean-default-observer`.
Its source is pinned CUE `/nix/store/f2wbig2h7zi6q39ynn7sakvp50m4j3i5-source`,
with vendor `/nix/store/fnclsqmhvgxbj52zdf4rylf62lfs12bd-cue-0.16.1-go-modules`;
builds disable CGO, proxies, sumdb and toolchain downloads. Upstream source and
licenses remain unmodified; only the additional command is copied into build space.

All five old constructor/export identities, four policy hashes and exact eight/
six source cuts match #188. Fresh native-policy construction proof retains 1407
derivations/3392 full build paths and excludes observer/CUE/source/full JSON/current
executor. Separately the compiler seed root has 1436 derivations/3466 full build
paths: observer executable AND derivation are absent, and its environment has no
provider variable. Independent `--requisites --include-outputs` queries match
both retained path sets. This is NOT Go-CUE-free seed construction or a minimal
seed-source attestation; the existing Go-CUE export dependency remains. Oracle/
verifier/consumer closures are deliberately separate from the attested native cuts.

Reproduce with:

```sh
nix build --no-link ./infra/tooling#default-observer
nix build --no-link ./infra/tooling#checks.x86_64-linux.default-observer
nix develop ./infra/tooling --command cargo test \
  --manifest-path bootstrap/executor/Cargo.toml --test default_observer
```
