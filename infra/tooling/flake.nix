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
    {
      devShells = kolohelios-nix.lib.forEachSupportedSystem (
        { system, pkgs, ... }:
        {
          default = pkgs.mkShell {
            packages = [
              pkgs.cue
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
                for fixture in literal addition; do
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
