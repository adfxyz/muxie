# Build PNG icons from the SVG source
icons:
    which resvg >/dev/null 2>&1 || { echo "resvg not found. Install via your package manager or 'cargo install resvg'"; exit 1; }
    resvg --width 32 --height 32 assets/icons/scalable/muxie.svg assets/icons/32x32/muxie.png
    resvg --width 48 --height 48 assets/icons/scalable/muxie.svg assets/icons/48x48/muxie.png
    resvg --width 64 --height 64 assets/icons/scalable/muxie.svg assets/icons/64x64/muxie.png
    resvg --width 96 --height 96 assets/icons/scalable/muxie.svg assets/icons/96x96/muxie.png
    resvg --width 128 --height 128 assets/icons/scalable/muxie.svg assets/icons/128x128/muxie.png
    resvg --width 256 --height 256 assets/icons/scalable/muxie.svg assets/icons/256x256/muxie.png

