{
  description = "Standard Lib";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
    
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    naersk-pkg = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, fenix, naersk-pkg, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ fenix.overlays.default ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        toolchain = fenix.packages.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-P39FCgpfDT04989+ZTNEdM/k/AE869JKSB4qjatYTSs=";
        };
        naersk = pkgs.callPackage naersk-pkg {
          cargo = toolchain;
          rustc = toolchain;
        };
        buildInputs = with pkgs; [];
        nativeBuildInputs = with pkgs; [
          toolchain
          pkg-config
          gcc
          cargo-expand
          cargo-public-api
          man-pages
          rust-analyzer-nightly
        ] ++ buildInputs;
      in with pkgs; rec
      {
        devShells.default = mkShell {
          inherit nativeBuildInputs;
        };

        packages.docs-raw = (naersk.buildPackage rec {
          src = ./.;

          inherit nativeBuildInputs;

          mode = "check";
          doDoc = true;
          doDocFail = true;
          cargoDocCommands = old: [
            ''
              cargo $cargo_options rustdoc -- \
                --theme ${src}/doc/kali-dark.css \
                --html-in-header ${./doc/robots.html} \
                --enable-index-page \
                -Z unstable-options
            # ''
            # ^ This is really bad but we just use a bash open comment here so that eval doesn't
            # complain about "unexpected syntax token near '||'". We need doDocFail anyway, so the
            # generated code is `|| false`. (No effect)
          ];

          postInstall = ''
            cp $src/doc/robots.txt $out/
            cp $src/doc/CNAME $out/
          '';
        }).doc;

        packages.docs = stdenv.mkDerivation {
          name = "standard-lib-doc";
          version = "0.1.0";
          src = "${packages.docs-raw}";

          buildPhase = ''
            # Highlight keywords
            find ./standard_lib -type f -name "*html" -exec sed -E "s/(>|>([^\">]*[; \[\(])?)(((pub|const|fn|self|Self|struct|enum|type|impl|for|unsafe|as|mut) ?)+)([<& \n:,\)])/\1<span class=\"extra-kw\">\3<\/span>\6/g" -i {} \;
            # Second pass for references and pointers
            find ./standard_lib -type f -name "*html" -exec sed -E "s/(>|>([^\">]*[; \[\(]*)?)(mut|const) /\1<span class=\"extra-kw\">\3<\/span> /g" -i {} \;
            # Highlight operators
            find ./standard_lib -type f -name "*html" -exec sed -E "s/(>|>([^\">]*[; \[\(\w])?)(&amp;|-&gt;|::|\*)([^/])/\1<span class=\"extra-op\">\3<\/span>\4/g" -i {} \;
            # Where
            find ./standard_lib -type f -name "*html" -exec sed -E "s/(<div class=\"where\">)(where)/\1<span class=\"extra-kw\">\2<\/span>/g" -i {} \;
            # TODO: '\w+, mut, <>, (), []
          '';

          installPhase = ''
            mkdir -p $out
            cp -R ./* $out/
          '';
        };

        apps.docs = {
          type = "app";
          program = "${(
            writeShellScript
              "open-docs"
              "${xdg-utils}/bin/xdg-open ${packages.docs}/standard_lib/index.html"
          )}";
        };
      }
    );
}
