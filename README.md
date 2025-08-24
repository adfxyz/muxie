# Muxie

A Rust-based URL routing tool that intelligently opens URLs in different browsers based on configurable patterns. This tool acts as a browser demultiplexer - it intercepts URL requests and routes them to specific browsers based on matching patterns.

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

# Uninstall - this removes the installed assets and restores the previous default browser.
muxie uninstall
```

## Dependencies

`muxie install` command needs `xdg-settings` command to be available.

## Configuration

The tool uses a TOML configuration file at `~/.config/muxie.toml` with separate browser definitions and routing patterns that map to browser names:

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
```
