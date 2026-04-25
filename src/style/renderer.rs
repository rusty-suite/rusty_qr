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
    use std::sync::{Arc, Mutex, OnceLock};

    // Cache the raw logo so we only hit the disk when the path changes.
    static LOGO_CACHE: OnceLock<Mutex<(String, Option<Arc<RgbaImage>>)>> = OnceLock::new();
    let cache = LOGO_CACHE.get_or_init(|| Mutex::new((String::new(), None)));

    let logo_arc: Option<Arc<RgbaImage>> = {
        let mut c = cache.lock().unwrap_or_else(|e| e.into_inner());
        if c.0 != profile.logo_path {
            c.0 = profile.logo_path.clone();
            c.1 = image::open(&profile.logo_path)
                .ok()
                .map(|l| Arc::new(l.into_rgba8()));
        }
        c.1.clone()
    };

    let logo_src = match logo_arc.as_deref() {
        Some(l) => l,
        None    => return,
    };

    // Plafond strict basé sur le niveau EC + limite absolue de sécurité
    let effective = profile.logo_ratio.clamp(0.01, logo_cap.clamp(0.0, 0.30));
    let target = (img.width() as f32 * effective) as u32;
    if target == 0 { return; }

    // Resize to target bbox, then trim transparent borders so that
    // positioning and the white background are based on the *visible* content,
    // not the full PNG bounding box (which may have large transparent margins).
    let logo_resized = imageops::resize(logo_src, target, target, imageops::FilterType::Lanczos3);
    let logo = trim_transparent(&logo_resized);

    // Position the visible content according to the profile
    let max_x = img.width().saturating_sub(logo.width());
    let max_y = img.height().saturating_sub(logo.height());
    let lx = (profile.logo_pos_x.clamp(0.0, 1.0) * max_x as f32) as u32;
    let ly = (profile.logo_pos_y.clamp(0.0, 1.0) * max_y as f32) as u32;

    // White background sized to the visible content only (pad == 0 → no background)
    let pad = profile.logo_padding;
    if pad > 0 {
        let px0 = lx.saturating_sub(pad);
        let py0 = ly.saturating_sub(pad);
        let pw  = (logo.width()  + pad * 2).min(img.width().saturating_sub(px0));
        let ph  = (logo.height() + pad * 2).min(img.height().saturating_sub(py0));
        for dy in 0..ph {
            for dx in 0..pw {
                img.put_pixel(px0 + dx, py0 + dy, Rgba([255, 255, 255, 255]));
            }
        }
    }

    imageops::overlay(img, &logo, lx as i64, ly as i64);
}

/// Crop an RGBA image to the tight bounding box of its non-transparent pixels.
/// Returns the original image unchanged if it is fully transparent or empty.
fn trim_transparent(img: &RgbaImage) -> RgbaImage {
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 { return img.clone(); }

    let mut x0 = w; let mut x1 = 0u32;
    let mut y0 = h; let mut y1 = 0u32;

    for y in 0..h {
        for x in 0..w {
            if img.get_pixel(x, y)[3] > 8 {
                if x < x0 { x0 = x; }
                if x > x1 { x1 = x; }
                if y < y0 { y0 = y; }
                if y > y1 { y1 = y; }
            }
        }
    }

    if x0 > x1 { return img.clone(); } // fully transparent

    let cw = x1 - x0 + 1;
    let ch = y1 - y0 + 1;
    image::imageops::crop_imm(img, x0, y0, cw, ch).to_image()
}

pub fn to_egui_image(img: &RgbaImage) -> egui::ColorImage {
    let size = [img.width() as usize, img.height() as usize];
    egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw())
}
