use printpdf::{
    ImageXObject, Px, ColorSpace, ColorBits, PdfDocument, Mm, Image, ImageTransform,
};
use crate::qr::encoder::QrMatrix;
use crate::style::{profile::StyleProfile, renderer};

pub fn export(matrix: &QrMatrix, profile: &StyleProfile, path: &str) -> Result<(), String> {
    let img = renderer::render(matrix, profile);
    let width_px = img.width();
    let height_px = img.height();

    // Convert RGBA to RGB for PDF
    let rgb_bytes: Vec<u8> = img
        .pixels()
        .flat_map(|p| [p.0[0], p.0[1], p.0[2]])
        .collect();

    // Page size in mm — assume 96 dpi: 1 px = 25.4/96 mm
    let dpi = 96.0f32;
    let w_mm = width_px as f32 / dpi * 25.4;
    let h_mm = height_px as f32 / dpi * 25.4;

    let (doc, page, layer) =
        PdfDocument::new("QR Code", Mm(w_mm), Mm(h_mm), "Layer 1");
    let layer = doc.get_page(page).get_layer(layer);

    let image_xobj = ImageXObject {
        width: Px(width_px as usize),
        height: Px(height_px as usize),
        color_space: ColorSpace::Rgb,
        bits_per_component: ColorBits::Bit8,
        interpolate: true,
        image_data: rgb_bytes,
        image_filter: None,
        clipping_bbox: None,
        smask: None,
    };

    let pdf_image = Image::from(image_xobj);
    pdf_image.add_to_layer(
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
