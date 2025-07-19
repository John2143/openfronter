{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    bun2nix.url = "github:baileyluTCD/bun2nix?tag=1.5.1";
  };

  outputs = inputs:
    inputs.flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import (inputs.nixpkgs) { inherit system; });
        bun2nix = inputs.bun2nix;
      in rec {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.bun
            pkgs.nodePackages.typescript
            pkgs.nodePackages.typescript-language-server
            pkgs.nodePackages.eslint
            # Add the bun2nix binary to our devshell
            bun2nix.packages.${system}.default
          ];

          shellHook = ''
            echo "Frontend development shell"
            echo "Run 'bun install' to install dependencies"
          '';
        };

        packages.default = pkgs.callPackage ./. {
          inherit (bun2nix.lib.${system}) mkBunDerivation;
          inherit pkgs;
        };

        packages.frontend = packages.default;
      }
    );
}
