{ mkBunDerivation, pkgs, ... }:

mkBunDerivation {
  pname = "openfront-frontend";
  version = "0.1.0";

  src = ./.;

  bunNix = ./bun.nix;

  # Specify runtime dependencies
  buildInputs = [
    pkgs.bun
    pkgs.nodePackages.typescript
  ];

  # Override default build phase to use Vite instead of bun compile
  buildPhase = ''
    runHook preBuild

    # Build the Vite project to create static files
    bun run build

    runHook postBuild
  '';

  # Override default install phase to install static files instead of binary
  installPhase = ''
    runHook preInstall

    mkdir -p $out

    if [ -d "extra_assets" ]; then
      cp -r extra_assets/* $out/
    fi

    # Copy built Vite output
    if [ -d "dist" ]; then
      cp -r dist/* $out/
    fi

    runHook postInstall
  '';

  meta = with pkgs.lib; {
    description = "openfront.pro frontend";
    homepage = "https://github.com/John2143/openfronter";
    license = licenses.mit;
    maintainers = [
        {
            name = "John2143";
            email = "john@john2143.com";
            github = "john2143";
            githubId = 365430;
        }
    ];
    platforms = platforms.all;
  };
}
