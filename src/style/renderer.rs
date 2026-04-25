use image::{ImageBuffer, Rgba, RgbaImage};
use crate::qr::encoder::QrMatrix;
use crate::qr::types::EcLevel;
use super::profile::StyleProfile;

/// Ratio maximum du logo selon le niveau de correction d'erreur.
/// Au-delà de cette limite, le QR code risque de ne plus être lisible.
pub fn max_logo_ratio(ec: EcLevel) -> f32 {
    match ec {
        EcLevel::L => 0.07, // EC L restaure 7 % de données
        EcLevel::M => 0.15, // EC M restaure 15 %
        EcLevel::Q => 0.25, // EC Q restaure 25 %
        EcLevel::H => 0.30, // EC H restaure 30 % (max pratique)
    }
}

/// Rendu standard — plafond logo par défaut (30 %, sûr pour EC H).
pub fn render(matrix: &QrMatrix, profile: &StyleProfile) -> RgbaImage {
    render_capped(matrix, profile, 0.30)
}

/// Rendu avec plafond logo explicite basé sur le niveau EC.
pub fn render_ec(matrix: &QrMatrix, profile: &StyleProfile, ec: EcLevel) -> RgbaImage {
    render_capped(matrix, profile, max_logo_ratio(ec))
}

/// Rendu avec un plafond logo personnalisé (0.0–1.0).
pub fn render_capped(matrix: &QrMatrix, profile: &StyleProfile, logo_cap: f32) -> RgbaImage {
    if matrix.is_empty() { return RgbaImage::new(1, 1); }

    let n  = matrix.len() as u32;
    let px = profile.module_px.max(1);
    let qz = profile.quiet_zone;
    let sz = (n + qz * 2) * px;

    let fg = Rgba(profile.fg_rgba());
    let bg = Rgba(profile.bg_rgba());

    let mut img: RgbaImage = ImageBuffer::from_pixel(sz, sz, bg);

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

    if profile.has_logo() {
        overlay_logo(&mut img, profile, logo_cap);
    }
    img
}

fn overlay_logo(img: &mut RgbaImage, profile: &StyleProfile, logo_cap: f32) {
    use image::imageops;

    let logo = match image::open(&profile.logo_path) {
        Ok(l)  => l.into_rgba8(),
        Err(_) => return,
    };

    // Plafond strict basé sur le niveau EC + limite absolue de sécurité
    let effective = profile.logo_ratio.clamp(0.01, logo_cap.clamp(0.0, 0.30));
    let target = (img.width() as f32 * effective) as u32;
    if target == 0 { return; }

    let logo = imageops::resize(&logo, target, target, imageops::FilterType::Lanczos3);

    let max_x = img.width().saturating_sub(logo.width());
    let max_y = img.height().saturating_sub(logo.height());
    let lx = (profile.logo_pos_x.clamp(0.0, 1.0) * max_x as f32) as u32;
    let ly = (profile.logo_pos_y.clamp(0.0, 1.0) * max_y as f32) as u32;

    // Fond blanc autour du logo
    let pad = profile.logo_padding;
    let px0 = lx.saturating_sub(pad);
    let py0 = ly.saturating_sub(pad);
    let pw  = logo.width()  + pad * 2;
    let ph  = logo.height() + pad * 2;
    for dy in 0..ph {
        for dx in 0..pw {
            let cx = px0 + dx;
            let cy = py0 + dy;
            if cx < img.width() && cy < img.height() {
                img.put_pixel(cx, cy, Rgba([255, 255, 255, 255]));
            }
        }
    }

    imageops::overlay(img, &logo, lx as i64, ly as i64);
}

pub fn to_egui_image(img: &RgbaImage) -> egui::ColorImage {
    let size = [img.width() as usize, img.height() as usize];
    egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw())
}
