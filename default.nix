{ rustPlatform, pkgs }:
rustPlatform.buildRustPackage {
  pname = "muxie";
  version = "0.1.1";
  cargoHash = "sha256-CnC+8HnjEmsGYuPVrk/m+maTeFg3qFzMbxZLSXRqzyw=";
  src = pkgs.lib.cleanSource ./.;
}
