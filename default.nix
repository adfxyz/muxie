{ rustPlatform, pkgs }:
rustPlatform.buildRustPackage {
  pname = "muxie";
  version = "0.1.1";
  cargoHash = "sha256-8q5FDY0tP9LKNZy/GlrUwzVKT+Dll1iplxLL+Os4bvo=";
  src = pkgs.lib.cleanSource ./.;
}
