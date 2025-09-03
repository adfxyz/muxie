{
  description = "A browser router that intelligently opens URLs in different browsers based on configurable wildcard patterns.";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs =
    { self, nixpkgs, rust-overlay }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      devShells = forAllSystems (system: {
        default =
          let
            pkgs = import nixpkgs {
              inherit system;
              overlays = [ rust-overlay.overlays.default ];
            };
            muslTarget = if system == "x86_64-linux" then "x86_64-unknown-linux-musl"
                         else if system == "aarch64-linux" then "aarch64-unknown-linux-musl"
                         else throw "Unsupported system: ${system}";
            toolchain = pkgs.rust-bin.stable.latest.default.override {
              targets = [ muslTarget ];
              extensions = [ "rust-src" "clippy" "rustfmt" ];
            };
          in pkgs.mkShell {
            buildInputs = [
              toolchain
              pkgs.just
              pkgs.cargo-deb
              pkgs.resvg
            ];
            # Ensure consistent static builds when targeting musl
            RUSTFLAGS = "-C target-feature=+crt-static";
            MUSL_TARGET = muslTarget;
          };
      });
      packages = forAllSystems (system: {
        default = (import nixpkgs { inherit system; }).callPackage ./default.nix { };
      });
    };
}
