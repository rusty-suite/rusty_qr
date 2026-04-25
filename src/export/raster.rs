use crate::qr::encoder::QrMatrix;
use crate::style::{profile::StyleProfile, renderer};

pub fn export_png(matrix: &QrMatrix, profile: &StyleProfile, path: &str) -> Result<(), String> {
    let img = renderer::render(matrix, profile);
    img.save(path).map_err(|e| format!("PNG: {e}"))
}

pub fn export_jpg(matrix: &QrMatrix, profile: &StyleProfile, path: &str) -> Result<(), String> {
    use image::DynamicImage;

    let img = renderer::render(matrix, profile);
    // Convert to RGB (JPEG doesn't support alpha)
    let rgb = DynamicImage::ImageRgba8(img).into_rgb8();
    rgb.save(path).map_err(|e| format!("JPEG: {e}"))
}
