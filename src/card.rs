//! Concepteur de cartes — mise en page QR (carte de visite, étiquette, badge, flyer).

use serde::{Deserialize, Serialize};
use crate::qr::encoder::QrMatrix;
use crate::style::{profile::StyleProfile, renderer};

// ─── Types ───────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub enum CardLayout {
    BusinessCard, // 85 × 54 mm  — QR gauche, texte droite
    Label,        // 60 × 60 mm  — QR centré, texte dessous
    Badge,        // 90 × 60 mm  — QR + Nom + Titre, bande d'accent haut
    Flyer,        // 148 × 105 mm (A6) — grand QR + descriptif
    Square,       // 80 × 80 mm  — autocollant / réseau social
    Poster,       // 148 × 210 mm (A5) — affiche promotionnelle
    Ticket,       // 90 × 50 mm  — bon / coupon / billet
}

impl CardLayout {
    pub const ALL: &'static [CardLayout] = &[
        Self::BusinessCard,
        Self::Label,
        Self::Badge,
        Self::Flyer,
        Self::Square,
        Self::Poster,
        Self::Ticket,
    ];

    /// Translation key of the layout name (section `card_layout`).
    pub fn label_key(&self) -> &'static str {
        match self {
            Self::BusinessCard => "card_layout.business_card",
            Self::Label        => "card_layout.label",
            Self::Badge        => "card_layout.badge",
            Self::Flyer        => "card_layout.flyer",
            Self::Square       => "card_layout.square",
            Self::Poster       => "card_layout.poster",
            Self::Ticket       => "card_layout.ticket",
        }
    }

    /// Canvas dimensions in pixels at 96 dpi (for preview / PNG export).
    pub fn canvas_px(&self) -> (u32, u32) {
        // 1 mm ≈ 3.78 px @ 96 dpi
        const K: f32 = 3.78;
        match self {
            Self::BusinessCard => (mm(85.0,  K), mm(54.0,  K)),
            Self::Label        => (mm(60.0,  K), mm(60.0,  K)),
            Self::Badge        => (mm(90.0,  K), mm(60.0,  K)),
            Self::Flyer        => (mm(148.0, K), mm(105.0, K)),
            Self::Square       => (mm(80.0,  K), mm(80.0,  K)),
            Self::Poster       => (mm(148.0, K), mm(210.0, K)),
            Self::Ticket       => (mm(90.0,  K), mm(50.0,  K)),
        }
    }

    /// Field labels for the text zones.
    /// Translation keys of the text fields of this layout (section `card_field`).
    pub fn field_label_keys(&self) -> &'static [&'static str] {
        match self {
            Self::BusinessCard => &["card_field.name", "card_field.job_title", "card_field.phone", "card_field.email", "card_field.website"],
            Self::Label        => &["card_field.title", "card_field.description"],
            Self::Badge        => &["card_field.name", "card_field.organization", "card_field.identifier"],
            Self::Flyer        => &["card_field.title", "card_field.subtitle", "card_field.description", "card_field.contact"],
            Self::Square       => &["card_field.title", "card_field.subtitle", "card_field.contact"],
            Self::Poster       => &["card_field.title", "card_field.tagline", "card_field.description", "card_field.contact", "card_field.website"],
            Self::Ticket       => &["card_field.event", "card_field.date_place", "card_field.reference"],
        }
    }
}

fn mm(v: f32, k: f32) -> u32 { (v * k) as u32 }

#[derive(Clone, Serialize, Deserialize)]
pub struct CardConfig {
    pub layout: CardLayout,
    pub bg_color: [u8; 3],
    pub text_color: [u8; 3],
    pub accent_color: [u8; 3],
    pub fields: Vec<String>,   // indexed by layout.field_label_keys()
}

impl CardConfig {
    pub fn new(layout: CardLayout) -> Self {
        Self {
            layout,
            bg_color: [255, 255, 255],
            text_color: [30, 30, 30],
            accent_color: [50, 100, 200],
            fields: layout.field_label_keys().iter().map(|_| String::new()).collect(),
        }
    }
}

impl Default for CardConfig {
    fn default() -> Self { Self::new(CardLayout::BusinessCard) }
}

