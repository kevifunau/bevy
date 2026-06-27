use std::path::Path;

use super::extract::SvgAssetEntry;

pub(crate) fn rasterize_svg_to_png(
    svg_markup: &str,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, String> {
    let tree = usvg::Tree::from_str(svg_markup, &usvg::Options::default())
        .map_err(|e| format!("Failed to parse SVG for rasterization: {e}"))?;

    let mut pixmap = tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| format!("Failed to create pixmap ({width}x{height})"))?;

    resvg::render(&tree, tiny_skia::Transform::identity(), &mut pixmap.as_mut());

    pixmap
        .encode_png()
        .map_err(|e| format!("Failed to encode PNG: {e}"))
}

pub(crate) fn write_svg_png_asset(
    base_dir: &Path,
    key: &str,
    png_bytes: &[u8],
) -> Result<String, String> {
    let png_dir = base_dir.join("assets").join("png");
    std::fs::create_dir_all(&png_dir)
        .map_err(|e| format!("Failed to create PNG directory {}: {e}", png_dir.display()))?;

    let png_path = png_dir.join(format!("{key}.png"));
    std::fs::write(&png_path, png_bytes)
        .map_err(|e| format!("Failed to write PNG {}: {e}", png_path.display()))?;

    Ok(format!("assets/png/{key}.png"))
}

pub(crate) fn rasterize_svg_assets(
    assets: &[SvgAssetEntry],
    base_dir: &Path,
) -> Result<(), String> {
    for entry in assets {
        let png_bytes = rasterize_svg_to_png(
            &entry.svg_markup,
            entry.render_width,
            entry.render_height,
        )?;
        write_svg_png_asset(base_dir, &entry.key, &png_bytes)?;
    }
    Ok(())
}
