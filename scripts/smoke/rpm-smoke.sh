#!/usr/bin/env bash
set -euo pipefail

if ! ls /rpms/*.rpm >/dev/null 2>&1; then
  echo "No .rpm files found in /rpms"
  exit 2
fi

rpmfile="$(ls -1 /rpms/*.rpm | head -n1)"
echo "==> Installing package: ${rpmfile}"

install_with_dnf() {
  # Prefer dnf when available (Fedora/RHEL/CentOS Stream)
  if command -v dnf >/dev/null 2>&1; then
    dnf -y install "/rpms/"*.rpm && return 0
  fi
  return 1
}

install_with_yum() {
  if command -v yum >/dev/null 2>&1; then
    yum -y install "/rpms/"*.rpm && return 0
  fi
  return 1
}

install_with_zypper() {
  if command -v zypper >/dev/null 2>&1; then
    zypper -n install --allow-unsigned-rpm "/rpms/"*.rpm && return 0
  fi
  return 1
}

fallback_with_rpm() {
  rpm -Uvh --replacepkgs "/rpms/"*.rpm
}

if ! install_with_dnf && ! install_with_yum && ! install_with_zypper; then
  echo "==> Package manager not found or install failed, trying plain rpm"
  fallback_with_rpm || {
    echo "Failed to install RPM"
    exit 3
  }
fi

echo "==> Verifying installation"
rpm -q muxie || {
  echo "muxie not registered after install"
  exit 4
}

echo "==> Installed files:"
rpm -ql muxie || true

hash -r || true

echo "==> Running muxie --help"
if ! /usr/bin/muxie --help >/dev/null 2>&1; then
  echo "Failed to execute /usr/bin/muxie"
  exit 5
fi
echo "==> Binary run check passed"

echo "==> Checking that install/uninstall subcommands are not available (packaged build)"
if /usr/bin/muxie install --help >/dev/null 2>&1; then
  echo "install subcommand unexpectedly available in packaged build" >&2
  exit 6
fi
if /usr/bin/muxie uninstall --help >/dev/null 2>&1; then
  echo "uninstall subcommand unexpectedly available in packaged build" >&2
  exit 7
fi

echo "RPM package smoke test: PASS (installation and files verified)"
