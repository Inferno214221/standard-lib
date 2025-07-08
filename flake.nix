{
  description = "Standard Collections";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-24.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustVersion = "2025-06-18";
        buildInputs = with pkgs; [
          (rust-bin.nightly."${rustVersion}".default.override {
            extensions = [ "rust-src" ];
          })
          pkg-config
          gcc
          cargo-expand
        ];
      in with pkgs; rec
      {
        devShells.default = mkShell {
          inherit buildInputs;
        };

        packages.docs = stdenv.mkDerivation {
          name = "rust-basic-types-doc";
          version = "0.1.0";

          inherit buildInputs;

          src = ./.;

          buildPhase = ''
            cargo rustdoc -- \
              --theme $src/doc/kali-dark.css \
              --html-in-header $src/doc/robots.html \
              --enable-index-page \
              -Z unstable-options
            # Highlight keywords
            find ./target/doc/standard_collections -type f -name "*html" -exec sed -E "s/(>|>([^\">]*[; \[\(])?)(((pub|const|fn|self|Self|struct|enum|type|impl|for|unsafe|as|mut) ?)+)([<& \n:,\)])/\1<span class=\"extra-kw\">\3<\/span>\6/g" -i {} \;
            # Second pass for references and pointers
            find ./target/doc/standard_collections -type f -name "*html" -exec sed -E "s/(>|>([^\">]*[; \[\(]*)?)(mut|const) /\1<span class=\"extra-kw\">\3<\/span> /g" -i {} \;
            # Highlight operators
            find ./target/doc/standard_collections -type f -name "*html" -exec sed -E "s/(>|>([^\">]*[; \[\(\w])?)(&amp;|-&gt;|::|\*)([^/])/\1<span class=\"extra-op\">\3<\/span>\4/g" -i {} \;
            # Where
            find ./target/doc/standard_collections -type f -name "*html" -exec sed -E "s/(<div class=\"where\">)(where)/\1<span class=\"extra-kw\">\2<\/span>/g" -i {} \;
            # TODO: '\w+, mut, <>, (), []
          '';

          installPhase = ''
            mkdir -p $out
            cp -R ./target/doc/* $out/
            cp $src/doc/robots.txt $out/
            cp $src/doc/CNAME $out/
          '';
        };

        apps.docs = {
          type = "app";
          program = "${(
            writeShellScript
              "open-docs"
              "${xdg-utils}/bin/xdg-open ${packages.docs}/standard_collections/index.html"
          )}";
        };
      }
    );
}
