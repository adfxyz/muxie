#!/usr/bin/env bash
set -euo pipefail

export DEBIAN_FRONTEND=noninteractive

if ! ls /debs/*.deb >/dev/null 2>&1; then
  echo "No .deb files found in /debs"
  exit 2
fi

apt-get update

# Pick the first .deb if multiple exist
debfile="$(ls -1 /debs/*.deb | head -n1)"
echo "Installing package: ${debfile}"

# Preferred: apt-get install handles dependencies automatically
if ! apt-get install -y "$debfile"; then
  echo "apt-get install failed, attempting fallback with dpkg and fix"
  dpkg -i "$debfile" || true
  apt-get -y -f install
  dpkg -i "$debfile"
fi

echo "Verifying installation"
dpkg -s muxie | grep -E 'Package:|Version:|Status:' || {
  echo "muxie not registered after install"
  exit 3
}

echo "Installed files:"
dpkg -L muxie || true

hash -r || true

echo "Running muxie --help"
if ! /usr/bin/muxie --help >/dev/null 2>&1; then
  echo "Failed to execute /usr/bin/muxie"
  exit 4
fi
echo "Binary run check passed"

echo "Debian package smoke test: PASS (installation and files verified)"
