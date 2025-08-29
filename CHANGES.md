# Changelog

## 0.3.0 — 2025-08-30

Highlights

- Graphical browser selection dialog when a matched pattern lists 2+ browsers. Uses native system dialogs.
- Configurable dialog provider via `[dialog] provider = "auto|kdialog|zenity|yad"` (default `auto`). When set to a
  specific provider, Muxie uses only that provider (no fallback).

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
