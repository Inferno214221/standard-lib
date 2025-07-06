{
  description = "Basic Types";

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
      in with pkgs;
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
            cargo rustdoc -- --theme $src/kali-dark.css
          '';

          installPhase = ''
            mkdir -p $out
            cp -R ./target/doc $out
          '';
        };
      }
    );
}
