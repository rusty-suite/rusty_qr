//! SVG template engine for the card designer.
//!
//! Templates are SVG files containing `{{VARIABLE}}` placeholders.
//! Built-in templates are embedded at compile-time from the `templates/` folder.
//! Remote templates are fetched from the project's GitHub repository.

use std::collections::HashMap;
use image::ImageEncoder;

use crate::card::{CardConfig, CardLayout};
use crate::qr::encoder::QrMatrix;
use crate::style::{profile::StyleProfile, renderer};

// ─── Built-in templates ───────────────────────────────────────────────────────

pub struct BuiltinTemplate {
    #[allow(dead_code)]
    pub id:          &'static str,
    pub name:        &'static str,
    pub description: &'static str,
    pub svg:         &'static str,
}

pub const BUILTIN: &[BuiltinTemplate] = &[
    BuiltinTemplate {
        id: "classic", name: "Classique",
        description: "Fond blanc avec bordure, mise en page professionnelle",
        svg: include_str!("../templates/classic.svg"),
    },
    BuiltinTemplate {
        id: "dark", name: "Sombre",
        description: "Fond sombre avec halo d'accent et bordure colorée",
        svg: include_str!("../templates/dark.svg"),
    },
    BuiltinTemplate {
        id: "minimal", name: "Minimal",
        description: "Fond uni sans décoration, typographie épurée",
        svg: include_str!("../templates/minimal.svg"),
    },
    BuiltinTemplate {
        id: "badge", name: "Badge",
        description: "Liseré latéral coloré, style badge / accréditation",
        svg: include_str!("../templates/badge.svg"),
    },
];

// ─── Remote template descriptor ───────────────────────────────────────────────

#[derive(Clone)]
pub struct RemoteTemplate {
    #[allow(dead_code)]
    pub id:          String,
    pub name:        String,
    pub description: String,
    pub file:        String,
    pub svg:         Option<String>,   // None until downloaded
}

// ─── Rendering ────────────────────────────────────────────────────────────────

/// Substitute all `{{VAR}}` placeholders in `template` and return the final SVG.
pub fn render(
    template:  &str,
    config:    &CardConfig,
    matrix:    Option<&QrMatrix>,
    profile:   &StyleProfile,
) -> String {
    let (w, h) = config.layout.canvas_px();

    let qr_sz = calc_qr_sz(config, w, h);
    let qr_x  = calc_qr_x(config, w, h, qr_sz);
    let qr_y  = calc_qr_y(config, h, qr_sz);

    let mut vars: HashMap<String, String> = HashMap::new();
    vars.insert("W".into(),  w.to_string());
    vars.insert("H".into(),  h.to_string());
    vars.insert("BG".into(), hex(config.bg_color));
    vars.insert("FG".into(), hex(config.text_color));
    vars.insert("AC".into(), hex(config.accent_color));
    vars.insert("QR_X".into(),  qr_x.to_string());
    vars.insert("QR_Y".into(),  qr_y.to_string());
    vars.insert("QR_SZ".into(), qr_sz.to_string());
    vars.insert("QR_IMAGE".into(),     build_qr_image(matrix, profile, qr_x, qr_y, qr_sz));
    vars.insert("ACCENT_BLOCK".into(), build_accent_block(config, w, h, qr_x, qr_y, qr_sz));
    vars.insert("TEXT_BLOCK".into(),   build_text_block(config, w, h, qr_x, qr_y, qr_sz));

    for (i, f) in config.fields.iter().enumerate() {
        vars.insert(format!("F{i}"), xml_escape(f));
    }
    for i in config.fields.len()..5 {
        vars.insert(format!("F{i}"), String::new());
    }

    substitute(template, &vars)
}

fn substitute(template: &str, vars: &HashMap<String, String>) -> String {
    let mut out = template.to_string();
    for (k, v) in vars {
        out = out.replace(&format!("{{{{{k}}}}}"), v);
    }
    out
}

// ─── Geometry helpers ─────────────────────────────────────────────────────────

fn calc_qr_sz(config: &CardConfig, w: u32, h: u32) -> u32 {
    match config.layout {
        CardLayout::BusinessCard => (h as f32 * 0.82) as u32,
        CardLayout::Label        => (w as f32 * 0.58) as u32,
        CardLayout::Badge        => (h as f32 * 0.75) as u32,
        CardLayout::Flyer        => (w as f32 * 0.40) as u32,
    }
}

