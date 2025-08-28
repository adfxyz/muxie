# Changelog

## 0.2.0 — 2025-08-28

Highlights

- Configuration format is now TOML (replacing YAML) and patterns can list multiple browsers in preference order.
- New background daemon with D-Bus activation. `muxie open` talks to the daemon;
- Config auto-reload. Changes to your config are picked up automatically by the daemon.

Changes

- Configuration moved to `~/.config/muxie/muxie.toml` (was `~/.config/muxie.toml`).
- `muxie install` now also installs a D-Bus service file.
- `muxie open` supports `-v/--verbose` for more insight into routing.
- New `muxie config validate` to check your configuration (strict validation of executables).
