# Muxie

A Rust-based URL routing tool that intelligently opens URLs in different browsers based on configurable patterns. This
tool acts as a browser demultiplexer - it intercepts URL requests and routes them to specific browsers based on matching
patterns.

## What it does

Muxie allows you to:

- Set different browsers for different websites or URL patterns
- Automatically route URLs to the appropriate browser based on wildcard patterns
- Install itself as the default system browser to handle all URL requests
- Auto-discover installed browsers from desktop entries

## Usage

Packaged installation (recommended):

- Install Muxie via your distribution’s package manager.
- Set Muxie as the default browser in your desktop environment’s settings (e.g., GNOME/KDE).
- Optional: create a default configuration file if missing:

```bash
muxie config create
```
- Optional sanity check:

```bash
# Typically not needed: the system calls this for you
muxie open https://example.com
```

Standalone binary (installed from source or `cargo install`)
- Use this only if you are not using a distribution package.
- Run `muxie install` once to register handlers and create a basic configuration. This command does the following:
  - Installs the application icons and the `.desktop` file.
  - Install the D-Bus service for running the `muxie` daemon.
  - Sets `muxie` as your default browser (this requires `xdg-settings` command to available in your system).
- Use `muxie uninstall` to remove the installed files.

```bash
# Install as default browser and create basic configuration
muxie install

# Open a URL (typically called by the system)
muxie open https://example.com

# Uninstall - removes installed assets. Use --restore-default to attempt restoring the previous default browser.
muxie uninstall [--restore-default]
```

Configuration commands:

```bash
# Create a default configuration file if missing
muxie config create

# Validate configuration strictly (checks for executables and dialog providers)
muxie config validate
```

### Graphical Selection Prompt

When a matched pattern lists two or more eligible browsers, Muxie shows a native selection dialog (if a GUI provider is
available) and asks which browser to use for this URL.

- Providers: auto-detected in this order: `kdialog` (KDE), `zenity` (GNOME), `yad`.
- Displayed text uses a redacted URL (host only), never the full URL.
- Cancel behavior: if you press Cancel or close the dialog, Muxie aborts opening the URL.
- Headless or no provider: no prompt is shown; Muxie proceeds non-interactively as before.
- Error handling: if the selected browser fails to start, Muxie tries the remaining browsers for that pattern in the
  configured order (no re-prompt).

## Dependencies

`muxie install` command needs `xdg-settings` command to be available.

## Configuration

The tool uses a TOML configuration file at `~/.config/muxie/muxie.toml` with separate browser definitions and routing
patterns that map to browser names:

```toml
version = 1

[[browsers]]
name = "Firefox"
executable = "firefox"
args = ["%u"]

[[browsers]]
name = "Chrome"
executable = "google-chrome"

[[patterns]]
pattern = "*.work.com"
browsers = ["Firefox"]

[[patterns]]
pattern = "github.com"
browsers = ["Chrome", "Firefox"]

[notifications]
enabled = true
redact_urls = true

[dialog]
# Dialog provider for selection prompts: one of "auto", "kdialog", "zenity", "yad"
# Default is "auto". When set to a specific provider, Muxie will use only that
# provider (no fallback).
provider = "auto"
```

## Build Packages (for maintainers)

- Build Debian package:
  - Install: `cargo install cargo-deb`
  - Build: `just deb`
  - Inspect: `dpkg-deb -c target/${MUSL_TARGET:-x86_64-unknown-linux-musl}/debian/*.deb`

- Build RPM package:
  - Install: `cargo install cargo-generate-rpm`
  - Build: `just rpm`
  - Inspect: `rpm -qpi target/generate-rpm/*.rpm && rpm -qpl target/generate-rpm/*.rpm`

Container smoke tests (optional, requires Docker):
- Debian/Ubuntu: `just test-deb`
- Fedora: `just test-rpm`