fn calc_qr_x(config: &CardConfig, w: u32, h: u32, qr_sz: u32) -> u32 {
    match config.layout {
        CardLayout::BusinessCard | CardLayout::Badge => (h as f32 * 0.09) as u32,
        _ => (w - qr_sz) / 2,
    }
}

fn calc_qr_y(config: &CardConfig, h: u32, qr_sz: u32) -> u32 {
    match config.layout {
        CardLayout::BusinessCard        => (h as f32 * 0.09) as u32,
        CardLayout::Label | CardLayout::Flyer => (h as f32 * 0.06) as u32,
        CardLayout::Badge               => (h - qr_sz) / 2,
    }
}

// ─── Block builders ───────────────────────────────────────────────────────────

fn build_qr_image(
    matrix:  Option<&QrMatrix>,
    profile: &StyleProfile,
    qr_x: u32, qr_y: u32, qr_sz: u32,
) -> String {
    let Some(matrix) = matrix else {
        return format!(
            "  <rect x=\"{qr_x}\" y=\"{qr_y}\" width=\"{qr_sz}\" height=\"{qr_sz}\" fill=\"#CCCCCC\" rx=\"4\"/>\n  <text x=\"{}\" y=\"{}\" text-anchor=\"middle\" fill=\"#888\" font-size=\"12\">QR code</text>",
            qr_x + qr_sz / 2,
            qr_y + qr_sz / 2,
        );
    };

    let mut tmp = profile.clone();
    tmp.module_px   = (qr_sz as usize / (matrix.len() + tmp.quiet_zone as usize * 2 + 1)).max(1) as u32;
    tmp.quiet_zone  = 2;
    let img = renderer::render(matrix, &tmp);

    let mut png_bytes: Vec<u8> = Vec::new();
    let enc = image::codecs::png::PngEncoder::new(&mut png_bytes);
    let _ = enc.write_image(img.as_raw(), img.width(), img.height(), image::ColorType::Rgba8);

    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);

    format!(
        "  <image x=\"{qr_x}\" y=\"{qr_y}\" width=\"{qr_sz}\" height=\"{qr_sz}\"\n         xlink:href=\"data:image/png;base64,{b64}\"/>",
    )
}

fn build_accent_block(config: &CardConfig, w: u32, h: u32, qr_x: u32, qr_y: u32, qr_sz: u32) -> String {
    let acc = hex(config.accent_color);
    match config.layout {
        CardLayout::BusinessCard => {
            let text_x = qr_x + qr_sz + (h as f32 * 0.07) as u32;
            let bar_x  = text_x as i32 - 6;
            format!("  <rect x=\"{bar_x}\" y=\"{qr_y}\" width=\"3\" height=\"{qr_sz}\" fill=\"{acc}\" rx=\"1\"/>")
        }
        CardLayout::Badge => {
            format!("  <rect x=\"0\" y=\"0\" width=\"{w}\" height=\"12\" fill=\"{acc}\"/>")
        }
        _ => String::new(),
    }
}

