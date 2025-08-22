{
  description = "A browser router that intelligently opens URLs in different browsers based on configurable wildcard patterns.";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };
  outputs =
    { self, nixpkgs }:
    let
      supportedSystems = [ "x86_64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      pkgs = nixpkgs.legacyPackages;
    in
    {
      devShells = forAllSystems (system: {
        default = pkgs.${system}.mkShell {
          buildInputs = with pkgs.${system}; [
            cargo
            rustc
            rustfmt
            clippy
            rust-analyzer
          ];
          env.RUST_SRC_PATH = "${pkgs.${system}.rust.packages.stable.rustPlatform.rustLibSrc}";
        };
      });
      packages = forAllSystems (system: {
        default = pkgs.${system}.callPackage ./default.nix { };
      });
    };
}
