# Browser Demux

A Rust-based URL routing tool that intelligently opens URLs in different browsers based on configurable patterns. This tool acts as a browser demultiplexer - it intercepts URL requests and routes them to specific browsers based on matching patterns.

## What it does

Browser Demux allows you to:
- Set different browsers for different websites or URL patterns
- Automatically route URLs to the appropriate browser based on wildcard patterns
- Install itself as the default system browser to handle all URL requests
- Auto-discover installed browsers from desktop entries

## Usage

```bash
# Install as default browser (required for proper functioning) and create basic configuration.
browser-demux install

# Open a URL (typically called by the system)
browser-demux open https://example.com
```

## Dependencies

`browser-demux install` command needs `xdg-settings` command to be available.

## Configuration

The tool uses a YAML configuration file that maps browsers to URL patterns (`~/.config/browser-demux.yml`):

```yaml
browsers:
  - name: "Firefox"
    executable: "firefox"
    patterns:
      - "*.work.com"
      - "internal.company.net"
  - name: "Chrome"  
    executable: "google-chrome"
    patterns:
      - "github.com"
```
