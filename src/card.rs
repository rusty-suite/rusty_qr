//! Concepteur de cartes — mise en page QR (carte de visite, étiquette, badge, flyer).

use serde::{Deserialize, Serialize};
use crate::qr::encoder::QrMatrix;
use crate::style::{profile::StyleProfile, renderer};

// ─── Types ───────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub enum CardLayout {
    BusinessCard, // 85 × 54 mm — QR gauche, texte droite
    Label,        // 60 × 60 mm — QR centré, texte dessous
    Badge,        // 90 × 60 mm — QR + Nom + Titre
    Flyer,        // 148 × 105 mm (A6) — QR grand + descriptif
}

impl CardLayout {
    pub const ALL: &'static [CardLayout] = &[
        Self::BusinessCard,
        Self::Label,
        Self::Badge,
        Self::Flyer,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::BusinessCard => "Carte de visite (85×54 mm)",
            Self::Label        => "Étiquette QR (60×60 mm)",
            Self::Badge        => "Badge (90×60 mm)",
            Self::Flyer        => "Flyer compact (A6)",
        }
    }

    /// Canvas dimensions in pixels at 96 dpi (for preview / PNG export).
    pub fn canvas_px(&self) -> (u32, u32) {
        // 1 mm = 3.78 px @ 96 dpi
        const K: f32 = 3.78;
        match self {
            Self::BusinessCard => (mm(85.0, K), mm(54.0,  K)),
            Self::Label        => (mm(60.0, K), mm(60.0,  K)),
            Self::Badge        => (mm(90.0, K), mm(60.0,  K)),
            Self::Flyer        => (mm(148.0,K), mm(105.0, K)),
        }
    }

    /// Field labels for the text zones.
    pub fn field_labels(&self) -> &'static [&'static str] {
        match self {
            Self::BusinessCard => &["Nom", "Titre / Poste", "Téléphone", "Email", "Site web"],
            Self::Label        => &["Titre", "Description"],
            Self::Badge        => &["Nom", "Organisation", "Identifiant"],
            Self::Flyer        => &["Titre", "Sous-titre", "Description", "Contact"],
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
    pub fields: Vec<String>,   // indexed by layout.field_labels()
}

impl CardConfig {
    pub fn new(layout: CardLayout) -> Self {
        Self {
            layout,
            bg_color: [255, 255, 255],
            text_color: [30, 30, 30],
            accent_color: [50, 100, 200],
            fields: layout.field_labels().iter().map(|_| String::new()).collect(),
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

            let labels = config.layout.field_labels();
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
            let labels = config.layout.field_labels();
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
            let labels = config.layout.field_labels();
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

            // Bottom text area
            let ty_base = (qr_y + qr_sz + 14) as i32;
            let labels = config.layout.field_labels();
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

// ─── PDF rendering (via printpdf) ────────────────────────────────────────────

pub fn to_pdf(config: &CardConfig, matrix: Option<&QrMatrix>, profile: &StyleProfile) -> Result<Vec<u8>, String> {
    use printpdf::{PdfDocument, Mm, Image, ImageXObject, Px, ColorSpace, ColorBits, ImageTransform};

    let (w_px, h_px) = config.layout.canvas_px();
    const DPI: f32 = 96.0;
    let w_mm = w_px as f32 / DPI * 25.4;
    let h_mm = h_px as f32 / DPI * 25.4;

    let (doc, page, layer) = PdfDocument::new("Carte QR", Mm(w_mm), Mm(h_mm), "Layer 1");
    let lay = doc.get_page(page).get_layer(layer);

    // Background
    lay.set_fill_color(printpdf::Color::Rgb(printpdf::Rgb::new(
        config.bg_color[0] as f32 / 255.0,
        config.bg_color[1] as f32 / 255.0,
        config.bg_color[2] as f32 / 255.0,
        None,
    )));
    lay.add_rect(printpdf::Rect::new(Mm(0.0), Mm(0.0), Mm(w_mm), Mm(h_mm)));

    // QR image
    if let Some(mat) = matrix {
        let qr_sz_px = match config.layout {
            CardLayout::BusinessCard => (h_px as f32 * 0.82) as u32,
            CardLayout::Label        => (w_px as f32 * 0.58) as u32,
            CardLayout::Badge        => (h_px as f32 * 0.75) as u32,
            CardLayout::Flyer        => (w_px as f32 * 0.40) as u32,
        };
        let mut tmp = profile.clone();
        tmp.module_px = (qr_sz_px as usize / (mat.len() + tmp.quiet_zone as usize * 2 + 1)).max(1) as u32;
        tmp.quiet_zone = 2;
        let img = renderer::render(mat, &tmp);
        let rgb: Vec<u8> = img.pixels().flat_map(|p| [p.0[0], p.0[1], p.0[2]]).collect();
        let iw = img.width();
        let ih = img.height();

        let xobj = ImageXObject {
            width: Px(iw as usize),
            height: Px(ih as usize),
            color_space: ColorSpace::Rgb,
            bits_per_component: ColorBits::Bit8,
            interpolate: true,
            image_data: rgb,
            image_filter: None,
            clipping_bbox: None,
            smask: None,
        };

        let qr_x_px = match config.layout {
            CardLayout::BusinessCard | CardLayout::Badge => (h_px as f32 * 0.09) as i32,
            _ => ((w_px - qr_sz_px) / 2) as i32,
        };
        let qr_y_px = match config.layout {
            CardLayout::BusinessCard => (h_px as f32 * 0.09) as i32,
            CardLayout::Label | CardLayout::Flyer => (h_px as f32 * 0.06) as i32,
            CardLayout::Badge => ((h_px - qr_sz_px) / 2) as i32,
        };

        let qr_x_mm = qr_x_px as f32 / DPI * 25.4;
        // PDF y-axis is from bottom
        let qr_y_mm = h_mm - (qr_y_px as f32 + qr_sz_px as f32) / DPI * 25.4;
        let sz_mm_w = iw as f32 / DPI * 25.4;
        let sz_mm_h = ih as f32 / DPI * 25.4;

        Image::from(xobj).add_to_layer(lay.clone(), ImageTransform {
            translate_x: Some(Mm(qr_x_mm)),
            translate_y: Some(Mm(qr_y_mm)),
            scale_x: Some(sz_mm_w / iw as f32),
            scale_y: Some(sz_mm_h / ih as f32),
            ..Default::default()
        });
    }

    // Text via built-in font
    let font = doc.add_builtin_font(printpdf::BuiltinFont::HelveticaBold)
        .map_err(|e| format!("font: {e}"))?;
    let font_reg = doc.add_builtin_font(printpdf::BuiltinFont::Helvetica)
        .map_err(|e| format!("font: {e}"))?;

    let text_color = printpdf::Color::Rgb(printpdf::Rgb::new(
        config.text_color[0] as f32 / 255.0,
        config.text_color[1] as f32 / 255.0,
        config.text_color[2] as f32 / 255.0,
        None,
    ));

    lay.set_fill_color(text_color);

    let lay = doc.get_page(page).get_layer(layer);
    let text_x_mm = match config.layout {
        CardLayout::BusinessCard => {
            let qr_sz  = (h_px as f32 * 0.82) as f32;
            let qr_x   = h_px as f32 * 0.09;
            (qr_x + qr_sz + h_px as f32 * 0.07) / DPI * 25.4
        }
        _ => 5.0,
    };

    let mut ty_mm = h_mm - 8.0;
    for (i, field) in config.fields.iter().enumerate() {
        if field.is_empty() { continue; }
        let (fs, use_bold) = match (config.layout, i) {
            (CardLayout::BusinessCard, 0) => (5.5, true),
            (CardLayout::BusinessCard, 1) => (4.0, false),
            (CardLayout::Badge, 0)        => (6.0, true),
            _                             => (3.5, false),
        };
        let f = if use_bold { &font } else { &font_reg };
        lay.use_text(field.as_str(), fs, Mm(text_x_mm), Mm(ty_mm), f);
        ty_mm -= fs + 1.5;
    }

    doc.save_to_bytes().map_err(|e| format!("PDF: {e}"))
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

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
