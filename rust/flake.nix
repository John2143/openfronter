{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    frontend = {
      url = "path:./frontend";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, utils, naersk, frontend }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
      in
      rec {
        defaultPackage = packages.bundle;
        devShell = with pkgs; mkShell {
          buildInputs = [ cargo rustc rustfmt pre-commit rustPackages.clippy rust-analyzer sqlx-cli bacon];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };

        packages.openfrontpro = naersk-lib.buildPackage {
          src = ./.;
          pname = "openfrontpro";
        };

        packages.openfront-frontend = frontend.outputs.packages.${system}.frontend;

        # This exists to act like the docker image, except only as a nix executable.
        # It only has two inputs, packages.openfrontpro and packages.openfront-frontend.
        packages.bundle = pkgs.stdenv.mkDerivation {
          name = "openfrontpro";
            inherit (packages.openfrontpro) version src;
            buildInputs = [
                packages.openfrontpro
                packages.openfront-frontend
                pkgs.cacert
                pkgs.bashInteractive
                pkgs.coreutils
                pkgs.curl
            ];
            nativeBuildInputs = [ pkgs.makeWrapper ];
            installPhase = ''
              mkdir -p $out/bin
              cp ${packages.openfrontpro}/bin/openfrontpro $out/bin/
              wrapProgram $out/bin/openfrontpro \
                --set FRONTEND_FOLDER ${packages.openfront-frontend} \
                --set RUST_LOG info
            '';
        };

        packages.container = pkgs.dockerTools.buildLayeredImage {
          name = "openfrontpro";
          contents = [
            packages.openfrontpro
            packages.openfront-frontend
            pkgs.cacert
            pkgs.bashInteractive
            pkgs.coreutils
            pkgs.curl
            #pkgs.glibcLocales
            #pkgs.openssl
            #pkgs.zlib
          ];

          config = {
            ExposedPorts = { "3000/tcp" = { }; };
            EntryPoint = [ "${packages.openfrontpro}/bin/openfrontpro" ];
            Env = [
              "RUST_LOG=info"
              "FRONTEND_FOLDER=${packages.openfront-frontend}"
            ];
            #Cmd = [ "openfrontpro" ];
          };
        };
      }
    );
}