// ─── SVG rendering ───────────────────────────────────────────────────────────

pub fn to_svg(config: &CardConfig, matrix: Option<&QrMatrix>, profile: &StyleProfile) -> String {
    let (w, h) = config.layout.canvas_px();
    let bg  = hex(config.bg_color);
    let fg  = hex(config.text_color);
    let acc = hex(config.accent_color);

    let mut svg = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"
     width="{w}" height="{h}" viewBox="0 0 {w} {h}">
  <rect width="{w}" height="{h}" fill="{bg}"/>
"#
    );

    // ── Layout-specific placement ─────────────────────────────────────────
    match config.layout {
        CardLayout::BusinessCard => {
            let qr_sz  = (h as f32 * 0.82) as u32;
            let qr_x   = (h as f32 * 0.09) as u32;
            let qr_y   = (h as f32 * 0.09) as u32;
            let text_x = (qr_x + qr_sz + (h as f32 * 0.07) as u32) as i32;
            let text_w = w as i32 - text_x - (h as f32 * 0.06) as i32;

            embed_qr_svg(&mut svg, matrix, profile, qr_x, qr_y, qr_sz);

            // Accent bar
            let bar_x = text_x - 6;
            svg.push_str(&format!(
                r#"  <rect x="{bar_x}" y="{qr_y}" width="3" height="{qr_sz}" fill="{acc}" rx="1"/>
"#
            ));

            let labels = config.layout.field_label_keys();
            let mut ty = qr_y as i32 + 8;
            for (i, field) in config.fields.iter().enumerate() {
                if field.is_empty() { continue; }
                let (fs, color, bold) = match i {
                    0 => (15, &fg, true),
                    1 => (10, &acc, false),
                    _ => (9, &fg, false),
                };
                let icon = match i {
                    2 => "☎ ", 3 => "✉ ", 4 => "🌐 ", _ => "",
                };
                let weight = if bold { "bold" } else { "normal" };
                let label_hint = egui_hint(labels.get(i).copied().unwrap_or(""));
                svg.push_str(&format!(
                    r#"  <text x="{text_x}" y="{ty}" font-family="Arial,sans-serif" font-size="{fs}"
       font-weight="{weight}" fill="{color}" textLength="{text_w}"
       lengthAdjust="spacingAndGlyphs"><!-- {label_hint} -->{icon}{}</text>
"#,
                    escape_xml(field)
                ));
                ty += fs + 5;
            }
        }

        CardLayout::Label => {
            let qr_sz = (w as f32 * 0.58) as u32;
            let qr_x  = (w - qr_sz) / 2;
            let qr_y  = (h as f32 * 0.06) as u32;
            embed_qr_svg(&mut svg, matrix, profile, qr_x, qr_y, qr_sz);

            let text_y_base = qr_y + qr_sz + 10;
            let labels = config.layout.field_label_keys();
            let mut ty = text_y_base as i32;
            for (i, field) in config.fields.iter().enumerate() {
                if field.is_empty() { continue; }
                let (fs, color) = if i == 0 { (13, &acc) } else { (9, &fg) };
                let label_hint = egui_hint(labels.get(i).copied().unwrap_or(""));
                svg.push_str(&format!(
                    r#"  <text x="{}" y="{ty}" text-anchor="middle"
       font-family="Arial,sans-serif" font-size="{fs}" fill="{color}">
    <!-- {label_hint} -->{}</text>
"#,
                    w / 2,
                    escape_xml(field)
                ));
                ty += fs + 4;
            }
        }

        CardLayout::Badge => {
            let qr_sz = (h as f32 * 0.75) as u32;
            let qr_x  = (h as f32 * 0.12) as u32;
            let qr_y  = (h - qr_sz) / 2;
            embed_qr_svg(&mut svg, matrix, profile, qr_x, qr_y, qr_sz);

            // Accent header strip
            svg.push_str(&format!(
                r#"  <rect x="0" y="0" width="{w}" height="12" fill="{acc}"/>
"#
            ));

            let text_x = (qr_x + qr_sz + 16) as i32;
            let text_w = w as i32 - text_x - 10;
            let labels = config.layout.field_label_keys();
            let mut ty = (h as f32 * 0.32) as i32;
            for (i, field) in config.fields.iter().enumerate() {
                if field.is_empty() { continue; }
                let (fs, color, bold) = match i {
                    0 => (16, &fg, true),
                    1 => (10, &acc, false),
                    _ => (8, &fg, false),
                };
                let weight = if bold { "bold" } else { "normal" };
                let label_hint = egui_hint(labels.get(i).copied().unwrap_or(""));
                svg.push_str(&format!(
                    r#"  <text x="{text_x}" y="{ty}" font-family="Arial,sans-serif" font-size="{fs}"
       font-weight="{weight}" fill="{color}" textLength="{text_w}"
       lengthAdjust="spacingAndGlyphs"><!-- {label_hint} -->{}</text>
"#,
                    escape_xml(field)
                ));
                ty += fs + 6;
            }
        }

        CardLayout::Flyer => {
            let qr_sz = (w as f32 * 0.40) as u32;
            let qr_x  = (w - qr_sz) / 2;
            let qr_y  = (h as f32 * 0.10) as u32;
            embed_qr_svg(&mut svg, matrix, profile, qr_x, qr_y, qr_sz);

            let ty_base = (qr_y + qr_sz + 14) as i32;
            let labels = config.layout.field_label_keys();
            let mut ty = ty_base;
            let cx = (w / 2) as i32;
            for (i, field) in config.fields.iter().enumerate() {
                if field.is_empty() { continue; }
                let (fs, color, bold) = match i {
                    0 => (18, &acc, true),
                    1 => (12, &fg, true),
                    _ => (10, &fg, false),
                };
                let weight = if bold { "bold" } else { "normal" };
                let label_hint = egui_hint(labels.get(i).copied().unwrap_or(""));
                svg.push_str(&format!(
                    r#"  <text x="{cx}" y="{ty}" text-anchor="middle"
       font-family="Arial,sans-serif" font-size="{fs}" font-weight="{weight}"
       fill="{color}"><!-- {label_hint} -->{}</text>
"#,
                    escape_xml(field)
                ));
                ty += fs + 5;
            }
        }

        // ── Carré ─────────────────────────────────────────────────────────────
        CardLayout::Square => {
            let qr_sz = (w as f32 * 0.70) as u32;
            let qr_x  = (w - qr_sz) / 2;
            let qr_y  = (h as f32 * 0.06) as u32;
            embed_qr_svg(&mut svg, matrix, profile, qr_x, qr_y, qr_sz);

            let labels = config.layout.field_label_keys();
            let mut ty = (qr_y + qr_sz + 12) as i32;
            let cx = (w / 2) as i32;
            for (i, field) in config.fields.iter().enumerate() {
                if field.is_empty() { continue; }
                let (fs, color, bold) = match i {
                    0 => (13, &acc, true),
                    1 => (10, &fg, false),
                    _ => (9,  &fg, false),
                };
                let weight = if bold { "bold" } else { "normal" };
                let label_hint = egui_hint(labels.get(i).copied().unwrap_or(""));
                svg.push_str(&format!(
                    r#"  <text x="{cx}" y="{ty}" text-anchor="middle"
       font-family="Arial,sans-serif" font-size="{fs}" font-weight="{weight}"
       fill="{color}"><!-- {label_hint} -->{}</text>
"#,
                    escape_xml(field)
                ));
                ty += fs + 4;
            }
        }

        // ── Affiche A5 ────────────────────────────────────────────────────────
        CardLayout::Poster => {
            // Accent header band
            svg.push_str(&format!(
                r#"  <rect x="0" y="0" width="{w}" height="14" fill="{acc}"/>
"#
            ));

            let qr_sz = (w as f32 * 0.45) as u32;
            let qr_x  = (w - qr_sz) / 2;
            let qr_y  = (h as f32 * 0.06) as u32;
            embed_qr_svg(&mut svg, matrix, profile, qr_x, qr_y, qr_sz);

            let labels = config.layout.field_label_keys();
            let mut ty = (qr_y + qr_sz + 20) as i32;
            let cx = (w / 2) as i32;
            for (i, field) in config.fields.iter().enumerate() {
                if field.is_empty() { continue; }
                let (fs, color, bold) = match i {
                    0 => (22, &acc, true),
                    1 => (14, &fg, true),
                    _ => (11, &fg, false),
                };
                let weight = if bold { "bold" } else { "normal" };
                let label_hint = egui_hint(labels.get(i).copied().unwrap_or(""));
                svg.push_str(&format!(
                    r#"  <text x="{cx}" y="{ty}" text-anchor="middle"
       font-family="Arial,sans-serif" font-size="{fs}" font-weight="{weight}"
       fill="{color}"><!-- {label_hint} -->{}</text>
"#,
                    escape_xml(field)
                ));
                ty += fs + 6;
            }
        }

        // ── Ticket ────────────────────────────────────────────────────────────
        CardLayout::Ticket => {
            // Thin accent band at top
            svg.push_str(&format!(
                r#"  <rect x="0" y="0" width="{w}" height="8" fill="{acc}"/>
"#
            ));

            let qr_sz  = (h as f32 * 0.78) as u32;
            let qr_x   = (h as f32 * 0.08) as u32;
            let qr_y   = (h - qr_sz) / 2;
            let text_x = (qr_x + qr_sz + 14) as i32;
            let text_w = w as i32 - text_x - 8;
            embed_qr_svg(&mut svg, matrix, profile, qr_x, qr_y, qr_sz);

            let labels = config.layout.field_label_keys();
            let mut ty = (h as f32 * 0.32) as i32;
            for (i, field) in config.fields.iter().enumerate() {
                if field.is_empty() { continue; }
                let (fs, color, bold) = match i {
                    0 => (13, &fg, true),
                    1 => (9,  &acc, false),
                    _ => (8,  &fg, false),
                };
                let weight = if bold { "bold" } else { "normal" };
                let label_hint = egui_hint(labels.get(i).copied().unwrap_or(""));
                svg.push_str(&format!(
                    r#"  <text x="{text_x}" y="{ty}" font-family="Arial,sans-serif" font-size="{fs}"
       font-weight="{weight}" fill="{color}" textLength="{text_w}"
       lengthAdjust="spacingAndGlyphs"><!-- {label_hint} -->{}</text>
"#,
                    escape_xml(field)
                ));
                ty += fs + 5;
            }
        }
    }

    svg.push_str("</svg>\n");
    svg
}

