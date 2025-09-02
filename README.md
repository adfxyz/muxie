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

```bash
# Install as default browser (required for proper functioning) and create basic configuration.
muxie install

# Open a URL (typically called by the system)
muxie open https://example.com

# Uninstall - removes installed assets. Use --restore-default to attempt restoring the previous default browser.
muxie uninstall [--restore-default]
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
