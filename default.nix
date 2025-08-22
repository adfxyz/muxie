{ rustPlatform, pkgs }:
rustPlatform.buildRustPackage {
  pname = "muxie";
  version = "0.1.1";
  cargoHash = "sha256-afMEEs+LSWlE0oGrDlUhn9exteuw3pU29+FwUrlAq0U=";
  src = pkgs.lib.cleanSource ./.;
}
