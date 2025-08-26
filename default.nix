{ rustPlatform, pkgs }:
rustPlatform.buildRustPackage {
  pname = "muxie";
  version = "0.1.1";
  cargoHash = "sha256-u2JNQujHSntDMKE10nt9ZM43E3fvxnmQTgML7GRGBtE=";
  src = pkgs.lib.cleanSource ./.;
}