/// Embed QR as PNG data-URI inside SVG (no external files).
fn embed_qr_svg(svg: &mut String, matrix: Option<&QrMatrix>, profile: &StyleProfile, x: u32, y: u32, size: u32) {
    let Some(matrix) = matrix else {
        // Placeholder grey square
        svg.push_str(&format!(
            r##"  <rect x="{x}" y="{y}" width="{size}" height="{size}" fill="#CCCCCC" rx="4"/>
  <text x="{}" y="{}" text-anchor="middle" fill="#888" font-size="12">QR code</text>
"##,
            x + size / 2, y + size / 2
        ));
        return;
    };

    // Render QR to a small profile-styled image, then base64-encode
    let mut tmp_profile = profile.clone();
    tmp_profile.module_px = (size as usize / (matrix.len() + tmp_profile.quiet_zone as usize * 2 + 1)).max(1) as u32;
    tmp_profile.quiet_zone = 2;
    let img = renderer::render(matrix, &tmp_profile);

    let mut png_bytes: Vec<u8> = Vec::new();
    {
        use image::ImageEncoder;
        let enc = image::codecs::png::PngEncoder::new(&mut png_bytes);
        let _ = enc.write_image(img.as_raw(), img.width(), img.height(), image::ColorType::Rgba8);
    }
    let b64 = base64_encode(&png_bytes);
    svg.push_str(&format!(
        r#"  <image x="{x}" y="{y}" width="{size}" height="{size}"
       xlink:href="data:image/png;base64,{b64}"/>
"#
    ));
}

fn hex(c: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", c[0], c[1], c[2])
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn egui_hint(s: &str) -> String { s.to_string() }

fn base64_encode(bytes: &[u8]) -> String {
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes)
}
