use image::{ImageBuffer, Rgba, RgbaImage};

use crate::qr::encoder::QrMatrix;
use super::profile::StyleProfile;

/// Render a QR matrix to an RGBA image with the given style profile.
pub fn render(matrix: &QrMatrix, profile: &StyleProfile) -> RgbaImage {
    if matrix.is_empty() {
        return RgbaImage::new(1, 1);
    }

    let n = matrix.len() as u32;
    let px = profile.module_px.max(1);
    let qz = profile.quiet_zone;
    let img_size = (n + qz * 2) * px;

    let fg = Rgba(profile.fg_rgba());
    let bg = Rgba(profile.bg_rgba());

    let mut img: RgbaImage = ImageBuffer::from_pixel(img_size, img_size, bg);

    for (row_i, row) in matrix.iter().enumerate() {
        for (col_i, &dark) in row.iter().enumerate() {
            if !dark { continue; }
            let ox = (col_i as u32 + qz) * px;
            let oy = (row_i as u32 + qz) * px;
            for dy in 0..px {
                for dx in 0..px {
                    img.put_pixel(ox + dx, oy + dy, fg);
                }
            }
        }
    }

    // Logo overlay
    if !profile.logo_path.is_empty() && profile.logo_ratio > 0.0 {
        overlay_logo(&mut img, &profile.logo_path, profile.logo_ratio);
    }

    img
}

fn overlay_logo(img: &mut RgbaImage, path: &str, ratio: f32) {
    use image::imageops;

    let logo = match image::open(path) {
        Ok(l) => l.into_rgba8(),
        Err(_) => return,
    };

    let target_px = (img.width() as f32 * ratio.clamp(0.05, 0.30)) as u32;
    if target_px == 0 { return; }

    let logo = imageops::resize(&logo, target_px, target_px, imageops::FilterType::Lanczos3);

    let x = (img.width().saturating_sub(logo.width())) / 2;
    let y = (img.height().saturating_sub(logo.height())) / 2;

    // White padding around logo (4 px)
    let pad = 4u32;
    let px0 = x.saturating_sub(pad);
    let py0 = y.saturating_sub(pad);
    let pw = logo.width() + pad * 2;
    let ph = logo.height() + pad * 2;
    for dy in 0..ph {
        for dx in 0..pw {
            let cx = px0 + dx;
            let cy = py0 + dy;
            if cx < img.width() && cy < img.height() {
                img.put_pixel(cx, cy, Rgba([255, 255, 255, 255]));
            }
        }
    }

    imageops::overlay(img, &logo, x as i64, y as i64);
}

/// Convert a rendered RGBA image to egui ColorImage for texture upload.
pub fn to_egui_image(img: &RgbaImage) -> egui::ColorImage {
    let size = [img.width() as usize, img.height() as usize];
    egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw())
}
