{ rustPlatform, pkgs }:
rustPlatform.buildRustPackage {
  pname = "muxie";
  version = "0.2.0";
  cargoHash = "sha256-JWce2mSS/2ZG4jPypVJ2FCGdQWRsbPMr2pOPz762Br4=";
  src = pkgs.lib.cleanSource ./.;
}
