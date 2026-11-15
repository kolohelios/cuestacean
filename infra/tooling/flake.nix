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
      executor =
        pkgs:
        pkgs.rustPlatform.buildRustPackage {
          pname = "bootstrap-executor";
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
    in
    {
      packages = kolohelios-nix.lib.forEachSupportedSystem ({ pkgs, ... }: { default = executor pkgs; });

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
                for fixture in literal addition evaluation tests/valid/evaluation-left-twice; do
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
