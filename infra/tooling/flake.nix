{
  description = "Cuestacean development and validation tools";

  inputs = {
    kolohelios-nix.url = "https://flakehub.com/f/kolohelios/kolohelios-nix/*.tar.gz";
    nixpkgs.follows = "kolohelios-nix/nixpkgs";
    shaka.url = "https://flakehub.com/f/kolohelios/shaka/*.tar.gz";
  };

  outputs =
    {
      kolohelios-nix,
      shaka,
      ...
    }:
    let
      scannerFixtures = [
        "selector-open"
        "slice-separator"
        "selector-identifier"
        "record-open"
        "conjunction-open"
        "record-close"
        "record-label"
        "record-colon"
        "record-tail"
        "tests/valid/bound-inclusive-only"
        "index-close"
        "list-open"
        "list-close"
        "list-tail"
        "call-open"
        "whitespace"
        "after-operand"
        "after-word-operator"
        "group-open"
        "group-close"
        "negation-prefix"
        "not-prefix"
        "bound-prefix"
        "infix"
        "decimal"
        "integer-literal"
        "identifier"
        "string-literal"
        "tests/valid/string-no-unicode"
        "tests/valid/string-no-wide"
        "tests/valid/string-nested"
        "tests/valid/scanner-predicates"
        "tests/valid/scanner-repeat"
        "tests/valid/repeat-scopes"
        "tests/valid/repeat-zero"
        "tests/valid/repeat-fuel"
        "tests/valid/repeat-fuel-over"
        "tests/valid/string-unrolled"
        "tests/valid/scanner-horizontal"
        "tests/valid/scanner-stalled"
        "tests/valid/scanner-advance"
        "tests/valid/scanner-decimal-permissive"
        "tests/valid/scanner-reject"
        "tests/valid/scanner-state"
        "tests/valid/scanner-lookahead"
        "tests/valid/scanner-char-break"
      ];
      scannerNames = builtins.concatStringsSep " " scannerFixtures;
      builderFixtures = [
        "ast-program"
        "ast-atom"
        "ast-integer"
        "ast-text"
        "ast-binary"
        "ast-ternary"
        "ast-unary"
        "ast-record-field"
        "tests/valid/ast-program-v2"
        "tests/valid/ast-escaped"
        "tests/valid/ast-duplicate"
      ];
      builderNames = builtins.concatStringsSep " " builderFixtures;
      builderExportFixtures = builderFixtures ++ [ "tests/valid/ast-literal" ];
      builderExportNames = builtins.concatStringsSep " " builderExportFixtures;
      prattFixtures = [
        "slice-control"
        "tests/valid/slice-control-lower-one"
        "tests/valid/slice-control-tail"
        "conjunction-control"
        "tests/valid/conjunction-control-reverse"
        "tests/valid/conjunction-control-tail"
        "list-control"
        "tests/valid/list-control-homogeneous"
        "record-control"
        "tests/valid/record-control-homogeneous"
        "tests/valid/record-control-reverse"
        "tests/valid/record-control-scalars"
        "tests/valid/list-control-reverse"
        "tests/valid/list-control-scalars"
        "tests/valid/list-control-stalled"
        "tests/valid/list-control-unfinished"
        "primary"
        "record-primary"
        "conjunction-primary"
        "tests/valid/conjunction-primary-high-minimum"
        "tests/valid/primary-no-conjunction"
        "tests/valid/record-primary-high-minimum"
        "tests/valid/primary-no-records"
        "tests/valid/primary-nested"
        "tests/valid/primary-return"
        "tests/valid/primary-return-empty"
        "tests/valid/pratt-return-loop"
        "tests/valid/primary-bound-high-minimum"
        "tests/valid/primary-no-calls"
        "tests/valid/primary-call-high-minimum"
        "tests/valid/primary-no-lists"
        "tests/valid/primary-no-atoms"
        "tests/valid/primary-high-minimum"
        "tests/valid/primary-low-negation"
        "tests/valid/primary-cross-phase"
        "pratt"
        "tests/valid/pratt-no-selectors"
        "tests/valid/pratt-no-slices"
        "tests/valid/pratt-slice-high-minimum"
        "tests/valid/pratt-no-index"
        "tests/valid/pratt-index-high-minimum"
        "tests/valid/pratt-right"
        "tests/valid/pratt-stalled"
        "tests/valid/pratt-invalid-state"
        "tests/valid/pratt-overflow"
      ];
      prattNames = builtins.concatStringsSep " " prattFixtures;
      unifyFixtures = [
        "list-slice"
        "tests/valid/list-slice-identity"
        "index"
        "tests/valid/index-zero"
        "record-index"
        "record-length"
        "tests/valid/record-index-zero"
        "lower-exclusive-bound"
        "upper-exclusive-bound"
        "tests/valid/exclusive-reversed"
        "list-index"
        "tests/valid/list-index-zero"
        "length-control"
        "list-length-control"
        "tests/valid/list-length-zero"
        "string-length-control"
        "tests/valid/length-zero"
        "unify"
        "sum"
        "equal"
        "not-equal"
        "less"
        "less-equal"
        "greater"
        "greater-equal"
        "logical-and"
        "logical-or"
        "logical-not"
        "json-output"
        "integer-output"
        "tests/valid/output-zero"
        "lower-bound"
        "upper-bound"
        "tests/valid/bound-reversed"
        "tests/valid/unary-bound-endpoints"
        "negate-control"
        "difference-control"
        "product-control"
        "divide-control"
        "modulo-control"
        "quotient-control"
        "remainder-control"
        "tests/valid/not-true"
        "tests/valid/control-default"
        "tests/valid/unary-conditions"
        "tests/valid/integer-literals"
        "tests/valid/logic-false"
        "tests/valid/less-false"
        "tests/valid/equal-false"
        "tests/valid/sum-right"
        "tests/valid/unify-right"
        "tests/valid/unify-scoped"
      ];
      unifyNames = builtins.concatStringsSep " " unifyFixtures;
      loweringFixtures = [
        "lowering"
        "pratt-lowering"
        "unify-lowering"
        "ast-lowering"
        "scan-lowering"
        "meta-lowering"
        "tests/valid/lowering-bindings"
        "tests/valid/lowering-identifier"
        "tests/valid/meta-driver-keys"
        "tests/valid/lowering-numbers"
        "tests/valid/lowering-select"
        "tests/valid/lowering-left-twice"
        "tests/valid/lowering-escaped"
        "tests/valid/lowering-reverse"
        "tests/valid/lowering-module-escaped"
        "tests/valid/lowering-collections"
      ];
      loweringNames = builtins.concatStringsSep " " loweringFixtures;
      exportFixtures = [
        "literal"
        "addition"
        "subtraction"
        "negation"
        "multiplication"
        "division"
        "unification"
        "int-constraint"
        "bounds"
        "exclusive-bounds"
        "booleans"
        "strings"
        "string-length"
        "lists"
        "list-length"
        "list-expressions"
        "conjunction"
        "conjunction-conflict"
        "record-expressions"
        "mixed-collections"
        "mixed-unify"
        "mixed-conflict"
        "nested-records"
        "nested-records-unify"
        "nested-records-conflict"
        "nested-records-scalar-conflict"
        "nested-lists"
        "records"
        "records-unify"
        "records-index"
        "records-length"
        "records-equal"
        "records-logical"
        "records-conflict"
        "records-fields-conflict"
        "list-indexing"
        "list-slicing"
        "optional-slices"
        "optional-slices-domain"
        "optional-slices-range"
        "list-slicing-range"
        "list-slicing-domain"
        "collection-index"
        "selectors"
        "collection-index-record"
        "tests/valid/lists-unify"
        "tests/valid/lists-capacity"
        "tests/valid/lists-equal"
        "concatenation"
        "equality"
        "inequality"
        "ordering"
        "ordering-text"
        "logic-and"
        "logic-or"
        "not"
        "null"
        "top"
        "top-null"
        "tests/valid/equality-mixed"
        "tests/valid/strings-empty"
        "tests/valid/strings-equal"
        "tests/valid/strings-incomplete"
        "tests/valid/strings-conflict"
        "tests/valid/strings-mixed"
        "evaluation"
        "grammar"
        "tests/valid/grammar-no-brackets"
        "tests/valid/evaluation-left-twice"
        "tests/valid/pratt-lowering-no-increment"
        "tests/valid/unify-lowering-permissive"
        "tests/valid/integer-kernel-alternate"
        "tests/valid/division-kernel-alternate"
        "tests/valid/unify-equality-probe"
        "tests/valid/ast-lowering-forced"
        "tests/valid/scan-lowering-unicode"
        "tests/valid/meta-lowering-forced"
        "tests/valid/meta-driver-previous"
        "tests/valid/ast-literal"
      ]
      ++ scannerFixtures
      ++ builderFixtures
      ++ prattFixtures
      ++ unifyFixtures
      ++ loweringFixtures;
      fixtureNames = builtins.concatStringsSep " " exportFixtures;
      jsonExports =
        pkgs:
        let
          cueSources = pkgs.lib.fileset.toSource {
            root = ../../bootstrap;
            fileset = pkgs.lib.fileset.unions (
              [
                ../../bootstrap/schema.cue
                ../../bootstrap/scanner-schema.cue
                ../../bootstrap/ast-schema.cue
                ../../bootstrap/scanner-fragments.cue
                ../../bootstrap/decimal-fragments.cue
                ../../bootstrap/primary-fragments.cue
                ../../bootstrap/pratt-fragments.cue
                ../../bootstrap/list-control-fragments.cue
                ../../bootstrap/record-control-fragments.cue
                ../../bootstrap/conjunction-control-fragments.cue
                ../../bootstrap/slice-control-fragments.cue
                ../../bootstrap/record-primary-fragments.cue
                ../../bootstrap/conjunction-primary-fragments.cue
                ../../bootstrap/string-literal-fragments.cue
                ../../bootstrap/equality-fragments.cue
                ../../bootstrap/ordering-fragments.cue
                ../../bootstrap/logic-fragments.cue
                ../../bootstrap/integer-control-fragments.cue
                ../../bootstrap/division-control-fragments.cue
                ../../bootstrap/output-control-fragments.cue
                ../../bootstrap/bound-control-fragments.cue
                ../../bootstrap/exclusive-bound-fragments.cue
                ../../bootstrap/lowering-fragments.cue
                ../../bootstrap/pratt-lowering-fragments.cue
                ../../bootstrap/unify-lowering-fragments.cue
                ../../bootstrap/ast-lowering-fragments.cue
                ../../bootstrap/ast-lowering-body.cue
                ../../bootstrap/ast-integer-policy.cue
                ../../bootstrap/scan-lowering-fragments.cue
                ../../bootstrap/meta-lowering-fragments.cue
                ../../bootstrap/tests/invalid
              ]
              ++ map (fixture: ../../bootstrap + "/${fixture}.cue") exportFixtures
            );
          };
        in
        pkgs.runCommand "cuestacean-bootstrap-json"
          {
            nativeBuildInputs = [
              pkgs.cue
              pkgs.jq
            ];
          }
          ''
            set -euo pipefail
            export LC_ALL=C TZ=UTC
            for fixture in ${fixtureNames}; do
              echo "exporting $fixture"
              case "$fixture" in
                ast-lowering)
                  mkdir -p "$out"
                  cp ${nativeAstLoweringPolicy pkgs}/ast-lowering.json "$out/ast-lowering.json"
                  continue
                  ;;
                ${builtins.concatStringsSep "|" builderExportFixtures})
                  mkdir -p "$(dirname "$out/$fixture.json")"
                  cp ${nativeBuilders pkgs}/"$fixture.json" "$out/$fixture.json"
                  continue
                  ;;
              esac
              # Load only the fixture's definition closure. Loading every compiler
              # for a literal can exhaust the CI runner while checking unused CUE.
              schema=schema
              case "$fixture" in
                ${builtins.concatStringsSep "|" scannerFixtures}) schema=scanner-schema ;;
              esac
              fragments=()
              case "$fixture" in
                pratt|tests/valid/pratt-no-slices|tests/valid/pratt-slice-high-minimum|tests/valid/pratt-no-selectors|tests/valid/pratt-no-index|tests/valid/pratt-index-high-minimum|tests/valid/pratt-right) fragments=(pratt-fragments) ;;
                list-control|tests/valid/list-control-reverse|tests/valid/list-control-scalars|tests/valid/list-control-homogeneous) fragments=(list-control-fragments) ;;
                slice-control|tests/valid/slice-control-lower-one|tests/valid/slice-control-tail) fragments=(slice-control-fragments) ;;
                conjunction-control|tests/valid/conjunction-control-reverse|tests/valid/conjunction-control-tail) fragments=(conjunction-control-fragments) ;;
                record-control|tests/valid/record-control-reverse|tests/valid/record-control-scalars|tests/valid/record-control-homogeneous) fragments=(record-control-fragments) ;;
                conjunction-primary|tests/valid/conjunction-primary-high-minimum) fragments=(conjunction-primary-fragments) ;;
                record-primary|tests/valid/record-primary-high-minimum) fragments=(record-primary-fragments) ;;
                primary|tests/valid/primary-no-conjunction|tests/valid/primary-bound-high-minimum|tests/valid/primary-call-high-minimum|tests/valid/primary-no-lists|tests/valid/primary-no-records) fragments=(primary-fragments) ;;
                decimal|integer-literal) fragments=(decimal-fragments) ;;
                string-literal|tests/valid/string-no-unicode|tests/valid/string-no-wide|tests/valid/string-nested|tests/valid/string-unrolled) fragments=(string-literal-fragments) ;;
                whitespace|after-operand|after-word-operator|group-close|call-open|index-close)
                  fragments=(scanner-fragments) ;;
                lowering|tests/valid/lowering-left-twice|tests/valid/lowering-reverse)
                  fragments=(lowering-fragments) ;;
                pratt-lowering|tests/valid/pratt-lowering-*) fragments=(pratt-lowering-fragments) ;;
                unify-lowering|tests/valid/unify-lowering-*|tests/valid/integer-kernel-alternate|tests/valid/division-kernel-alternate) fragments=(unify-lowering-fragments) ;;
                ast-lowering|tests/valid/ast-lowering-*) fragments=(ast-lowering-fragments) ;;
                scan-lowering|tests/valid/scan-lowering-*) fragments=(scan-lowering-fragments) ;;
                meta-lowering|tests/valid/meta-lowering-*|tests/valid/meta-driver-previous)
                  fragments=(meta-lowering-fragments) ;;
                equal|not-equal) fragments=(equality-fragments) ;;
                less|less-equal|greater|greater-equal) fragments=(ordering-fragments) ;;
                logical-and|logical-or) fragments=(equality-fragments logic-fragments) ;;
                json-output|integer-output) fragments=(output-control-fragments) ;;
                lower-exclusive-bound|upper-exclusive-bound|tests/valid/exclusive-reversed) fragments=(exclusive-bound-fragments) ;;
                lower-bound|upper-bound|tests/valid/bound-reversed) fragments=(bound-control-fragments) ;;
                difference-control|product-control) fragments=(integer-control-fragments) ;;
                divide-control|modulo-control|quotient-control|remainder-control) fragments=(division-control-fragments) ;;
              esac
              inputs=("${cueSources}/$schema.cue")
              for fragment in "''${fragments[@]}"; do inputs+=("${cueSources}/$fragment.cue"); done
              inputs+=("${cueSources}/$fixture.cue")
              # Choose the provider before execution: overrides need the raw
              # preference, never the already materialized default artifact.
              case "$fixture" in
                tests/valid/ast-lowering-*)
                  inputs+=("${cueSources}/ast-lowering-body.cue" "${cueSources}/ast-integer-policy.cue") ;;

              esac
              cue export "''${inputs[@]}" --out json | jq -S . > first.json
              cue export "''${inputs[@]}" --out json | jq -S . > second.json
              cmp first.json second.json
              mkdir -p "$(dirname "$out/$fixture.json")"
              cp first.json "$out/$fixture.json"
            done
            for fixture in ${cueSources}/tests/invalid/*.cue; do
              if cue export ${cueSources}/schema.cue ${cueSources}/scanner-schema.cue ${cueSources}/ast-schema.cue "$fixture" --out json > invalid.log 2>&1; then
                echo "Expected rejection: $fixture" >&2
                exit 1
              fi
              if grep -Eq 'reference .*not found|undefined' invalid.log; then
                echo "Missing definition in invalid fixture: $fixture" >&2
                exit 1
              fi
            done
          '';

      rustPackage =
        pkgs:
        pkgs.rustPlatform.buildRustPackage {
          pname = "bootstrap-executor";
          meta.mainProgram = "bootstrap-executor";
          version = "0.1.0";
          src = ../../bootstrap;
          postUnpack = ''
            sourceRoot="$sourceRoot/executor"
          '';
          cargoLock.lockFile = ../../bootstrap/executor/Cargo.lock;
          # Test-only provider; seed overrides inherit the native artifact, not its
          # Go-CUE-dependent verifier. No policy JSON is embedded in seed binaries.
          CUESTACEAN_DEMAND_POLICIES = nativeDemandPolicies pkgs;
          nativeCheckInputs = [
            pkgs.rustfmt
            pkgs.clippy
            # Architecture tests run mandatory pinned upstream probes.
            pkgs.cue
          ];
          preCheck = ''
            cargo fmt --check
            cargo clippy --offline --all-targets -- -D warnings
            (
              mkdir lowering-profile-save
              mv ../generated/lowering*.rs ../generated/pratt-lowering.rs ../generated/unify-lowering.rs ../generated/ast-lowering.rs ../generated/scan-lowering.rs ../generated/meta-lowering.rs ../generated/meta-driver-*.rs lowering-profile-save/
              trap 'mv lowering-profile-save/*.rs ../generated/; rmdir lowering-profile-save' EXIT
              cargo check --offline --no-default-features --features semantics --bins
            )
            cargo check --offline --no-default-features --features compiled-lowering --bins
            cargo test --offline --no-default-features --lib
            cargo test --offline --no-default-features --test solver_spike
            cargo test --offline --no-default-features --test default_provenance
            cargo test --offline --no-default-features --test default_readiness
            cargo test --offline --no-default-features --test demand_policy
            cargo test --offline --no-default-features --test preference_projection
            cargo test --offline --no-default-features --test transparent_preferences
            cargo test --offline --no-default-features --test context_preferences
          '';
        };

      # Test-only command built inside the exact pinned CUE module/vendor tree.
      # Do not put this provider in rustPackage: codegenSeed inherits that base.
      defaultObserver =
        pkgs:
        pkgs.runCommand "cuestacean-default-observer"
          {
            nativeBuildInputs = [ pkgs.go ];
            meta.mainProgram = "cuestacean-default-observer";
          }
          ''
            cp -R ${pkgs.cue.src} source
            chmod -R u+w source
            cd source
            ln -s ${pkgs.cue.goModules} vendor
            mkdir -p cmd/cuestacean-default-observer
            cp ${../../bootstrap/oracles/default-observer/main.go} cmd/cuestacean-default-observer/main.go
            cp ${../../bootstrap/oracles/default-observer/main_test.go} cmd/cuestacean-default-observer/main_test.go
            export CGO_ENABLED=0 GOTOOLCHAIN=local GOPROXY=off GOSUMDB=off
            export HOME=$TMPDIR GOCACHE=$TMPDIR/go-cache
            go test -mod=vendor ./cmd/cuestacean-default-observer
            mkdir -p "$out/bin"
            go build -mod=vendor -trimpath -o "$out/bin/cuestacean-default-observer" ./cmd/cuestacean-default-observer
          '';
      # Separate instrumented oracle: the pinned evaluator is changed only by
      # ten diagnostic-only hook lines; old modes stay byte-stable. No seed wiring.
      defaultTrace =
        pkgs:
        pkgs.runCommand "cuestacean-default-trace"
          {
            nativeBuildInputs = [
              pkgs.go
              pkgs.perl
            ];
            meta.mainProgram = "cuestacean-default-trace";
          }
          ''
            cp -R ${pkgs.cue.src} source
            chmod -R u+w source
            cd source
            ln -s ${pkgs.cue.goModules} vendor
            mkdir -p cmd/cuestacean-default-trace "$out/bin"
            cp ${../../bootstrap/oracles/default-trace/adt.go} internal/core/adt/cuestacean_trace.go
            cp ${../../bootstrap/oracles/default-trace/adt_test.go} internal/core/adt/cuestacean_trace_test.go
            cp ${../../bootstrap/oracles/default-trace/main.go} cmd/cuestacean-default-trace/main.go
            cp ${../../bootstrap/oracles/default-trace/main_test.go} cmd/cuestacean-default-trace/main_test.go
            perl -0pi -e '
              my $n = s~(\Qfunc (n *nodeContext) crossProduct(dst, cross []*nodeContext, dn *envDisjunct, mode runMode) []*nodeContext {\E\n)~$1\ttraceID := cuestaceanTraceEnter()\n~g;
              die "trace entry hook drift" unless $n == 1;
              $n = s~(\ttraceID := cuestaceanTraceEnter\(\)\n)~$1\tif cuestaceanTraceEqualityEnabled() { cuestaceanTraceRunMode(traceID, int(mode)) }\n~g;
              die "trace run mode hook drift" unless $n == 1;
              $n = s~(\t\t\ttmp = append\(tmp, r\)\n)~\t\t\tcuestaceanTraceOrigin(traceID, p, int(priority))\n$1~g;
              die "trace origin hook drift" unless $n == 1;
              $n = s~(\thasNonMaybe := false\n)~\tcuestaceanTraceTrials(traceID, tmp, leftDropsDefault, rightDropsDefault)\n$1~g;
              die "trace trials hook drift" unless $n == 1;
              $n = s~(\treturn dst\n\}\n\nfunc combineDefault2)~\tcuestaceanTraceResults(traceID, dst)\n$1~g;
              die "trace results hook drift" unless $n == 1;
              $n = s~(\tnv := x.node.DerefValue\(\)\n)~\ttraceAppend := cuestaceanTraceAppend(x)\n$1~g;
              die "trace append entry hook drift" unless $n == 1;
              $n = s~(\t\tif xn == x \{\n)~$1\t\t\tcuestaceanTraceIdentity(traceAppend, xn)\n~g;
              die "trace identity hook drift" unless $n == 1;
              $n = s~(\t\txv := xn.node.DerefValue\(\)\n)~$1\t\tif cuestaceanTraceEqualityEnabled() { cuestaceanTraceCompare(traceAppend, xn, xv.status != finalized || nv.status != finalized) }\n~g;
              die "trace comparison hook drift" unless $n == 1;
              $n = s~(\t\t// free vertex\n)~\t\tcuestaceanTraceAccept(traceAppend)\n$1~g;
              die "trace accept hook drift" unless $n == 1;
              $n = s~(\treturn append\(a, x\)\n)~\tcuestaceanTraceInsert(traceAppend, len(a))\n$1~g;
              die "trace insert hook drift" unless $n == 1;
            ' internal/core/adt/disjunct2.go
            # Do not gofmt the injected upstream file: guards are single lines.
            # Removing exactly ten diagnostic lines must restore pinned bytes.
            perl -ne 'if (/^\s*(?:traceID := cuestaceanTraceEnter\(\)|traceAppend := cuestaceanTraceAppend\(x\)|cuestaceanTrace(?:Origin|Trials|Results|Identity|Accept|Insert)\(.*\)|if cuestaceanTraceEqualityEnabled\(\) \{ cuestaceanTrace(?:RunMode|Compare)\(.*\) \})\n$/) { $n++ } else { print } END { die "trace restoration hook count" unless $n == 10 }' \
              internal/core/adt/disjunct2.go > restored.go
            cmp restored.go ${pkgs.cue.src}/internal/core/adt/disjunct2.go
            diff -u --label pinned/disjunct2.go --label instrumented/disjunct2.go ${pkgs.cue.src}/internal/core/adt/disjunct2.go internal/core/adt/disjunct2.go > "$out/instrumentation.diff" || test "$?" = 1
            export CGO_ENABLED=0 GOTOOLCHAIN=local GOPROXY=off GOSUMDB=off
            export HOME=$TMPDIR GOCACHE=$TMPDIR/go-cache
            go test -mod=vendor ./internal/core/adt -run '^TestCuestacean'
            go test -mod=vendor ./cmd/cuestacean-default-trace
            go build -mod=vendor -trimpath -o "$out/bin/cuestacean-default-trace" ./cmd/cuestacean-default-trace
          '';
      defaultObserverVerification =
        pkgs:
        let
          # This one seed builds all six fresh compilers. .drv roots include
          # recursive build-time dependencies, unlike runtime output references.
          seedClosure = pkgs.closureInfo { rootPaths = [ (codegenSeed pkgs).drvPath ]; };
        in
        pkgs.runCommand "cuestacean-default-observer-verification" { nativeBuildInputs = [ pkgs.jq ]; } ''
          for forbidden in ${(defaultObserver pkgs).drvPath} ${(defaultTrace pkgs).drvPath}; do
            if grep -Fx "$forbidden" ${seedClosure}/store-paths; then
              echo "observer leaked into compiler construction graph" >&2
              exit 1
            fi
          done
          mkdir -p "$out"
          cp ${seedClosure}/store-paths "$out/seed-build-paths"
          cp ${seedClosure}/registration "$out/seed-build-registration"
          printf '%s\n' ${(codegenSeed pkgs).drvPath} > "$out/seed-build-roots"
          printf '%s\n' ${(defaultObserver pkgs).drvPath} > "$out/forbidden-observer-drv"
          printf 'a: *1|2\n' > probe.cue
          env -i PATH=/nonexistent ${defaultObserver pkgs}/bin/cuestacean-default-observer inspect probe.cue a > "$out/observation.json"
          printf '%s\n' '{"formatVersion":1,"mode":"inspect","leftDefault":null,"rightDefault":null,"defaultSelected":true,"status":"concrete","value":1}' > expected.json
          cmp expected.json "$out/observation.json"
          printf '%s\n' ${(defaultTrace pkgs).drvPath} > "$out/forbidden-trace-drv"
          env -i PATH=/nonexistent ${defaultTrace pkgs}/bin/cuestacean-default-trace trace probe.cue a > "$out/trace.json"
          jq -c '{formatVersion,mode,defaultSelected,status,value}' "$out/trace.json" > trace-summary.json
          printf '%s\n' '{"formatVersion":1,"mode":"trace","defaultSelected":true,"status":"concrete","value":1}' > trace-expected.json
          cmp trace-expected.json trace-summary.json
          jq -e '.batches | length > 0' "$out/trace.json" > /dev/null
          env -i PATH=/nonexistent ${defaultTrace pkgs}/bin/cuestacean-default-trace trace-origins probe.cue a > "$out/trace-origins.json"
          jq -e '.formatVersion == 2 and .mode == "trace-origins" and .value == 1 and all(.batches[].trials[]; .leftMode == "maybe" and .leftOriginalMode == "maybe")' "$out/trace-origins.json" > /dev/null
          env -i PATH=/nonexistent ${defaultTrace pkgs}/bin/cuestacean-default-trace trace-history probe.cue a > "$out/trace-history.json"
          jq -e '.formatVersion == 3 and .mode == "trace-history" and .value == 1 and .batches[0].results == [{mode:"default",originalMode:"default",value:1},{mode:"not",originalMode:"not",value:2}]' "$out/trace-history.json" > /dev/null
          mkdir -p "$TMPDIR/equality-home"
          env -i PATH=/nonexistent HOME="$TMPDIR/equality-home" ${defaultTrace pkgs}/bin/cuestacean-default-trace trace-equality probe.cue a > "$out/trace-equality.json"
          jq -e '.formatVersion == 4 and .mode == "trace-equality" and .value == 1 and .batches[0].phase == {runMode:"finalize",appends:[{candidate:{mode:"default",originalMode:"default",value:1},index:0,kind:"insert",comparisons:[]},{candidate:{mode:"not",originalMode:"not",value:2},index:1,kind:"insert",comparisons:[{index:0,kind:"final",existing:{mode:"default",originalMode:"default",value:1},accepted:false}]}]}' "$out/trace-equality.json" > /dev/null
        '';

      # Delete only staged build copies, never the working tree. Neither stage
      # may consume leftover working-tree JSON or committed Rust intermediates.
      clearArtifacts = ''
        for fixture in ${fixtureNames}; do
          rm -f "../$fixture.json"
        done
        rm -f ../generated/*.rs
      '';

      codegenSeed =
        pkgs:
        (rustPackage pkgs).overrideAttrs (old: {
          pname = "bootstrap-codegen-seed";
          # Policy is fresh upstream CUE data, not a generated Rust seed dependency.
          CUESTACEAN_BOOTSTRAP_JSON = jsonExports pkgs;
          meta = old.meta // {
            mainProgram = "bootstrap-codegen";
          };
          cargoBuildFlags = [
            "--no-default-features"
            "--bin"
            "bootstrap-codegen"
            "--bin"
            "bootstrap-scan-codegen"
            "--bin"
            "bootstrap-ast-codegen"
            "--bin"
            "bootstrap-pratt-codegen"
            "--bin"
            "bootstrap-unify-codegen"
            "--bin"
            "bootstrap-lower-codegen"
          ];
          postPatch = clearArtifacts;
          # The tests require generated modules. Break that bootstrap cycle here;
          # the final package below runs the complete test suite, including codegen.
          doCheck = false;
        });

      executor =
        pkgs:
        (rustPackage pkgs).overrideAttrs (old: {
          nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [
            pkgs.rustfmt
            (codegenSeed pkgs)
          ];
          CUESTACEAN_BOOTSTRAP_JSON = jsonExports pkgs;
          CUESTACEAN_DEFAULT_OBSERVER = "${defaultObserver pkgs}/bin/cuestacean-default-observer";
          CUESTACEAN_DEFAULT_TRACE = "${defaultTrace pkgs}/bin/cuestacean-default-trace";
          preCheck = old.preCheck + ''
            cargo test --offline --no-default-features --test default_observer
            cargo test --offline --no-default-features --test default_trace
            cargo test --offline --no-default-features --test batch_defaults
            cargo test --offline --no-default-features --test flat_default_sources
            cargo test --offline --no-default-features --test recursive_default_scheduling
            cargo test --offline --no-default-features --test default_equality_trace
            cargo test --offline --no-default-features --test native_coalescing_audit
          '';
          postPatch = clearArtifacts + ''
            # Do not leak strict shell options into later stdenv hooks.
            (
              set -euo pipefail
              for policy in ${loweringNames}; do
                name=$(basename "$policy")
                bootstrap-lower-codegen "$CUESTACEAN_BOOTSTRAP_JSON/$policy.json" | rustfmt --edition 2024 > "../generated/$name.rs"
              done
              for rules in ${jsonExports pkgs}/evaluation.json ${jsonExports pkgs}/tests/valid/evaluation-left-twice.json tests/fixtures/codegen-escaped-rules.json; do
                name=$(basename "$rules" .json)
                bootstrap-codegen "$rules" | rustfmt --edition 2024 > "../generated/$name.rs"
              done
              bootstrap-codegen "$CUESTACEAN_BOOTSTRAP_JSON/evaluation.json" "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/lowering-left-twice.json" | rustfmt --edition 2024 > policy-generated.rs
              cmp policy-generated.rs ../generated/evaluation-left-twice.rs
              mv policy-generated.rs ../generated/evaluation-left-twice.rs
              bootstrap-lower-codegen "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/lowering-numbers.json" "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/meta-lowering-forced.json" | rustfmt --edition 2024 > ../generated/lowering-meta-forced.rs
              bootstrap-lower-codegen "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/meta-driver-keys.json" "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/meta-driver-previous.json" | rustfmt --edition 2024 > ../generated/meta-driver-previous.rs
              bootstrap-codegen "$CUESTACEAN_BOOTSTRAP_JSON/evaluation.json" "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/lowering-reverse.json" | rustfmt --edition 2024 > ../generated/evaluation-reverse.rs
              for machine in ${scannerNames}; do
                name=$(basename "$machine")
                bootstrap-scan-codegen "$CUESTACEAN_BOOTSTRAP_JSON/$machine.json" | rustfmt --edition 2024 > "../generated/$name.rs"
              done
              bootstrap-scan-codegen "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/scanner-advance.json" "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/scan-lowering-unicode.json" | rustfmt --edition 2024 > ../generated/scan-unicode-advance.rs
              for builder in ${builderNames}; do
                name=$(basename "$builder")
                bootstrap-ast-codegen "$CUESTACEAN_BOOTSTRAP_JSON/$builder.json" | rustfmt --edition 2024 > "../generated/$name.rs"
              done
              bootstrap-ast-codegen "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/ast-literal.json" "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/ast-lowering-forced.json" | rustfmt --edition 2024 > ../generated/ast-forced-integer.rs
              for control in ${prattNames}; do
                name=$(basename "$control")
                bootstrap-pratt-codegen "$CUESTACEAN_BOOTSTRAP_JSON/$control.json" | rustfmt --edition 2024 > "../generated/$name.rs"
              done
              bootstrap-pratt-codegen "$CUESTACEAN_BOOTSTRAP_JSON/pratt.json" "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/pratt-lowering-no-increment.json" | rustfmt --edition 2024 > ../generated/pratt-no-increment.rs
              for control in ${unifyNames}; do
                name=$(basename "$control")
                bootstrap-unify-codegen "$CUESTACEAN_BOOTSTRAP_JSON/$control.json" | rustfmt --edition 2024 > "../generated/$name.rs"
              done
              bootstrap-unify-codegen "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/unify-equality-probe.json" "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/unify-lowering-permissive.json" | rustfmt --edition 2024 > ../generated/unify-permissive-bool.rs
              bootstrap-unify-codegen "$CUESTACEAN_BOOTSTRAP_JSON/difference-control.json" "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/integer-kernel-alternate.json" | rustfmt --edition 2024 > ../generated/difference-as-product.rs
              bootstrap-unify-codegen "$CUESTACEAN_BOOTSTRAP_JSON/divide-control.json" "$CUESTACEAN_BOOTSTRAP_JSON/tests/valid/division-kernel-alternate.json" | rustfmt --edition 2024 > ../generated/divide-as-quotient.rs
            )
          '';
        });
      specSources =
        pkgs: tests:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions (
            [
              ../../bootstrap/spec-export/src/lib.rs
              ../../bootstrap/spec-export/src/main.rs
              ../../bootstrap/spec-export/src/model.rs
              ../../bootstrap/spec-export/src/parser.rs
              ../../bootstrap/spec-export/src/eval.rs
              ../../bootstrap/spec-export/src/bindings.rs
              ../../bootstrap/spec-export/Cargo.toml
              ../../bootstrap/spec-export/Cargo.lock
            ]
            ++ pkgs.lib.optionals tests (
              [
                ../../bootstrap/spec-export/tests
                ../../bootstrap/ast-integer-policy.cue
                ../../bootstrap/native-policy-package.cue
                ../../bootstrap/schema.cue
                ../../bootstrap/ast-lowering.cue
                ../../bootstrap/tests/valid/ast-lowering-forced.cue
                ../../bootstrap/ast-lowering-fragments.cue
                ../../bootstrap/ast-lowering-body.cue
                ../../bootstrap/compose-policy.jq
                ../../bootstrap/scanner-schema.cue
                ../../bootstrap/group-open.cue
                ../../bootstrap/ast-schema.cue
                ../../bootstrap/tests/invalid/ast-operation.cue
                ../../bootstrap/spikes/demand-policy-schema.cue
                ../../bootstrap/spikes/demand-policy.cue
                ../../bootstrap/spikes/demand-policy-defer.cue
                ../../bootstrap/spikes/preference-projection-schema.cue
                ../../bootstrap/spikes/preference-projection.cue
                ../../bootstrap/spikes/preference-projection-defer.cue
              ]
              ++ map (fixture: ../../bootstrap + "/${fixture}.cue") builderExportFixtures
            )
          );
        };
      specExporter =
        pkgs:
        pkgs.rustPlatform.buildRustPackage {
          pname = "cuestacean-spec-export";
          version = "0.1.0";
          src = specSources pkgs false;
          postUnpack = ''sourceRoot="$sourceRoot/spec-export"'';
          cargoLock.lockFile = "${specSources pkgs false}/spec-export/Cargo.lock";
          cargoBuildFlags = [
            "--bin"
            "cuestacean-spec-export"
          ];
          # Expected answers/oracles belong only to the separate check derivation.
          doCheck = false;
        };
      specTests =
        pkgs:
        (specExporter pkgs).overrideAttrs (_: {
          pname = "cuestacean-spec-export-tests";
          src = specSources pkgs true;
          doCheck = true;
          nativeCheckInputs = [
            pkgs.rustfmt
            pkgs.clippy
            pkgs.cue
            pkgs.jq
          ];
          checkPhase = ''
            cargo fmt --check
            cargo clippy --offline --all-targets -- -D warnings
            cargo test --offline --all-targets
            cargo test --offline --all-targets --release
          '';
        });
      demandPolicyPairs = builtins.concatStringsSep " " [
        "demand-policy-schema:demand-policy"
        "demand-policy-schema:demand-policy-defer"
        "preference-projection-schema:preference-projection"
        "preference-projection-schema:preference-projection-defer"
      ];
      demandPolicySources =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions [
            ../../bootstrap/spikes/demand-policy-schema.cue
            ../../bootstrap/spikes/demand-policy.cue
            ../../bootstrap/spikes/demand-policy-defer.cue
            ../../bootstrap/spikes/preference-projection-schema.cue
            ../../bootstrap/spikes/preference-projection.cue
            ../../bootstrap/spikes/preference-projection-defer.cue
          ];
        };
      nativeDemandPolicies =
        pkgs:
        pkgs.runCommand "cuestacean-native-demand-policies" { } ''
          set -euo pipefail
          mkdir -p "$out"
          for pair in ${demandPolicyPairs}; do
            schema="''${pair%%:*}"
            program="''${pair#*:}"
            env -i PATH=/nonexistent ${specExporter pkgs}/bin/cuestacean-spec-export \
              ${demandPolicySources pkgs}/spikes/"$schema.cue" \
              ${demandPolicySources pkgs}/spikes/"$program.cue" > first.json
            env -i PATH=/nonexistent ${specExporter pkgs}/bin/cuestacean-spec-export \
              ${demandPolicySources pkgs}/spikes/"$schema.cue" \
              ${demandPolicySources pkgs}/spikes/"$program.cue" > second.json
            cmp first.json second.json
            cp first.json "$out/$program.json"
          done
        '';
      nativeDemandPolicyBuildClosure =
        pkgs:
        pkgs.closureInfo {
          # Derivation roots attest build inputs, not merely runtime references.
          rootPaths = [
            (specExporter pkgs).drvPath
            (nativeDemandPolicies pkgs).drvPath
          ];
        };
      nativeDemandPolicyVerification =
        pkgs:
        pkgs.runCommand "cuestacean-native-demand-policies-verification"
          {
            nativeBuildInputs = [
              pkgs.cue
              pkgs.jq
            ];
          }
          ''
            set -euo pipefail
            # These identities are forbidden only in the native construction
            # graph. This separate oracle/check package is not Go-CUE-free.
            for forbidden in ${pkgs.cue.drvPath} ${pkgs.cue.src} ${(jsonExports pkgs).drvPath} ${(executor pkgs).drvPath} ${(defaultObserver pkgs).drvPath} ${(defaultTrace pkgs).drvPath}; do
              if grep -Fx "$forbidden" ${nativeDemandPolicyBuildClosure pkgs}/store-paths; then
                echo "forbidden bootstrap dependency: $forbidden" >&2; exit 1
              fi
            done
            for pair in ${demandPolicyPairs}; do
              schema="''${pair%%:*}"
              program="''${pair#*:}"
              cue export ${demandPolicySources pkgs}/spikes/"$schema.cue" \
                ${demandPolicySources pkgs}/spikes/"$program.cue" --out json | jq -S . > oracle.json
              cmp oracle.json ${nativeDemandPolicies pkgs}/"$program.json"
            done
            mkdir -p "$out"
            cp ${nativeDemandPolicyBuildClosure pkgs}/store-paths "$out/native-build-paths"
            cp ${nativeDemandPolicyBuildClosure pkgs}/registration "$out/native-build-registration"
            printf '%s\n' ${(specExporter pkgs).drvPath} ${(nativeDemandPolicies pkgs).drvPath} > "$out/native-build-roots"
          '';
      astIntegerPolicySources =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions [
            ../../bootstrap/ast-integer-policy.cue
            ../../bootstrap/native-policy-package.cue
          ];
        };
      nativeAstIntegerPolicy =
        pkgs:
        pkgs.runCommand "cuestacean-native-ast-integer-policy" { } ''
          set -euo pipefail
          mkdir -p "$out" "$PWD/home"
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          test -r "$exporter" && test -x "$exporter"
          for run in first second; do
            env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" \
              ${astIntegerPolicySources pkgs}/native-policy-package.cue \
              ${astIntegerPolicySources pkgs}/ast-integer-policy.cue > "$run.json"
          done
          cmp first.json second.json
          cp first.json "$out/ast-integer-policy.json"
        '';
      nativeAstIntegerPolicyBuildClosure =
        pkgs:
        pkgs.closureInfo {
          rootPaths = [
            (specExporter pkgs).drvPath
            (nativeAstIntegerPolicy pkgs).drvPath
          ];
        };
      nativeAstIntegerPolicyVerification =
        pkgs:
        pkgs.runCommand "cuestacean-native-ast-integer-policy-verification"
          {
            nativeBuildInputs = [
              pkgs.jq
              pkgs.rustfmt
            ];
          }
          ''
            set -euo pipefail
            # This check may use Go CUE and the executor; construction may not.
            for forbidden in ${pkgs.cue} ${pkgs.cue.drvPath} ${pkgs.cue.src} \
              ${jsonExports pkgs} ${(jsonExports pkgs).drvPath} \
              ${executor pkgs} ${(executor pkgs).drvPath} \
              ${defaultObserver pkgs} ${(defaultObserver pkgs).drvPath} \
              ${defaultTrace pkgs} ${(defaultTrace pkgs).drvPath}; do
              if grep -Fx "$forbidden" ${nativeAstIntegerPolicyBuildClosure pkgs}/store-paths; then
                echo "forbidden integer-policy construction dependency: $forbidden" >&2
                exit 1
              fi
            done
            mkdir -p "$out" "$PWD/probe/home"
            cd probe
            exporter=${specExporter pkgs}/bin/cuestacean-spec-export
            oracle=${pkgs.cue}/bin/cue
            codegen=${executor pkgs}/bin/bootstrap-ast-codegen
            for binary in "$exporter" "$oracle" "$codegen"; do
              test -r "$binary" && test -x "$binary"
              case "$binary" in /*) ;; *) exit 1 ;; esac
            done
            native() { env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "$@"; }
            cue() { env -i HOME="$PWD/home" PATH=/nonexistent "$oracle" "$@"; }
            generate() {
              env -i HOME="$PWD/home" PATH=/nonexistent "$codegen" "$@" | rustfmt --edition 2024
            }
            no_export() {
              if cue export "$@" > failed.json 2> failed.err; then
                echo "unexpected concrete policy" >&2; exit 1
              fi
              test ! -s failed.json && test -s failed.err
            }
            cp ${astIntegerPolicySources pkgs}/native-policy-package.cue renamed-package.cue
            cp ${astIntegerPolicySources pkgs}/ast-integer-policy.cue renamed-owner.cue
            # Complete independent answers; neither is a construction input.
            answer='{"cases":{"integer":{"op":"concat","parts":[{"op":"text","value":"Value::from("},{"op":"signed","field":"value"},{"op":"text","value":"i64)"}]}}}'
            printf '%s\n' "$answer" | jq -S . > expected.json
            native renamed-package.cue renamed-owner.cue > default.json
            native renamed-owner.cue renamed-package.cue > reversed.json
            cmp default.json reversed.json
            cmp default.json expected.json
            cmp default.json ${nativeAstIntegerPolicy pkgs}/ast-integer-policy.json
            cue export renamed-package.cue renamed-owner.cue | jq -S . > oracle.json
            cmp default.json oracle.json

            # Both forced paths still observe the RAW preference, in two files.
            printf 'package bootstrap\n{cases:{integer:{op:"text",value:"Value::from(99i64)"}}}\n' > forced.cue
            native renamed-owner.cue forced.cue > forced.json
            printf '%s\n' '{"cases":{"integer":{"op":"text","value":"Value::from(99i64)"}}}' | jq -S . > expected-forced.json
            cmp forced.json expected-forced.json
            cue export renamed-owner.cue forced.cue | jq -S . > oracle-forced.json
            cmp forced.json oracle-forced.json
            printf 'package bootstrap\n{cases:{integer:{op:"text"}}}\n' > incomplete.cue
            if native renamed-owner.cue incomplete.cue > incomplete.json 2> incomplete.err; then exit 1; fi
            test ! -s incomplete.json
            grep -F 'incomplete export demand' incomplete.err
            no_export renamed-owner.cue incomplete.cue
            cue eval renamed-owner.cue incomplete.cue > /dev/null

            schema=${../../bootstrap/schema.cue}
            fragments=${../../bootstrap/ast-lowering-fragments.cue}
            body=${../../bootstrap/ast-lowering-body.cue}
            selector=${../../bootstrap/ast-lowering.cue}
            # Whole policies, under the existing canonicalization, not subtrees.
            cue export "$schema" "$fragments" "$body" "$selector" renamed-owner.cue | jq -S . > raw-policy.json
            cue export "$schema" "$fragments" "$body" "$selector" \
              ${nativeAstIntegerPolicy pkgs}/ast-integer-policy.json | jq -S . > assembled-policy.json
            cmp raw-policy.json assembled-policy.json
            cmp assembled-policy.json ${jsonExports pkgs}/ast-lowering.json
            no_export "$schema" "$fragments" "$body" "$selector"
            cp failed.err missing-provider.err
            grep -F 'incomplete value' missing-provider.err
            no_export "$schema" "$fragments" "$body" "$selector" renamed-owner.cue incomplete.cue
            cue eval "$schema" "$fragments" "$body" "$selector" renamed-owner.cue incomplete.cue > /dev/null
            no_export "$schema" "$fragments" "$body" "$selector" \
              ${nativeAstIntegerPolicy pkgs}/ast-integer-policy.json forced.cue
            cue export "$schema" "$fragments" "$body" renamed-owner.cue \
              ${../../bootstrap/tests/valid/ast-lowering-forced.cue} | jq -S . > forced-policy.json
            cmp forced-policy.json ${jsonExports pkgs}/tests/valid/ast-lowering-forced.json
            generate ${jsonExports pkgs}/tests/valid/ast-literal.json forced-policy.json > forced.rs
            cmp forced.rs ${../../bootstrap/generated/ast-forced-integer.rs}
            for builder in ${builderNames}; do
              name=$(basename "$builder")
              generate ${jsonExports pkgs}/"$builder.json" assembled-policy.json > "$name.rs"
              cmp "$name.rs" ${../../bootstrap/generated}/"$name.rs"
            done

            # Verifier-only source mutation. BOTH alternatives remain in place;
            # preferred source data must affect native JSON, full policy and Rust.
            substitute renamed-owner.cue variant.cue \
              --replace-fail 'Value::from(' 'Value::from(1 + '
            native renamed-package.cue variant.cue > variant.json
            printf '%s\n' '{"cases":{"integer":{"op":"concat","parts":[{"op":"text","value":"Value::from(1 + "},{"op":"signed","field":"value"},{"op":"text","value":"i64)"}]}}}' | jq -S . > expected-variant.json
            cmp variant.json expected-variant.json
            if cmp -s default.json variant.json; then exit 1; fi
            cue export renamed-package.cue variant.cue | jq -S . > oracle-variant.json
            cmp variant.json oracle-variant.json
            cue export "$schema" "$fragments" "$body" "$selector" variant.json | jq -S . > variant-policy.json
            cue export "$schema" "$fragments" "$body" "$selector" variant.cue | jq -S . > raw-variant-policy.json
            cmp variant-policy.json raw-variant-policy.json
            if cmp -s assembled-policy.json variant-policy.json; then exit 1; fi
            generate ${jsonExports pkgs}/tests/valid/ast-literal.json variant-policy.json > variant.rs
            printf '%s\n' \
              '// Generated by bootstrap-ast-codegen. Do not edit by hand.' \
              'use serde_json::Value;' \
              'pub fn build() -> Value {' \
              '    Value::from(1 + 7i64)' \
              '}' > expected-variant.rs
            cmp variant.rs expected-variant.rs
            cp default.json forced.json incomplete.err missing-provider.err \
              raw-policy.json assembled-policy.json variant.json variant-policy.json variant.rs "$out/"
            cp ${nativeAstIntegerPolicyBuildClosure pkgs}/store-paths "$out/native-build-paths"
            cp ${nativeAstIntegerPolicyBuildClosure pkgs}/registration "$out/native-build-registration"
            printf '%s\n' ${(specExporter pkgs).drvPath} ${(nativeAstIntegerPolicy pkgs).drvPath} > "$out/native-build-roots"
          '';
      astLoweringPolicySources =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions [
            ../../bootstrap/native-policy-package.cue
            ../../bootstrap/ast-lowering-body.cue
          ];
        };
      nativeAstLoweringPolicy =
        pkgs:
        pkgs.runCommand "cuestacean-native-ast-lowering-policy" { } ''
          set -euo pipefail
          mkdir -p "$PWD/home"
          exporter=${specExporter pkgs}/bin/cuestacean-spec-export
          jq=${pkgs.jq}/bin/jq
          for binary in "$exporter" "$jq"; do
            test -r "$binary" && test -x "$binary"
            case "$binary" in /*) ;; *) exit 1 ;; esac
          done
          integer=${nativeAstIntegerPolicy pkgs}/ast-integer-policy.json
          test "$(wc -c < "$integer")" -le 65536
          for run in first second; do
            # Each whole export retains its own unchanged native invocation ledger.
            env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" \
              ${astLoweringPolicySources pkgs}/native-policy-package.cue \
              ${astLoweringPolicySources pkgs}/ast-lowering-body.cue > "$run-body.json"
            test "$(wc -c < "$run-body.json")" -le 65536
            env -i HOME="$PWD/home" PATH=/nonexistent "$jq" -s -S \
              -f ${../../bootstrap/compose-policy.jq} "$run-body.json" "$integer" > "$run.json"
            env -i HOME="$PWD/home" PATH=/nonexistent "$jq" -s -S \
              -f ${../../bootstrap/compose-policy.jq} "$integer" "$run-body.json" > "$run-reversed.json"
            test "$(wc -c < "$run.json")" -le 65536
            test "$(wc -c < "$run-reversed.json")" -le 65536
            cmp "$run.json" "$run-reversed.json"
          done
          cmp first-body.json second-body.json
          cmp first.json second.json
          mkdir -p "$out"
          cp first.json "$out/ast-lowering.json"
        '';
      nativeAstLoweringPolicyBuildClosure =
        pkgs:
        pkgs.closureInfo {
          rootPaths = [
            (specExporter pkgs).drvPath
            (nativeAstIntegerPolicy pkgs).drvPath
            (nativeAstLoweringPolicy pkgs).drvPath
          ];
        };
      nativeAstLoweringPolicyVerification =
        pkgs:
        pkgs.runCommand "cuestacean-native-ast-lowering-policy-verification"
          {
            nativeBuildInputs = [
              pkgs.jq
              pkgs.rustfmt
            ];
          }
          ''
            set -euo pipefail
            # Independent validation is allowed Go/schema/compiler dependencies;
            # none may feed the native construction graph.
            for forbidden in ${pkgs.cue} ${pkgs.cue.drvPath} ${pkgs.cue.src} \
              ${jsonExports pkgs} ${(jsonExports pkgs).drvPath} \
              ${executor pkgs} ${(executor pkgs).drvPath} \
              ${defaultObserver pkgs} ${(defaultObserver pkgs).drvPath} \
              ${defaultTrace pkgs} ${(defaultTrace pkgs).drvPath}; do
              if grep -Fx "$forbidden" ${nativeAstLoweringPolicyBuildClosure pkgs}/store-paths; then
                echo "forbidden complete-policy construction dependency: $forbidden" >&2
                exit 1
              fi
            done
            mkdir -p "$out" "$PWD/probe/home"
            cd probe
            exporter=${specExporter pkgs}/bin/cuestacean-spec-export
            oracle=${pkgs.cue}/bin/cue
            codegen=${executor pkgs}/bin/bootstrap-ast-codegen
            jq=${pkgs.jq}/bin/jq
            for binary in "$exporter" "$oracle" "$codegen" "$jq"; do
              test -r "$binary" && test -x "$binary"
              case "$binary" in /*) ;; *) exit 1 ;; esac
            done
            native() { env -i HOME="$PWD/home" PATH=/nonexistent "$exporter" "$@"; }
            cue() { env -i HOME="$PWD/home" PATH=/nonexistent "$oracle" "$@"; }
            compose() {
              test "$(wc -c < "$1")" -le 65536
              test "$(wc -c < "$2")" -le 65536
              env -i HOME="$PWD/home" PATH=/nonexistent "$jq" -s -S \
                -f ${../../bootstrap/compose-policy.jq} "$@" > composed.json
              test "$(wc -c < composed.json)" -le 65536
            }
            generate() {
              env -i HOME="$PWD/home" PATH=/nonexistent "$codegen" "$@" | rustfmt --edition 2024
            }
            no_export() {
              if cue export "$@" > failed.json 2> failed.err; then exit 1; fi
              test ! -s failed.json && test -s failed.err
            }
            schema=${../../bootstrap/schema.cue}
            bridge=${../../bootstrap/ast-lowering-fragments.cue}
            selector=${../../bootstrap/ast-lowering.cue}
            cp ${astLoweringPolicySources pkgs}/native-policy-package.cue package.cue
            cp ${astLoweringPolicySources pkgs}/ast-lowering-body.cue body.cue
            cp ${astIntegerPolicySources pkgs}/ast-integer-policy.cue integer.cue
            native package.cue body.cue > body.json
            native body.cue package.cue > reversed-body.json
            cmp body.json reversed-body.json
            native package.cue integer.cue > integer.json
            cmp integer.json ${nativeAstIntegerPolicy pkgs}/ast-integer-policy.json
            compose body.json integer.json
            cp composed.json policy.json
            compose integer.json body.json
            cmp composed.json policy.json
            cmp policy.json ${nativeAstLoweringPolicy pkgs}/ast-lowering.json
            cmp policy.json ${jsonExports pkgs}/ast-lowering.json
            cue export "$schema" "$bridge" body.cue integer.cue "$selector" | jq -S . > raw-policy.json
            cmp policy.json raw-policy.json
            cue export "$selector" integer.cue body.cue "$bridge" "$schema" | jq -S . > reversed-policy.json
            cmp policy.json reversed-policy.json
            # Schema validation remains independent, not an exporter capability.
            cue export "$schema" "$bridge" "$selector" policy.json | jq -S . > validated-policy.json
            cmp policy.json validated-policy.json
            for missing in body integer; do
              if test "$missing" = body; then provider=integer.cue; else provider=body.cue; fi
              no_export "$schema" "$bridge" "$selector" "$provider"
              grep -F 'incomplete value' failed.err
              cue eval "$schema" "$bridge" "$selector" "$provider" > /dev/null
              cp failed.err "$missing-missing.err"
            done
            for builder in ${builderNames}; do
              name=$(basename "$builder")
              generate ${jsonExports pkgs}/"$builder.json" policy.json > "$name.rs"
              cmp "$name.rs" ${../../bootstrap/generated}/"$name.rs"
            done

            # Ordinary lexical body source, not JSON patching: all parameter/use
            # spellings change together, preserving Copy/Owned binding semantics.
            substitute body.cue variant-body.cue --replace-fail '_arg' '_native'
            native package.cue variant-body.cue > variant-body.json
            compose variant-body.json integer.json
            cp composed.json body-policy.json
            cue export "$schema" "$bridge" "$selector" variant-body.cue integer.cue | jq -S . > raw-body-policy.json
            cmp body-policy.json raw-body-policy.json
            if cmp -s policy.json body-policy.json; then exit 1; fi
            generate ${jsonExports pkgs}/ast-atom.json body-policy.json > body-variant.rs
            printf '%s\n' \
              '// Generated by bootstrap-ast-codegen. Do not edit by hand.' \
              'use serde_json::Value;' \
              'pub fn build(_native0: String) -> Value {' \
              '    Value::Object(serde_json::Map::from_iter([(' \
              '        String::from("kind"),' \
              '        Value::String(_native0),' \
              '    )]))' \
              '}' > expected-body.rs
            cmp body-variant.rs expected-body.rs

            # Independently change the preferred integer SOURCE; retain BOTH
            # alternatives and construct the complete native policy before use.
            substitute integer.cue variant-integer.cue \
              --replace-fail 'Value::from(' 'Value::from(1 + '
            native package.cue variant-integer.cue > variant-integer.json
            compose body.json variant-integer.json
            cp composed.json integer-policy.json
            cue export "$schema" "$bridge" "$selector" body.cue variant-integer.cue | jq -S . > raw-integer-policy.json
            cmp integer-policy.json raw-integer-policy.json
            if cmp -s policy.json integer-policy.json; then exit 1; fi
            generate ${jsonExports pkgs}/tests/valid/ast-literal.json integer-policy.json > integer-variant.rs
            printf '%s\n' \
              '// Generated by bootstrap-ast-codegen. Do not edit by hand.' \
              'use serde_json::Value;' \
              'pub fn build() -> Value {' \
              '    Value::from(1 + 7i64)' \
              '}' > expected-integer.rs
            cmp integer-variant.rs expected-integer.rs

            # Invalid instructions are still materializable DATA, never admitted
            # schema/typed compiler programs. Neither validator emits code.
            for invalid in opcode shape; do
              if test "$invalid" = opcode; then
                substitute body.cue invalid.cue --replace-fail 'op: "join"' 'op: "notAnOpcode"'
              else
                substitute body.cue invalid.cue --replace-fail 'parts: [' 'wrongParts: ['
              fi
              native package.cue invalid.cue > invalid-body.json
              compose invalid-body.json integer.json
              no_export "$schema" "$bridge" "$selector" composed.json
              if env -i HOME="$PWD/home" PATH=/nonexistent "$codegen" \
                ${jsonExports pkgs}/ast-atom.json composed.json > invalid.rs 2> invalid.err; then exit 1; fi
              test ! -s invalid.rs && test -s invalid.err
              cp failed.err "$invalid-schema.err"
              cp invalid.err "$invalid-typed.err"
            done
            cp body.json integer.json policy.json raw-policy.json body-policy.json integer-policy.json \
              body-variant.rs integer-variant.rs *-missing.err *-schema.err *-typed.err "$out/"
            cp ${nativeAstLoweringPolicyBuildClosure pkgs}/store-paths "$out/native-build-paths"
            cp ${nativeAstLoweringPolicyBuildClosure pkgs}/registration "$out/native-build-registration"
            printf '%s\n' ${(specExporter pkgs).drvPath} ${(nativeAstIntegerPolicy pkgs).drvPath} \
              ${(nativeAstLoweringPolicy pkgs).drvPath} > "$out/native-build-roots"
          '';
      builderSources =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions (
            [ ../../bootstrap/ast-schema.cue ]
            ++ map (fixture: ../../bootstrap + "/${fixture}.cue") builderExportFixtures
          );
        };
      nativeBuilders =
        pkgs:
        pkgs.runCommand "cuestacean-native-ast-builders" { } ''
          set -euo pipefail
          for fixture in ${builderExportNames}; do
            env -i PATH=/nonexistent ${specExporter pkgs}/bin/cuestacean-spec-export \
              ${builderSources pkgs}/ast-schema.cue ${builderSources pkgs}/"$fixture.cue" > first.json
            env -i PATH=/nonexistent ${specExporter pkgs}/bin/cuestacean-spec-export \
              ${builderSources pkgs}/ast-schema.cue ${builderSources pkgs}/"$fixture.cue" > second.json
            cmp first.json second.json
            mkdir -p "$(dirname "$out/$fixture.json")"
            cp first.json "$out/$fixture.json"
          done
        '';
      nativeBuilderBuildClosure =
        pkgs:
        pkgs.closureInfo {
          # Attest the entire build graph, not only runtime output references.
          rootPaths = [
            (specExporter pkgs).drvPath
            (nativeBuilders pkgs).drvPath
          ];
        };
      nativeBuilderVerification =
        pkgs:
        pkgs.runCommand "cuestacean-native-builder-family-verification"
          {
            nativeBuildInputs = [
              pkgs.cue
              pkgs.jq
              pkgs.rustfmt
            ];
          }
          ''
            set -euo pipefail
            for forbidden in ${pkgs.cue.drvPath} ${pkgs.cue.src} ${(jsonExports pkgs).drvPath} ${(executor pkgs).drvPath} ${(defaultObserver pkgs).drvPath} ${(defaultTrace pkgs).drvPath}; do
              if grep -Fx "$forbidden" ${nativeBuilderBuildClosure pkgs}/store-paths; then
                echo "forbidden bootstrap dependency: $forbidden" >&2; exit 1
              fi
            done
            mkdir -p "$out"
            for fixture in ${builderExportNames}; do
              cue export ${builderSources pkgs}/ast-schema.cue ${builderSources pkgs}/"$fixture.cue" --out json | jq -S . > expected.json
              cmp expected.json ${nativeBuilders pkgs}/"$fixture.json"
              name=$(basename "$fixture")
              case "$fixture" in
                tests/valid/ast-literal)
                  # The specification contains 7; this alternate compiler policy emits 99.
                  name=ast-forced-integer
                  ${executor pkgs}/bin/bootstrap-ast-codegen ${nativeBuilders pkgs}/"$fixture.json" \
                    ${jsonExports pkgs}/tests/valid/ast-lowering-forced.json \
                    | rustfmt --edition 2024 > actual.rs
                  ;;
                *)
                  ${executor pkgs}/bin/bootstrap-ast-codegen ${nativeBuilders pkgs}/"$fixture.json" \
                    | rustfmt --edition 2024 > actual.rs
                  ;;
              esac
              cmp actual.rs ${../../bootstrap/generated}/"$name.rs"
              cp actual.rs "$out/$name.rs"
            done
            cp ${nativeBuilderBuildClosure pkgs}/store-paths "$out/native-build-paths"
            cp ${nativeBuilderBuildClosure pkgs}/registration "$out/native-build-registration"
            printf '%s\n' ${(specExporter pkgs).drvPath} ${(nativeBuilders pkgs).drvPath} > "$out/native-build-roots"
          '';
      atomSources =
        pkgs:
        pkgs.lib.fileset.toSource {
          root = ../../bootstrap;
          fileset = pkgs.lib.fileset.unions [
            ../../bootstrap/ast-schema.cue
            ../../bootstrap/ast-atom.cue
          ];
        };
      nativeAtom =
        pkgs:
        pkgs.runCommand "cuestacean-native-ast-atom" { } ''
          mkdir -p "$out"
          env -i PATH=/nonexistent ${specExporter pkgs}/bin/cuestacean-spec-export \
            ${atomSources pkgs}/ast-schema.cue ${atomSources pkgs}/ast-atom.cue > "$out/ast-atom.json"
        '';
      nativeBuildClosure =
        pkgs:
        pkgs.closureInfo {
          # .drv roots include build dependencies, not just output references.
          rootPaths = [
            (specExporter pkgs).drvPath
            (nativeAtom pkgs).drvPath
          ];
        };
      nativeVerification =
        pkgs:
        pkgs.runCommand "cuestacean-native-export-verification"
          {
            nativeBuildInputs = [
              pkgs.cue
              pkgs.jq
              pkgs.rustfmt
            ];
          }
          ''
            set -euo pipefail
            for forbidden in ${pkgs.cue.drvPath} ${pkgs.cue.src} ${(jsonExports pkgs).drvPath} ${(executor pkgs).drvPath} ${(defaultObserver pkgs).drvPath} ${(defaultTrace pkgs).drvPath}; do
              if grep -Fx "$forbidden" ${nativeBuildClosure pkgs}/store-paths; then
                echo "forbidden bootstrap dependency: $forbidden" >&2; exit 1
              fi
            done
            cue export ${atomSources pkgs}/ast-schema.cue ${atomSources pkgs}/ast-atom.cue --out json | jq -S . > expected
            cmp expected ${nativeAtom pkgs}/ast-atom.json
            env -i PATH=/nonexistent ${specExporter pkgs}/bin/cuestacean-spec-export \
              ${atomSources pkgs}/ast-schema.cue ${atomSources pkgs}/ast-atom.cue > repeated
            cmp repeated ${nativeAtom pkgs}/ast-atom.json
            ${executor pkgs}/bin/bootstrap-ast-codegen ${nativeAtom pkgs}/ast-atom.json \
              | rustfmt --edition 2024 > actual.rs
            cmp actual.rs ${../../bootstrap/generated/ast-atom.rs}
            # Test-only raw fragment/answers: none feed specExporter/nativeAtom.
            # Produce with the minimal production binary BEFORE oracle comparison.
            tagged="$PWD/tagged"
            mkdir -p "$tagged/home"
            cp ${astIntegerPolicySources pkgs}/ast-integer-policy.cue "$tagged/program.cue"
            test -r ${specExporter pkgs}/bin/cuestacean-spec-export
            test -x ${specExporter pkgs}/bin/cuestacean-spec-export
            for name in default text incomplete; do
              case "$name" in
                default) context='{}' ;;
                text) context='{cases:{integer:{op:"text",value:"x"}}}' ;;
                incomplete) context='{cases:{integer:{op:"text"}}}' ;;
              esac
              printf 'package bootstrap\n%s\n' "$context" > "$tagged/schema.cue"
              if [ "$name" = incomplete ]; then
                if env -i HOME="$tagged/home" PATH=/nonexistent ${specExporter pkgs}/bin/cuestacean-spec-export \
                  "$tagged/schema.cue" "$tagged/program.cue" > "$tagged/$name.json" 2> "$tagged/$name.err"; then
                  echo "incomplete tagged branch unexpectedly exported" >&2; exit 1
                fi
                test ! -s "$tagged/$name.json"
                grep -F 'incomplete export demand' "$tagged/$name.err"
                if env -i HOME="$tagged/home" PATH=/nonexistent ${pkgs.cue}/bin/cue export \
                  "$tagged/schema.cue" "$tagged/program.cue" > "$tagged/oracle.json" 2> "$tagged/oracle.err"; then
                  echo "incomplete oracle unexpectedly exported" >&2; exit 1
                fi
                test ! -s "$tagged/oracle.json"
                env -i HOME="$tagged/home" PATH=/nonexistent ${pkgs.cue}/bin/cue eval \
                  "$tagged/schema.cue" "$tagged/program.cue" > /dev/null
              else
                env -i HOME="$tagged/home" PATH=/nonexistent ${specExporter pkgs}/bin/cuestacean-spec-export \
                  "$tagged/schema.cue" "$tagged/program.cue" > "$tagged/$name.json"
                env -i HOME="$tagged/home" PATH=/nonexistent ${specExporter pkgs}/bin/cuestacean-spec-export \
                  "$tagged/program.cue" "$tagged/schema.cue" > "$tagged/repeated.json"
                cmp "$tagged/repeated.json" "$tagged/$name.json"
                case "$name" in
                  default) answer='{"cases":{"integer":{"op":"concat","parts":[{"op":"text","value":"Value::from("},{"op":"signed","field":"value"},{"op":"text","value":"i64)"}]}}}' ;;
                  text) answer='{"cases":{"integer":{"op":"text","value":"x"}}}' ;;
                esac
                printf '%s\n' "$answer" | jq -S . > "$tagged/expected.json"
                cmp "$tagged/expected.json" "$tagged/$name.json"
                env -i HOME="$tagged/home" PATH=/nonexistent ${pkgs.cue}/bin/cue export \
                  "$tagged/schema.cue" "$tagged/program.cue" | jq -S . > "$tagged/oracle.json"
                cmp "$tagged/oracle.json" "$tagged/$name.json"
              fi
            done
            mkdir -p "$out"
            cp "$tagged/default.json" "$tagged/text.json" "$tagged/incomplete.err" "$out/"
            cp ${nativeBuildClosure pkgs}/store-paths "$out/native-build-paths"
            cp ${nativeBuildClosure pkgs}/registration "$out/native-build-registration"
            printf '%s\n' ${(specExporter pkgs).drvPath} ${(nativeAtom pkgs).drvPath} > "$out/native-build-roots"
            cp actual.rs "$out/ast-atom.rs"
          '';
    in
    {
      packages = kolohelios-nix.lib.forEachSupportedSystem (
        { pkgs, ... }: {
          default = executor pkgs;
          exports = jsonExports pkgs;
          spec-export = specExporter pkgs;
          native-ast-atom = nativeAtom pkgs;
          native-ast-builders = nativeBuilders pkgs;
          native-demand-policies = nativeDemandPolicies pkgs;
          native-ast-integer-policy = nativeAstIntegerPolicy pkgs;
          native-ast-lowering-policy = nativeAstLoweringPolicy pkgs;
          default-observer = defaultObserver pkgs;
          default-trace = defaultTrace pkgs;
        }
      );

      apps = kolohelios-nix.lib.forEachSupportedSystem (
        { pkgs, ... }:
        {
          generated = {
            type = "app";
            program = "${executor pkgs}/bin/bootstrap-generated";
            meta.description = "Evaluate JSON with the CUE-defined rules compiled into Rust";
          };
          source = {
            type = "app";
            program = "${executor pkgs}/bin/bootstrap-source";
            meta.description = "Evaluate the bootstrap decimal integer expression subset";
          };
        }
      );

      devShells = kolohelios-nix.lib.forEachSupportedSystem (
        { system, pkgs, ... }:
        {
          default = pkgs.mkShell {
            CUESTACEAN_BOOTSTRAP_JSON = jsonExports pkgs;
            CUESTACEAN_DEMAND_POLICIES = nativeDemandPolicies pkgs;
            CUESTACEAN_DEFAULT_OBSERVER = "${defaultObserver pkgs}/bin/cuestacean-default-observer";
            CUESTACEAN_DEFAULT_TRACE = "${defaultTrace pkgs}/bin/cuestacean-default-trace";
            packages = [
              pkgs.cue
              pkgs.jq
              pkgs.rustc
              pkgs.cargo
              pkgs.rustfmt
              pkgs.clippy
              pkgs.actionlint
              pkgs.shellcheck
              pkgs.taplo
            ]
            ++ kolohelios-nix.lib.workflowPackages pkgs;

            # The shared tooling includes a monorepo-only shaka shim.
            shellHook = ''
              export PATH="${shaka.packages.${system}.default}/bin:$PATH"
            '';
          };
        }
      );

      checks = kolohelios-nix.lib.forEachSupportedSystem (
        { pkgs, ... }:
        {
          bootstrap-executor = executor pkgs;
          native-spec-tests = specTests pkgs;
          default-observer = defaultObserverVerification pkgs;
          native-spec-closure = nativeVerification pkgs;
          native-builder-family = nativeBuilderVerification pkgs;
          native-demand-policies = nativeDemandPolicyVerification pkgs;
          native-ast-integer-policy = nativeAstIntegerPolicyVerification pkgs;
          native-ast-lowering-policy = nativeAstLoweringPolicyVerification pkgs;
          bootstrap-codegen =
            pkgs.runCommand "cuestacean-bootstrap-codegen"
              {
                nativeBuildInputs = [
                  (executor pkgs)
                  pkgs.rustfmt
                ];
              }
              ''
                set -euo pipefail
                for rules in ${jsonExports pkgs}/evaluation.json ${jsonExports pkgs}/tests/valid/evaluation-left-twice.json ${../../bootstrap/executor/tests/fixtures}/codegen-escaped-rules.json; do
                  name=$(basename "$rules" .json)
                  bootstrap-codegen "$rules" | rustfmt --edition 2024 > first.rs
                  bootstrap-codegen "$rules" | rustfmt --edition 2024 > second.rs
                  cmp first.rs second.rs
                  diff -u ${../../bootstrap/generated}/"$name.rs" first.rs
                done
                for policy in ${loweringNames}; do
                  name=$(basename "$policy")
                  bootstrap-lower-codegen ${jsonExports pkgs}/"$policy.json" | rustfmt --edition 2024 > first.rs
                  bootstrap-lower-codegen ${jsonExports pkgs}/"$policy.json" | rustfmt --edition 2024 > second.rs
                  cmp first.rs second.rs
                  diff -u ${../../bootstrap/generated}/"$name.rs" first.rs
                done
                bootstrap-codegen ${jsonExports pkgs}/evaluation.json ${jsonExports pkgs}/tests/valid/lowering-left-twice.json | rustfmt --edition 2024 > policy.rs
                diff -u ${../../bootstrap/generated}/evaluation-left-twice.rs policy.rs
                bootstrap-codegen ${jsonExports pkgs}/evaluation.json ${jsonExports pkgs}/tests/valid/lowering-reverse.json | rustfmt --edition 2024 > reverse.rs
                bootstrap-codegen ${jsonExports pkgs}/evaluation.json ${jsonExports pkgs}/tests/valid/lowering-reverse.json | rustfmt --edition 2024 > reverse-second.rs
                cmp reverse.rs reverse-second.rs
                diff -u ${../../bootstrap/generated}/evaluation-reverse.rs reverse.rs
                bootstrap-lower-codegen ${jsonExports pkgs}/tests/valid/lowering-numbers.json ${jsonExports pkgs}/tests/valid/meta-lowering-forced.json | rustfmt --edition 2024 > meta-forced.rs
                bootstrap-lower-codegen ${jsonExports pkgs}/tests/valid/lowering-numbers.json ${jsonExports pkgs}/tests/valid/meta-lowering-forced.json | rustfmt --edition 2024 > meta-forced-second.rs
                cmp meta-forced.rs meta-forced-second.rs
                diff -u ${../../bootstrap/generated}/lowering-meta-forced.rs meta-forced.rs
                bootstrap-lower-codegen ${jsonExports pkgs}/tests/valid/meta-driver-keys.json ${jsonExports pkgs}/tests/valid/meta-driver-previous.json | rustfmt --edition 2024 > driver-previous.rs
                bootstrap-lower-codegen ${jsonExports pkgs}/tests/valid/meta-driver-keys.json ${jsonExports pkgs}/tests/valid/meta-driver-previous.json | rustfmt --edition 2024 > driver-previous-second.rs
                cmp driver-previous.rs driver-previous-second.rs
                diff -u ${../../bootstrap/generated}/meta-driver-previous.rs driver-previous.rs
                for machine in ${scannerNames}; do
                  name=$(basename "$machine")
                  bootstrap-scan-codegen ${jsonExports pkgs}/"$machine.json" | rustfmt --edition 2024 > first.rs
                  bootstrap-scan-codegen ${jsonExports pkgs}/"$machine.json" | rustfmt --edition 2024 > second.rs
                  cmp first.rs second.rs
                  diff -u ${../../bootstrap/generated}/"$name.rs" first.rs
                done
                bootstrap-scan-codegen ${jsonExports pkgs}/tests/valid/scanner-advance.json ${jsonExports pkgs}/tests/valid/scan-lowering-unicode.json | rustfmt --edition 2024 > scan-alternate.rs
                bootstrap-scan-codegen ${jsonExports pkgs}/tests/valid/scanner-advance.json ${jsonExports pkgs}/tests/valid/scan-lowering-unicode.json | rustfmt --edition 2024 > scan-alternate-second.rs
                cmp scan-alternate.rs scan-alternate-second.rs
                diff -u ${../../bootstrap/generated}/scan-unicode-advance.rs scan-alternate.rs
                for builder in ${builderNames}; do
                  name=$(basename "$builder")
                  bootstrap-ast-codegen ${jsonExports pkgs}/"$builder.json" | rustfmt --edition 2024 > first.rs
                  bootstrap-ast-codegen ${jsonExports pkgs}/"$builder.json" | rustfmt --edition 2024 > second.rs
                  cmp first.rs second.rs
                  diff -u ${../../bootstrap/generated}/"$name.rs" first.rs
                done
                bootstrap-ast-codegen ${jsonExports pkgs}/tests/valid/ast-literal.json ${jsonExports pkgs}/tests/valid/ast-lowering-forced.json | rustfmt --edition 2024 > ast-forced.rs
                bootstrap-ast-codegen ${jsonExports pkgs}/tests/valid/ast-literal.json ${jsonExports pkgs}/tests/valid/ast-lowering-forced.json | rustfmt --edition 2024 > ast-forced-second.rs
                cmp ast-forced.rs ast-forced-second.rs
                diff -u ${../../bootstrap/generated}/ast-forced-integer.rs ast-forced.rs
                for control in ${prattNames}; do
                  name=$(basename "$control")
                  bootstrap-pratt-codegen ${jsonExports pkgs}/"$control.json" | rustfmt --edition 2024 > first.rs
                  bootstrap-pratt-codegen ${jsonExports pkgs}/"$control.json" | rustfmt --edition 2024 > second.rs
                  cmp first.rs second.rs
                  diff -u ${../../bootstrap/generated}/"$name.rs" first.rs
                done
                bootstrap-pratt-codegen ${jsonExports pkgs}/pratt.json ${jsonExports pkgs}/tests/valid/pratt-lowering-no-increment.json | rustfmt --edition 2024 > pratt-alternate.rs
                bootstrap-pratt-codegen ${jsonExports pkgs}/pratt.json ${jsonExports pkgs}/tests/valid/pratt-lowering-no-increment.json | rustfmt --edition 2024 > pratt-alternate-second.rs
                cmp pratt-alternate.rs pratt-alternate-second.rs
                diff -u ${../../bootstrap/generated}/pratt-no-increment.rs pratt-alternate.rs
                for control in ${unifyNames}; do
                  name=$(basename "$control")
                  bootstrap-unify-codegen ${jsonExports pkgs}/"$control.json" | rustfmt --edition 2024 > first.rs
                  bootstrap-unify-codegen ${jsonExports pkgs}/"$control.json" | rustfmt --edition 2024 > second.rs
                  cmp first.rs second.rs
                  diff -u ${../../bootstrap/generated}/"$name.rs" first.rs
                done
                bootstrap-unify-codegen ${jsonExports pkgs}/tests/valid/unify-equality-probe.json ${jsonExports pkgs}/tests/valid/unify-lowering-permissive.json | rustfmt --edition 2024 > unify-alternate.rs
                bootstrap-unify-codegen ${jsonExports pkgs}/tests/valid/unify-equality-probe.json ${jsonExports pkgs}/tests/valid/unify-lowering-permissive.json | rustfmt --edition 2024 > unify-alternate-second.rs
                cmp unify-alternate.rs unify-alternate-second.rs
                diff -u ${../../bootstrap/generated}/unify-permissive-bool.rs unify-alternate.rs
                bootstrap-unify-codegen ${jsonExports pkgs}/difference-control.json ${jsonExports pkgs}/tests/valid/integer-kernel-alternate.json | rustfmt --edition 2024 > integer-alternate.rs
                bootstrap-unify-codegen ${jsonExports pkgs}/difference-control.json ${jsonExports pkgs}/tests/valid/integer-kernel-alternate.json | rustfmt --edition 2024 > integer-alternate-second.rs
                cmp integer-alternate.rs integer-alternate-second.rs
                diff -u ${../../bootstrap/generated}/difference-as-product.rs integer-alternate.rs
                bootstrap-unify-codegen ${jsonExports pkgs}/divide-control.json ${jsonExports pkgs}/tests/valid/division-kernel-alternate.json | rustfmt --edition 2024 > division-alternate.rs
                bootstrap-unify-codegen ${jsonExports pkgs}/divide-control.json ${jsonExports pkgs}/tests/valid/division-kernel-alternate.json | rustfmt --edition 2024 > division-alternate-second.rs
                cmp division-alternate.rs division-alternate-second.rs
                diff -u ${../../bootstrap/generated}/divide-as-quotient.rs division-alternate.rs
                touch "$out"
              '';
          bootstrap-source =
            pkgs.runCommand "cuestacean-bootstrap-source"
              {
                nativeBuildInputs = [
                  (executor pkgs)
                  pkgs.cue
                  pkgs.jq
                ];
              }
              ''
                set -euo pipefail
                printf 'package probe\nimport "encoding/json"\nout: json.Marshal(value)\n' > compact.cue
                for source in ${../../bootstrap/source}/valid/*.cue; do
                  cue export "$source" --out json > expected
                  # CUE can spell integer zero as -0; i64 output is canonical 0.
                  # Normalize only that spelling, without rounding any integers.
                  IFS= read -r scalar < expected
                  if [ "$scalar" = "-0" ]; then printf '0\n' > expected; fi
                  bootstrap-source "$source" > actual
                  case "$scalar" in
                    '['*|'{'*)
                      # Reimport/marshal collections with CUE: exact integers,
                      # normalized whitespace/string escapes, never jq numbers.
                      # Record fixtures spell byte-ordered labels at every level;
                      # independent Rust tests pin ordering of arbitrary input.
                      for side in expected actual; do
                        cue import -p probe -l '"value"' --outfile - json: "$side" > value.cue
                        cue export value.cue compact.cue -e out --out text > normalized-"$side"
                      done
                      diff -u normalized-expected normalized-actual
                      ;;
                    \"*)
                      # String encoders may differ in JSON escape spelling.
                      jq -e 'type == "string"' expected > /dev/null
                      jq -e 'type == "string"' actual > /dev/null
                      jq -c . expected > normalized-expected
                      jq -c . actual > normalized-actual
                      diff -u normalized-expected normalized-actual
                      ;;
                    *) diff -u expected actual ;;
                  esac
                done
                for source in ${../../bootstrap/source}/conflicts/*.cue ${../../bootstrap/source}/incomplete/*.cue; do
                  if cue export "$source" --out json > expected 2> cue-error; then
                    echo "Expected upstream export failure: $source" >&2
                    exit 1
                  fi
                  if bootstrap-source "$source" > actual 2> error; then
                    echo "Expected bootstrap export failure: $source" >&2
                    exit 1
                  fi
                  test ! -s expected
                  test -s cue-error
                  test ! -s actual
                  test -s error
                done
                # These selected domain/syntax rejections must also fail upstream.
                for source in ${../../bootstrap/source}/rejected/ordering-{boolean,mixed}.cue ${../../bootstrap/source}/rejected/logic-{left,selected,eager,or-eager}.cue ${../../bootstrap/source}/rejected/call-{domain,arity,empty,name-newline}.cue ${../../bootstrap/source}/rejected/list-index-{range,negative,type,close}.cue ${../../bootstrap/source}/rejected/slice-{range,domain,stride}.cue ${../../bootstrap/source}/rejected/conjunction-{domain,close}.cue ${../../bootstrap/source}/rejected/bounds-exclusive-domain.cue ${../../bootstrap/source}/rejected/not-integer.cue ${../../bootstrap/source}/rejected/null-conflict.cue ${../../bootstrap/source}/rejected/radix-uppercase-{binary,octal}.cue; do
                  if cue export "$source" --out json > expected 2> cue-error; then exit 1; fi
                  test ! -s expected
                  test -s cue-error
                done
                for source in ${../../bootstrap/source}/rejected/*.cue; do
                  if bootstrap-source "$source" > actual 2> error; then
                    echo "Expected rejection: $source" >&2
                    exit 1
                  fi
                  test ! -s actual
                  test -s error
                done
                touch "$out"
              '';
          bootstrap-values =
            pkgs.runCommand "cuestacean-bootstrap-values"
              {
                nativeBuildInputs = [
                  (executor pkgs)
                  pkgs.cue
                  pkgs.jq
                ];
              }
              ''
                set -euo pipefail
                for fixture in lists list-expressions conjunction list-slicing optional-slices nested-lists records records-unify record-expressions nested-records nested-records-unify mixed-collections mixed-unify collection-index-record tests/valid/lists-unify; do
                  probe="''${fixture##*/}"
                  # Marshal in pinned CUE, not jq: keep every i64 exact.
                  printf '%s\n' "$(cue export ${../../bootstrap/tests/upstream}/"$probe.cue" --out text)" > expected
                  for binary in bootstrap-executor bootstrap-generated; do
                    if [ "$binary" = bootstrap-executor ]; then
                      "$binary" ${jsonExports pkgs}/"$fixture.json" ${jsonExports pkgs}/evaluation.json > actual
                    else
                      "$binary" ${jsonExports pkgs}/"$fixture.json" > actual
                    fi
                    diff -u expected actual
                  done
                done
                if cue export ${../../bootstrap/tests/upstream/conjunction-conflict.cue} --out json > expected 2> cue-error; then exit 1; fi
                test ! -s expected
                test -s cue-error
                for binary in bootstrap-executor bootstrap-generated; do
                  if [ "$binary" = bootstrap-executor ]; then
                    if "$binary" ${jsonExports pkgs}/conjunction-conflict.json ${jsonExports pkgs}/evaluation.json > actual 2> error; then exit 1; fi
                  else
                    if "$binary" ${jsonExports pkgs}/conjunction-conflict.json > actual 2> error; then exit 1; fi
                  fi
                  test ! -s actual
                  grep -Fx 'conflicting integer values 1 and 2' error
                done
                for pair in list-slicing:range list-slicing:domain optional-slices:range optional-slices:domain; do
                  mode="''${pair#*:}"
                  fixture="''${pair%:*}-$mode"
                  if cue export ${../../bootstrap/tests/upstream}/"$fixture.cue" --out json > expected 2> cue-error; then exit 1; fi
                  test ! -s expected
                  test -s cue-error
                  if [ "$mode" = range ]; then message='list slice out of range'; else message='list slicing requires a concrete closed list and integer endpoints'; fi
                  for binary in bootstrap-executor bootstrap-generated; do
                    if [ "$binary" = bootstrap-executor ]; then
                      if "$binary" ${jsonExports pkgs}/"$fixture.json" ${jsonExports pkgs}/evaluation.json > actual 2> error; then exit 1; fi
                    else
                      if "$binary" ${jsonExports pkgs}/"$fixture.json" > actual 2> error; then exit 1; fi
                    fi
                    test ! -s actual
                    grep -Fx "$message" error
                  done
                done
                for fixture in records-conflict records-fields-conflict nested-records-conflict nested-records-scalar-conflict mixed-conflict; do
                  if cue export ${../../bootstrap/tests/upstream}/"$fixture.cue" --out json > expected 2> cue-error; then exit 1; fi
                  test ! -s expected
                  test -s cue-error
                  grep -E 'conflicting values|field not allowed' cue-error
                  for binary in bootstrap-executor bootstrap-generated; do
                    if [ "$binary" = bootstrap-executor ]; then
                      if "$binary" ${jsonExports pkgs}/"$fixture.json" ${jsonExports pkgs}/evaluation.json > actual 2> error; then exit 1; fi
                    else
                      if "$binary" ${jsonExports pkgs}/"$fixture.json" > actual 2> error; then exit 1; fi
                    fi
                    test ! -s actual
                    grep -Fx 'conflicting closed record values' error
                  done
                done
                for fixture in list-length lists-equal list-indexing collection-index selectors exclusive-bounds records-index records-length records-equal records-logical; do
                  cue export ${../../bootstrap/tests/upstream}/"$fixture.cue" --out json > expected
                  if [ "$fixture" = lists-equal ]; then input=tests/valid/$fixture; else input=$fixture; fi
                  bootstrap-executor ${jsonExports pkgs}/"$input.json" ${jsonExports pkgs}/evaluation.json > actual
                  diff -u expected actual
                  bootstrap-generated ${jsonExports pkgs}/"$input.json" > actual
                  diff -u expected actual
                done
                cue export ${../../bootstrap/tests/upstream/string-length.cue} --out json > expected
                for binary in bootstrap-executor bootstrap-generated; do
                  if [ "$binary" = bootstrap-executor ]; then
                    "$binary" ${jsonExports pkgs}/string-length.json ${jsonExports pkgs}/evaluation.json > actual
                  else
                    "$binary" ${jsonExports pkgs}/string-length.json > actual
                  fi
                  diff -u expected actual
                done
                for pair in concrete:strings empty:tests/valid/strings-empty equal:tests/valid/strings-equal concatenated:concatenation; do
                  property="''${pair%%:*}"
                  fixture="''${pair#*:}"
                  cue export ${../../bootstrap/tests/upstream/strings.cue} -e "$property" --out json | jq -c . > expected
                  for binary in bootstrap-executor bootstrap-generated; do
                    if [ "$binary" = bootstrap-executor ]; then
                      "$binary" ${jsonExports pkgs}/"$fixture.json" ${jsonExports pkgs}/evaluation.json > actual
                    else
                      "$binary" ${jsonExports pkgs}/"$fixture.json" > actual
                    fi
                    # Normalize string spelling only; never round numeric scalars.
                    jq -e 'type == "string"' actual > /dev/null
                    jq -c . actual > normalized
                    diff -u expected normalized
                  done
                done
                for pair in equal:equality notEqual:inequality mixed:tests/valid/equality-mixed; do
                  property="''${pair%%:*}"
                  fixture="''${pair#*:}"
                  cue export ${../../bootstrap/tests/upstream/equality.cue} -e "$property" --out json > expected
                  for binary in bootstrap-executor bootstrap-generated; do
                    if [ "$binary" = bootstrap-executor ]; then
                      "$binary" ${jsonExports pkgs}/"$fixture.json" ${jsonExports pkgs}/evaluation.json > actual
                    else
                      "$binary" ${jsonExports pkgs}/"$fixture.json" > actual
                    fi
                    jq -e 'type == "boolean"' actual > /dev/null
                    diff -u expected actual
                  done
                done
                for pair in integer:ordering text:ordering-text; do
                  property="''${pair%%:*}"
                  fixture="''${pair#*:}"
                  cue export ${../../bootstrap/tests/upstream/ordering.cue} -e "$property" --out json > expected
                  for binary in bootstrap-executor bootstrap-generated; do
                    if [ "$binary" = bootstrap-executor ]; then
                      "$binary" ${jsonExports pkgs}/"$fixture.json" ${jsonExports pkgs}/evaluation.json > actual
                    else
                      "$binary" ${jsonExports pkgs}/"$fixture.json" > actual
                    fi
                    jq -e 'type == "boolean"' actual > /dev/null
                    diff -u expected actual
                  done
                done
                for pair in and:logic-and or:logic-or not:not; do
                  property="''${pair%%:*}"
                  fixture="''${pair#*:}"
                  cue export ${../../bootstrap/tests/upstream/logic.cue} -e "$property" --out json > expected
                  for binary in bootstrap-executor bootstrap-generated; do
                    if [ "$binary" = bootstrap-executor ]; then
                      "$binary" ${jsonExports pkgs}/"$fixture.json" ${jsonExports pkgs}/evaluation.json > actual
                    else
                      "$binary" ${jsonExports pkgs}/"$fixture.json" > actual
                    fi
                    jq -e 'type == "boolean"' actual > /dev/null
                    diff -u expected actual
                  done
                done
                for pair in nullValue:null combined:top-null; do
                  property="''${pair%%:*}"
                  fixture="''${pair#*:}"
                  cue export ${../../bootstrap/tests/upstream/lattice.cue} -e "$property" --out json > expected
                  for binary in bootstrap-executor bootstrap-generated; do
                    if [ "$binary" = bootstrap-executor ]; then
                      "$binary" ${jsonExports pkgs}/"$fixture.json" ${jsonExports pkgs}/evaluation.json > actual
                    else
                      "$binary" ${jsonExports pkgs}/"$fixture.json" > actual
                    fi
                    jq -e 'type == "null"' actual > /dev/null
                    diff -u expected actual
                  done
                done
                if cue export ${../../bootstrap/tests/upstream/lattice.cue} -e top --out json > expected 2> cue-error; then exit 1; fi
                test ! -s expected
                test -s cue-error
                for binary in bootstrap-executor bootstrap-generated; do
                  if [ "$binary" = bootstrap-executor ]; then
                    if "$binary" ${jsonExports pkgs}/top.json ${jsonExports pkgs}/evaluation.json > actual 2> error; then exit 1; fi
                  else
                    if "$binary" ${jsonExports pkgs}/top.json > actual 2> error; then exit 1; fi
                  fi
                  test ! -s actual
                  test -s error
                done
                for fixture in strings-incomplete strings-conflict strings-mixed; do
                  case "$fixture" in
                    strings-incomplete) probe=${../../bootstrap/tests/upstream/strings.cue}; field=abstract ;;
                    strings-conflict) probe=${../../bootstrap/tests/upstream/string-conflict.cue}; field=value ;;
                    strings-mixed) probe=${../../bootstrap/tests/upstream/string-mixed.cue}; field=value ;;
                  esac
                  if cue export "$probe" -e "$field" --out json > expected 2> cue-error; then exit 1; fi
                  test ! -s expected
                  test -s cue-error
                  for binary in bootstrap-executor bootstrap-generated; do
                    if [ "$binary" = bootstrap-executor ]; then
                      if "$binary" ${jsonExports pkgs}/tests/valid/"$fixture.json" ${jsonExports pkgs}/evaluation.json > actual 2> error; then exit 1; fi
                    else
                      if "$binary" ${jsonExports pkgs}/tests/valid/"$fixture.json" > actual 2> error; then exit 1; fi
                    fi
                    test ! -s actual
                    test -s error
                  done
                done
                touch "$out"
              '';
          bootstrap-export = jsonExports pkgs;
        }
      );

      formatter = kolohelios-nix.formatter;
    };
}
