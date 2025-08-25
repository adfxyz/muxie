{ rustPlatform, pkgs }:
rustPlatform.buildRustPackage {
  pname = "muxie";
  version = "0.1.1";
  cargoHash = "sha256-gvJxG1IRKCjeZUDoPHKeL8s05KSaArrDz9AzWvBdlwA=";
  src = pkgs.lib.cleanSource ./.;
}
