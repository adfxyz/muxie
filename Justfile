# Build PNG icons from the SVG source
icons:
    which resvg >/dev/null 2>&1 || { echo "resvg not found. Install via your package manager or 'cargo install resvg'"; exit 1; }
    resvg --width 32 --height 32 assets/icons/scalable/muxie.svg assets/icons/32x32/muxie.png
    resvg --width 48 --height 48 assets/icons/scalable/muxie.svg assets/icons/48x48/muxie.png
    resvg --width 64 --height 64 assets/icons/scalable/muxie.svg assets/icons/64x64/muxie.png
    resvg --width 96 --height 96 assets/icons/scalable/muxie.svg assets/icons/96x96/muxie.png
    resvg --width 128 --height 128 assets/icons/scalable/muxie.svg assets/icons/128x128/muxie.png
    resvg --width 256 --height 256 assets/icons/scalable/muxie.svg assets/icons/256x256/muxie.png

deb:
    command -v cargo-deb >/dev/null 2>&1 || { echo "Install cargo-deb first (inside dev shell): cargo install cargo-deb"; exit 1; }
    cargo deb --locked --no-default-features --target "${MUSL_TARGET:-x86_64-unknown-linux-musl}"

# Run a containerized smoke test against the built .deb (requires Docker)
test-deb:
    command -v docker >/dev/null 2>&1 || { echo "Docker is not installed or not in PATH"; exit 1; }
    test -n "$(ls -1 target/${MUSL_TARGET:-x86_64-unknown-linux-musl}/debian/*.deb 2>/dev/null)" || { echo "No .deb found. Build first: just deb"; exit 1; }
    docker run --rm \
      -v "$PWD/target/${MUSL_TARGET:-x86_64-unknown-linux-musl}/debian:/debs:ro" \
      -v "$PWD/scripts/smoke:/test:ro" \
      ubuntu:24.04 \
      bash -lc "bash /test/deb-smoke.sh"
