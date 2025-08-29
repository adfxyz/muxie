{ rustPlatform, pkgs }:
rustPlatform.buildRustPackage {
  pname = "muxie";
  version = "0.2.0";
  cargoHash = "sha256-6pnBZZsVs7Lv+PQ1TgkioXV3uyYCJJNEzVCy4P4koTQ=";
  src = pkgs.lib.cleanSource ./.;
}
