#!/usr/bin/env bash
set -euo pipefail

echo "==> Nix info"
nix --version || true

echo "==> Building flake package .#muxie"
nix --extra-experimental-features 'nix-command flakes' build .#muxie

if [ ! -e result ]; then
  echo "result symlink not found after nix build" >&2
  exit 2
fi

echo "==> Validating installed assets under result/"
test -f result/share/applications/muxie.desktop || { echo "Missing desktop entry" >&2; exit 3; }
for s in 32 48 64 96 128 256; do
  test -f "result/share/icons/hicolor/${s}x${s}/apps/muxie.png" || {
    echo "Missing icon ${s}x${s}" >&2; exit 4;
  }
done
test -f result/share/icons/hicolor/scalable/apps/muxie.svg || { echo "Missing scalable icon" >&2; exit 5; }

svc="result/share/dbus-1/services/xyz.adf.Muxie.service"
test -f "$svc" || { echo "Missing D-Bus service file" >&2; exit 6; }

echo "==> Checking Exec path in D-Bus service"
exec_line=$(grep -E '^Exec=' "$svc" || true)
if [ -z "$exec_line" ]; then
  echo "Exec line not found in service file" >&2
  exit 7
fi
cmd_path=$(printf '%s' "$exec_line" | sed -E 's/^Exec=([^ ]+).*/\1/')
case "$cmd_path" in
  */bin/muxie) : ;; # ok
  *) echo "Unexpected Exec path: $cmd_path" >&2; exit 8 ;;
esac
test -x "$cmd_path" || { echo "Exec path not executable: $cmd_path" >&2; exit 9; }

echo "==> Running nix run .#muxie -- --help"
nix --extra-experimental-features 'nix-command flakes' run .#muxie -- --help >/dev/null 2>&1 || {
  echo "nix run failed" >&2
  exit 10
}

echo "Nix packaging validation: PASS"
