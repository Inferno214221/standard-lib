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
      in
      {
        devShells.default = with pkgs; mkShell {
          buildInputs = [
            (rust-bin.nightly."${rustVersion}".default.override {
              extensions = [ "rust-src" ];
            })
            pkg-config
            gcc
            cargo-expand
          ];
        };
      }
    );
}
