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
          '';
        };

      # Delete only staged build copies, never the working tree. Both stages
      # must succeed without the committed JSON/Rust intermediates.
      clearArtifacts = ''
        for fixture in literal addition evaluation grammar tests/valid/evaluation-left-twice; do
          rm -f "../$fixture.json"
        done
        rm -f ../generated/*.rs
      '';

      codegenSeed =
        pkgs:
        (rustPackage pkgs).overrideAttrs (old: {
          pname = "bootstrap-codegen-seed";
          meta = old.meta // {
            mainProgram = "bootstrap-codegen";
          };
          cargoBuildFlags = [
            "--bin"
            "bootstrap-codegen"
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
            pkgs.cue
            pkgs.jq
            pkgs.rustfmt
            (codegenSeed pkgs)
          ];
          postPatch = clearArtifacts + ''
            # Do not leak strict shell options into later stdenv hooks.
            (
              set -euo pipefail
              export LC_ALL=C TZ=UTC
              for fixture in literal addition evaluation grammar tests/valid/evaluation-left-twice; do
                cue export ../schema.cue "../$fixture.cue" --out json | jq -S . > "../$fixture.json"
              done
              for rules in evaluation.json tests/valid/evaluation-left-twice.json executor/tests/fixtures/codegen-escaped-rules.json; do
                name=$(basename "$rules" .json)
                bootstrap-codegen "../$rules" | rustfmt --edition 2024 > "../generated/$name.rs"
              done
            )
          '';
        });
    in
    {
      packages = kolohelios-nix.lib.forEachSupportedSystem ({ pkgs, ... }: { default = executor pkgs; });

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
                for rules in evaluation.json tests/valid/evaluation-left-twice.json executor/tests/fixtures/codegen-escaped-rules.json; do
                  name=$(basename "$rules" .json)
                  bootstrap-codegen ${../../bootstrap}/"$rules" | rustfmt --edition 2024 > first.rs
                  bootstrap-codegen ${../../bootstrap}/"$rules" | rustfmt --edition 2024 > second.rs
                  cmp first.rs second.rs
                  diff -u ${../../bootstrap/generated}/"$name.rs" first.rs
                done
                touch "$out"
              '';
          bootstrap-source =
            pkgs.runCommand "cuestacean-bootstrap-source"
              {
                nativeBuildInputs = [
                  (executor pkgs)
                  pkgs.cue
                ];
              }
              ''
                set -euo pipefail
                for source in ${../../bootstrap/source}/valid/*.cue; do
                  cue export "$source" --out json > expected
                  bootstrap-source "$source" > actual
                  diff -u expected actual
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
          bootstrap-export =
            pkgs.runCommand "cuestacean-bootstrap-export"
              {
                nativeBuildInputs = [
                  pkgs.cue
                  pkgs.jq
                ];
              }
              ''
                set -euo pipefail
                export LC_ALL=C TZ=UTC
                for fixture in literal addition evaluation grammar tests/valid/evaluation-left-twice; do
                  cue export ${../../bootstrap/schema.cue} ${../../bootstrap}/$fixture.cue --out json | jq -S . > first.json
                  cue export ${../../bootstrap/schema.cue} ${../../bootstrap}/$fixture.cue --out json | jq -S . > second.json
                  cmp first.json second.json
                  diff -u ${../../bootstrap}/$fixture.json first.json
                done
                for fixture in ${../../bootstrap/tests/invalid}/*.cue; do
                  if cue export ${../../bootstrap/schema.cue} "$fixture" --out json > invalid.log 2>&1; then
                    echo "Expected rejection: $fixture" >&2
                    exit 1
                  fi
                done
                touch "$out"
              '';
        }
      );

      formatter = kolohelios-nix.formatter;
    };
}
