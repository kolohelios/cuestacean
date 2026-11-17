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
        "identifier"
        "string-literal"
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
        "ast-unary"
        "tests/valid/ast-program-v2"
        "tests/valid/ast-escaped"
        "tests/valid/ast-duplicate"
      ];
      builderNames = builtins.concatStringsSep " " builderFixtures;
      prattFixtures = [
        "primary"
        "tests/valid/primary-no-atoms"
        "tests/valid/primary-high-minimum"
        "tests/valid/primary-low-negation"
        "tests/valid/primary-cross-phase"
        "pratt"
        "tests/valid/pratt-right"
        "tests/valid/pratt-stalled"
        "tests/valid/pratt-invalid-state"
        "tests/valid/pratt-overflow"
      ];
      prattNames = builtins.concatStringsSep " " prattFixtures;
      unifyFixtures = [
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
        "negate-control"
        "difference-control"
        "product-control"
        "tests/valid/not-true"
        "tests/valid/control-default"
        "tests/valid/unary-conditions"
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
        "booleans"
        "strings"
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
        "tests/valid/evaluation-left-twice"
        "tests/valid/pratt-lowering-no-increment"
        "tests/valid/unify-lowering-permissive"
        "tests/valid/integer-kernel-alternate"
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
                ../../bootstrap/scanner-fragments.cue
                ../../bootstrap/equality-fragments.cue
                ../../bootstrap/ordering-fragments.cue
                ../../bootstrap/logic-fragments.cue
                ../../bootstrap/integer-control-fragments.cue
                ../../bootstrap/lowering-fragments.cue
                ../../bootstrap/pratt-lowering-fragments.cue
                ../../bootstrap/unify-lowering-fragments.cue
                ../../bootstrap/ast-lowering-fragments.cue
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
              # Load only the fixture's definition closure. Loading every compiler
              # for a literal can exhaust the CI runner while checking unused CUE.
              fragments=()
              case "$fixture" in
                whitespace|after-operand|after-word-operator|group-close)
                  fragments=(scanner-fragments) ;;
                lowering|tests/valid/lowering-left-twice|tests/valid/lowering-reverse)
                  fragments=(lowering-fragments) ;;
                pratt-lowering|tests/valid/pratt-lowering-*) fragments=(pratt-lowering-fragments) ;;
                unify-lowering|tests/valid/unify-lowering-*|tests/valid/integer-kernel-alternate) fragments=(unify-lowering-fragments) ;;
                ast-lowering|tests/valid/ast-lowering-*) fragments=(ast-lowering-fragments) ;;
                scan-lowering|tests/valid/scan-lowering-*) fragments=(scan-lowering-fragments) ;;
                meta-lowering|tests/valid/meta-lowering-*|tests/valid/meta-driver-previous)
                  fragments=(meta-lowering-fragments) ;;
                equal|not-equal) fragments=(equality-fragments) ;;
                less|less-equal|greater|greater-equal) fragments=(ordering-fragments) ;;
                logical-and|logical-or) fragments=(equality-fragments logic-fragments) ;;
                difference-control|product-control) fragments=(integer-control-fragments) ;;
              esac
              inputs=(${cueSources}/schema.cue)
              for fragment in "''${fragments[@]}"; do inputs+=("${cueSources}/$fragment.cue"); done
              inputs+=("${cueSources}/$fixture.cue")
              cue export "''${inputs[@]}" --out json | jq -S . > first.json
              cue export "''${inputs[@]}" --out json | jq -S . > second.json
              cmp first.json second.json
              mkdir -p "$(dirname "$out/$fixture.json")"
              cp first.json "$out/$fixture.json"
            done
            for fixture in ${cueSources}/tests/invalid/*.cue; do
              if cue export ${cueSources}/schema.cue "$fixture" --out json > invalid.log 2>&1; then
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
          nativeCheckInputs = [
            pkgs.rustfmt
            pkgs.clippy
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
          '';
        };

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
            )
          '';
        });
    in
    {
      packages = kolohelios-nix.lib.forEachSupportedSystem (
        { pkgs, ... }: {
          default = executor pkgs;
          exports = jsonExports pkgs;
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
            packages = [
              pkgs.cue
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
                for source in ${../../bootstrap/source}/valid/*.cue; do
                  cue export "$source" --out json > expected
                  # CUE can spell integer zero as -0; i64 output is canonical 0.
                  # Normalize only that spelling, without rounding any integers.
                  IFS= read -r scalar < expected
                  if [ "$scalar" = "-0" ]; then printf '0\n' > expected; fi
                  bootstrap-source "$source" > actual
                  case "$scalar" in
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
                # These rejected operand domains must also fail upstream.
                for source in ${../../bootstrap/source}/rejected/ordering-{boolean,mixed}.cue ${../../bootstrap/source}/rejected/logic-{left,selected,eager,or-eager}.cue ${../../bootstrap/source}/rejected/not-integer.cue ${../../bootstrap/source}/rejected/null-conflict.cue; do
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
