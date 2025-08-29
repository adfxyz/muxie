{ rustPlatform, pkgs }:
rustPlatform.buildRustPackage {
  pname = "muxie";
  version = "0.3.0";
  cargoHash = "sha256-7A9zk2ao3QIWscEIT08lZU8GJDoC8SnxAlinoo4O1p0=";
  src = pkgs.lib.cleanSource ./.;
}