fn build_text_block(config: &CardConfig, w: u32, h: u32, qr_x: u32, qr_y: u32, qr_sz: u32) -> String {
    let mut out = String::new();
    let fg  = hex(config.text_color);
    let acc = hex(config.accent_color);
    let labels = config.layout.field_labels();

    match config.layout {
        CardLayout::BusinessCard => {
            let text_x = (qr_x + qr_sz + (h as f32 * 0.07) as u32) as i32;
            let text_w = w as i32 - text_x - (h as f32 * 0.06) as i32;
            let mut ty = qr_y as i32 + 8;
            for (i, field) in config.fields.iter().enumerate() {
                if field.is_empty() { continue; }
                let (fs, color, bold) = match i {
                    0 => (15, fg.as_str(), true),
                    1 => (10, acc.as_str(), false),
                    _ => (9,  fg.as_str(), false),
                };
                let icon   = match i { 2 => "☎ ", 3 => "✉ ", 4 => "🌐 ", _ => "" };
                let weight = if bold { "bold" } else { "normal" };
                let _ = labels.get(i); // suppress unused
                out.push_str(&format!(
                    "  <text x=\"{text_x}\" y=\"{ty}\" font-family=\"Arial,sans-serif\" \
                     font-size=\"{fs}\" font-weight=\"{weight}\" fill=\"{color}\" \
                     textLength=\"{text_w}\" lengthAdjust=\"spacingAndGlyphs\">{icon}{}</text>\n",
                    xml_escape(field),
                ));
                ty += fs + 5;
            }
        }

        CardLayout::Label => {
            let cx    = (w / 2) as i32;
            let mut ty = (qr_y + qr_sz + 10) as i32;
            for (i, field) in config.fields.iter().enumerate() {
                if field.is_empty() { continue; }
                let (fs, color) = if i == 0 { (13, acc.as_str()) } else { (9, fg.as_str()) };
                out.push_str(&format!(
                    "  <text x=\"{cx}\" y=\"{ty}\" text-anchor=\"middle\" \
                     font-family=\"Arial,sans-serif\" font-size=\"{fs}\" fill=\"{color}\">{}</text>\n",
                    xml_escape(field),
                ));
                ty += fs + 4;
            }
        }

        CardLayout::Badge => {
            let text_x = (qr_x + qr_sz + 16) as i32;
            let text_w = w as i32 - text_x - 10;
            let mut ty = (h as f32 * 0.32) as i32;
            for (i, field) in config.fields.iter().enumerate() {
                if field.is_empty() { continue; }
                let (fs, color, bold) = match i {
                    0 => (16, fg.as_str(), true),
                    1 => (10, acc.as_str(), false),
                    _ => (8,  fg.as_str(), false),
                };
                let weight = if bold { "bold" } else { "normal" };
                out.push_str(&format!(
                    "  <text x=\"{text_x}\" y=\"{ty}\" font-family=\"Arial,sans-serif\" \
                     font-size=\"{fs}\" font-weight=\"{weight}\" fill=\"{color}\" \
                     textLength=\"{text_w}\" lengthAdjust=\"spacingAndGlyphs\">{}</text>\n",
                    xml_escape(field),
                ));
                ty += fs + 6;
            }
        }

        CardLayout::Flyer => {
            let cx    = (w / 2) as i32;
            let mut ty = (qr_y + qr_sz + 14) as i32;
            for (i, field) in config.fields.iter().enumerate() {
                if field.is_empty() { continue; }
                let (fs, color, bold) = match i {
                    0 => (18, acc.as_str(), true),
                    1 => (12, fg.as_str(), true),
                    _ => (10, fg.as_str(), false),
                };
                let weight = if bold { "bold" } else { "normal" };
                out.push_str(&format!(
                    "  <text x=\"{cx}\" y=\"{ty}\" text-anchor=\"middle\" \
                     font-family=\"Arial,sans-serif\" font-size=\"{fs}\" font-weight=\"{weight}\" \
                     fill=\"{color}\">{}</text>\n",
                    xml_escape(field),
                ));
                ty += fs + 5;
            }
        }
    }

    out
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn hex(c: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", c[0], c[1], c[2])
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
}

// ─── GitHub remote fetching ───────────────────────────────────────────────────

const GITHUB_BASE: &str =
    "https://raw.githubusercontent.com/rusty-suite/rusty_qr/main/templates/";

/// Fetch the remote template index (non-blocking: call from a background thread).
pub fn fetch_remote_index() -> Result<Vec<RemoteTemplate>, String> {
    let url = format!("{GITHUB_BASE}index.json");
    let resp = ureq::get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .call()
        .map_err(|e| e.to_string())?;

    let json: serde_json::Value = resp.into_json::<serde_json::Value>().map_err(|e| e.to_string())?;
    let arr = json.as_array()
        .ok_or_else(|| "index.json invalide".to_string())?;

    Ok(arr.iter().filter_map(|v| {
        Some(RemoteTemplate {
            id:          v["id"].as_str()?.to_string(),
            name:        v["name"].as_str()?.to_string(),
            description: v["description"].as_str()?.to_string(),
            file:        v["file"].as_str()?.to_string(),
            svg:         None,
        })
    }).collect())
}

/// Download a single remote template SVG file (non-blocking: call from a background thread).
pub fn fetch_remote_svg(file: &str) -> Result<String, String> {
    let url = format!("{GITHUB_BASE}{file}");
    ureq::get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .call()
        .map_err(|e| e.to_string())?
        .into_string()
        .map_err(|e| e.to_string())
}
