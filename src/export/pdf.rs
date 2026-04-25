use printpdf::{
    ImageXObject, Px, ColorSpace, ColorBits, PdfDocument, Mm, Image, ImageTransform,
};
use crate::qr::encoder::QrMatrix;
use crate::qr::types::EcLevel;
use crate::style::{profile::StyleProfile, renderer};

/// Export a standalone QR code to PDF, rendered at ~300 DPI.
pub fn export(matrix: &QrMatrix, profile: &StyleProfile, ec: EcLevel, path: &str) -> Result<(), String> {
    // Compute module_px needed for ≥ 300 DPI at the same physical size.
    // Physical size = (n + qz*2) * module_px / 96 dpi (canvas dpi)
    // At 300 dpi: module_px_300 = module_px * (300/96)
    let pdf_scale = 300.0_f32 / 96.0;
    let mut hires = profile.clone();
    hires.module_px = ((profile.module_px as f32 * pdf_scale).ceil() as u32).max(profile.module_px);

    let img = renderer::render_ec(matrix, &hires, ec);
    let width_px  = img.width();
    let height_px = img.height();

    // Convert RGBA → RGB (PDF)
    let rgb_bytes: Vec<u8> = img.pixels()
        .flat_map(|p| [p.0[0], p.0[1], p.0[2]])
        .collect();

    // Physical dimensions at 300 DPI (keep same physical size as profile would produce at 96 DPI)
    let w_mm = profile.module_px as f32
        * (matrix.len() as f32 + profile.quiet_zone as f32 * 2.0)
        / 96.0 * 25.4;
    let h_mm = w_mm;

    let (doc, page, layer) =
        PdfDocument::new("QR Code", Mm(w_mm), Mm(h_mm), "Layer 1");
    let layer = doc.get_page(page).get_layer(layer);

    let image_xobj = ImageXObject {
        width:  Px(width_px  as usize),
        height: Px(height_px as usize),
        color_space: ColorSpace::Rgb,
        bits_per_component: ColorBits::Bit8,
        interpolate: false,
        image_data: rgb_bytes,
        image_filter: None,
        clipping_bbox: None,
        smask: None,
    };

    Image::from(image_xobj).add_to_layer(
        layer,
        ImageTransform {
            translate_x: Some(Mm(0.0)),
            translate_y: Some(Mm(0.0)),
            scale_x: Some(w_mm / width_px as f32),
            scale_y: Some(h_mm / height_px as f32),
            ..Default::default()
        },
    );

    let bytes = doc.save_to_bytes().map_err(|e| format!("PDF save: {e}"))?;
    std::fs::write(path, bytes).map_err(|e| format!("PDF write: {e}"))
}

/// Embed pre-rasterized RGBA image (from resvg) into a single-page PDF.
/// `orig_w_px` / `orig_h_px` are the canvas dimensions at 96 DPI (used for physical mm size).
pub fn export_from_rgba(
    rgba:       &[u8],
    width_px:   u32,
    height_px:  u32,
    orig_w_px:  u32,
    orig_h_px:  u32,
    path:       &str,
) -> Result<(), String> {
    // Physical dimensions from the original 96-DPI canvas
    let w_mm = orig_w_px as f32 / 96.0 * 25.4;
    let h_mm = orig_h_px as f32 / 96.0 * 25.4;

    // Convert RGBA → RGB
    let rgb: Vec<u8> = rgba.chunks_exact(4)
        .flat_map(|p| [p[0], p[1], p[2]])
        .collect();

    let (doc, page, layer) =
        PdfDocument::new("Carte QR", Mm(w_mm), Mm(h_mm), "Layer 1");
    let layer = doc.get_page(page).get_layer(layer);

    let xobj = ImageXObject {
        width:  Px(width_px  as usize),
        height: Px(height_px as usize),
        color_space: ColorSpace::Rgb,
        bits_per_component: ColorBits::Bit8,
        interpolate: true,
        image_data: rgb,
        image_filter: None,
        clipping_bbox: None,
        smask: None,
    };

    Image::from(xobj).add_to_layer(
        layer,
        ImageTransform {
            translate_x: Some(Mm(0.0)),
            translate_y: Some(Mm(0.0)),
            scale_x: Some(w_mm / width_px as f32),
            scale_y: Some(h_mm / height_px as f32),
            ..Default::default()
        },
    );

    let bytes = doc.save_to_bytes().map_err(|e| format!("PDF save: {e}"))?;
    std::fs::write(path, bytes).map_err(|e| format!("PDF write: {e}"))
}
