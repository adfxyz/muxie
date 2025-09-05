{ rustPlatform, lib, pkgs }:
rustPlatform.buildRustPackage {
  pname = "muxie";
  version = "0.4.0";
  cargoHash = "sha256-osM3lkkAXGJqBMxXUa7qM3dTmdYK5rQeydOo6imdjWw";
  src = pkgs.lib.cleanSource ./.;

  # Build without default features to omit self-install commands in packaged builds
  cargoBuildFlags = [ "--no-default-features" ];
  cargoTestFlags = [ "--no-default-features" ];

  # Install desktop entry, icons, and D-Bus activation service into $out
  postInstall = ''
    install -Dm0644 assets/muxie.desktop "$out/share/applications/muxie.desktop"
    for s in 32 48 64 96 128 256; do
      install -Dm0644 assets/icons/''${s}x''${s}/muxie.png "$out/share/icons/hicolor/''${s}x''${s}/apps/muxie.png"
    done
    install -Dm0644 assets/icons/scalable/muxie.svg "$out/share/icons/hicolor/scalable/apps/muxie.svg"
    install -Dm0644 resources/packaging/shared/xyz.adf.Muxie.service \
      "$out/share/dbus-1/services/xyz.adf.Muxie.service"
    substituteInPlace "$out/share/dbus-1/services/xyz.adf.Muxie.service" \
      --replace /usr/bin/muxie "$out/bin/muxie"
  '';

  meta = with lib; {
    description = "Browser demultiplexer that routes URLs to different browsers via patterns";
    homepage = "https://github.com/adfxyz/muxie";
    license = licenses.mit;
    platforms = platforms.linux;
    mainProgram = "muxie";
  };
}
