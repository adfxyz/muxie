use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/assets/"]
pub struct Asset;

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/assets/icons"]
pub struct Icon;
