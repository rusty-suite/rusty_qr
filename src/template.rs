//! SVG template engine — variable substitution, field detection, preview rendering.
//!
//! ## Template variable reference
//!
//! | Variable                           | Content                                           |
//! |------------------------------------|---------------------------------------------------|
//! | `{{W}}` `{{H}}`                   | Canvas width / height in pixels                   |
//! | `{{BG}}` `{{FG}}` `{{AC}}`        | Colors (#RRGGBB) — driven by app color pickers    |
//! | `{{QR_X}}` `{{QR_Y}}` `{{QR_SZ}}` | QR position + size                               |
//! | `{{TX}}` `{{TA}}`                 | Text X / text-anchor ("start" or "middle")        |
//! | `{{TY0}}`..`{{TY4}}`              | Pre-computed Y for each text line                 |
//! | `{{AX}}` `{{AY}}` `{{AW}}` `{{AH}}` | Accent rect geometry                          |
//! | `{{QR_IMAGE}}`                    | Complete `<image>` element (or placeholder rect)  |
//! | `{{ACCENT_BLOCK}}`                | Layout-specific accent decoration                 |
//! | `{{TEXT_BLOCK}}`                  | All `<text>` elements (standard layout)           |
//! | `{{F0:default}}`..`{{F4:default}}` | Text field with fallback text                   |
//! | `{{C0:#RRGGBB\|Label}}`..`{{C4:...}}` | Extra color slot — shows a color picker in UI |
//!
//! ## Template metadata (in SVG comment)
//! Add `<!-- @palette BG=#rrggbb FG=#rrggbb AC=#rrggbb -->` to suggest default
//! colors; the app applies them automatically when the template is selected.

use std::collections::HashMap;
use image::ImageEncoder;

use crate::card::{CardConfig, CardLayout};
use crate::qr::encoder::QrMatrix;
use crate::qr::types::EcLevel;
use crate::style::{profile::StyleProfile, renderer};

// ─── Built-in templates (compiled-in) ────────────────────────────────────────

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
        description: "Fond blanc avec bordure, texte organisé par le thème",
        svg: include_str!("../templates/classic.svg"),
    },
    BuiltinTemplate {
        id: "dark", name: "Sombre",
        description: "Fond sombre, halo d'accent, texte clair",
        svg: include_str!("../templates/dark.svg"),
    },
    BuiltinTemplate {
        id: "minimal", name: "Minimal",
        description: "Fond uni, aucune décoration, typographie épurée",
        svg: include_str!("../templates/minimal.svg"),
    },
    BuiltinTemplate {
        id: "badge", name: "Badge",
        description: "Liseré coloré, style accréditation / badge",
        svg: include_str!("../templates/badge.svg"),
    },
    BuiltinTemplate {
        id: "modern", name: "Épuré Pro",
        description: "Bandeau dégradé, coins arrondis, palette personnalisable",
        svg: include_str!("../templates/modern.svg"),
    },
    BuiltinTemplate {
        id: "neon", name: "Néon",
        description: "Fond sombre, bordure et texte lumineux style néon",
        svg: include_str!("../templates/neon.svg"),
    },
    BuiltinTemplate {
        id: "gradient", name: "Dégradé",
        description: "Panneau gauche en dégradé, zone texte claire à droite",
        svg: include_str!("../templates/gradient.svg"),
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
    pub svg:         Option<String>,
}

// ─── Template field ───────────────────────────────────────────────────────────

/// One `{{Fx}}` text zone detected in a template.
#[derive(Clone)]
pub struct TemplateField {
    pub var:     String,   // "F0", "F1", …
    pub label:   String,   // display label (from card layout)
    pub default: String,   // default text from template syntax `{{F0:default}}`
    pub value:   String,   // current user value
    pub visible: bool,
}

// ─── Template color ───────────────────────────────────────────────────────────

/// One `{{Cx:#RRGGBB|Label}}` color slot detected in a template.
#[derive(Clone)]
pub struct TemplateColor {
    pub var:     String,   // "C0", "C1", …
    pub label:   String,   // display label (from `|Label` part, or auto)
    #[allow(dead_code)]
    pub default: [u8; 3], // color from template syntax (for future "reset" feature)
    pub value:   [u8; 3], // current user-picked color
}

/// Parse `<!-- @palette BG=#rrggbb FG=#rrggbb AC=#rrggbb -->` from template.
/// Returns (bg, fg, ac) defaults, any of which may be `None` if absent.
pub fn detect_palette_defaults(template: &str)
    -> (Option<[u8;3]>, Option<[u8;3]>, Option<[u8;3]>)
{
    let Some(line) = template.lines().find(|l| l.contains("@palette")) else {
        return (None, None, None);
    };
    fn ph(line: &str, key: &str) -> Option<[u8;3]> {
        let kw = format!("{key}=#");
        let pos = line.find(&kw)?;
        let s = &line[pos + kw.len()..];
        if s.len() < 6 { return None; }
        Some([
            u8::from_str_radix(&s[0..2], 16).ok()?,
            u8::from_str_radix(&s[2..4], 16).ok()?,
            u8::from_str_radix(&s[4..6], 16).ok()?,
        ])
    }
    (ph(line, "BG"), ph(line, "FG"), ph(line, "AC"))
}

/// Scan template for `{{C0:#RRGGBB|Label}}`..`{{C4:...}}` color slots.
pub fn detect_colors(template: &str) -> Vec<TemplateColor> {
    (0..5usize).filter_map(|i| {
        let prefix = format!("{{{{C{i}:#");
        let pos = template.find(&prefix)?;
        let rest = &template[pos + prefix.len()..];
        let end  = rest.find("}}")?;
        let inner = &rest[..end]; // e.g. "6C63FF|Couleur principale"
        let (hex_str, label) = if let Some(pipe) = inner.find('|') {
            (&inner[..pipe], inner[pipe+1..].to_string())
        } else {
            (inner, format!("Couleur C{i}"))
        };
        if hex_str.len() < 6 { return None; }
        let r = u8::from_str_radix(&hex_str[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex_str[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex_str[4..6], 16).ok()?;
        Some(TemplateColor {
            var: format!("C{i}"), label,
            default: [r, g, b], value: [r, g, b],
        })
    }).collect()
}

/// Scan `template` for `{{F0}}`..`{{F4}}` (and `{{F0:default}}`) placeholders.
pub fn detect_fields(template: &str, layout_labels: &[&str]) -> Vec<TemplateField> {
    (0..5usize).filter_map(|i| {
        let simple  = format!("{{{{F{i}}}}}");
        let ext_pfx = format!("{{{{F{i}:");
        if !template.contains(&simple) && !template.contains(&ext_pfx) {
            return None;
        }
        let default = extract_default(template, i);
        let label   = layout_labels.get(i).copied().unwrap_or("Champ").to_string();
        Some(TemplateField { var: format!("F{i}"), label, default, value: String::new(), visible: true })
    }).collect()
}

fn extract_default(template: &str, i: usize) -> String {
    let prefix = format!("{{{{F{i}:");
    if let Some(start) = template.find(&prefix) {
        let rest = &template[start + prefix.len()..];
        if let Some(end) = rest.find("}}") {
            return rest[..end].to_string();
        }
    }
    String::new()
}

// ─── Rendering ────────────────────────────────────────────────────────────────

/// Render `template` with all variables substituted, including the real QR PNG.
/// Use for file export.
pub fn render(
    template: &str,
    config:   &CardConfig,
    matrix:   Option<&QrMatrix>,
    profile:  &StyleProfile,
    fields:   &[TemplateField],
    colors:   &[TemplateColor],
    ec:       EcLevel,
) -> String {
    let t1   = preprocess_colors(template, colors);
    let t2   = preprocess_defaults(&t1, fields);
    let vars = build_vars(config, matrix, profile, fields, false, ec, None);
    substitute(&t2, &vars)
}

/// Like `render()` but optimised for live preview.
/// When `qr_image` is supplied (the already-rendered sidebar QR), it is
/// embedded directly — no re-render, and the result is pixel-identical to
/// the right-panel preview. Falls back to matrix re-render, then placeholder.
pub fn render_preview(
    template:  &str,
    config:    &CardConfig,
    fields:    &[TemplateField],
    colors:    &[TemplateColor],
    matrix:    Option<&QrMatrix>,
    profile:   &StyleProfile,
    ec:        EcLevel,
    qr_image:  Option<&image::RgbaImage>,
) -> String {
    let t1      = preprocess_colors(template, colors);
    let t2      = preprocess_defaults(&t1, fields);
    let preview = matrix.is_none() && qr_image.is_none();
    let vars    = build_vars(config, matrix, profile, fields, preview, ec, qr_image);
    substitute(&t2, &vars)
}

// ─── Variable substitution ────────────────────────────────────────────────────

/// Replace `{{Cx:#hex|label}}` patterns with the current color value.
fn preprocess_colors(template: &str, colors: &[TemplateColor]) -> String {
    let mut out = template.to_string();
    for color in colors {
        let prefix = format!("{{{{{}:#", color.var);
        loop {
            let Some(start) = out.find(&prefix) else { break };
            let after = &out[start + prefix.len()..];
            let Some(end) = after.find("}}") else { break };
            let full = format!("{prefix}{}}}}}",  &after[..end]);
            out = out.replacen(&full, &hex(color.value), 1);
        }
    }
    out
}

/// Replace `{{F0:default text}}` patterns BEFORE regular substitution.
/// Rules:
/// - visible=false → empty string
/// - visible=true, value non-empty → user value
/// - visible=true, value empty → template default text
fn preprocess_defaults(template: &str, fields: &[TemplateField]) -> String {
    let mut out = template.to_string();
    for i in 0..5usize {
        let prefix = format!("{{{{F{i}:");
        // Replace every occurrence of {{Fi:anything}}
        loop {
            let Some(start) = out.find(&prefix) else { break };
            let after = &out[start + prefix.len()..];
            let Some(end) = after.find("}}") else { break };
            let def_text = after[..end].to_string();
            let full     = format!("{prefix}{def_text}}}}}");
            let field    = fields.iter().find(|f| f.var == format!("F{i}"));
            let replacement = match field {
                Some(f) if !f.visible       => String::new(),
                Some(f) if !f.value.is_empty() => xml_escape(&f.value),
                _                            => xml_escape(&def_text),
            };
            out = out.replacen(&full, &replacement, 1);
        }
    }
    out
}

fn substitute(template: &str, vars: &HashMap<String, String>) -> String {
    let mut out = template.to_string();
    for (k, v) in vars {
        out = out.replace(&format!("{{{{{k}}}}}"), v);
    }
    out
}

// ─── Build variable map ───────────────────────────────────────────────────────

fn build_vars(
    config:   &CardConfig,
    matrix:   Option<&QrMatrix>,
    profile:  &StyleProfile,
    fields:   &[TemplateField],
    preview:  bool,
    ec:       EcLevel,
    qr_image: Option<&image::RgbaImage>,
) -> HashMap<String, String> {
    let (w, h) = config.layout.canvas_px();
    let qr_sz  = calc_qr_sz(config, w, h);
    let qr_x   = calc_qr_x(config, w, h, qr_sz);
    let qr_y   = calc_qr_y(config, h, qr_sz);

    // Text positioning per layout
    let (tx, ta, tys) = calc_text_geometry(config, w, h, qr_x, qr_y, qr_sz);
    // Accent rect
    let (ax, ay, aw, ah) = calc_accent_rect(config, w, h, qr_x, qr_y, qr_sz, tx);

    let mut vars: HashMap<String, String> = HashMap::new();
    // Dimensions
    vars.insert("W".into(), w.to_string());
    vars.insert("H".into(), h.to_string());
    // Colors
    vars.insert("BG".into(), hex(config.bg_color));
    vars.insert("FG".into(), hex(config.text_color));
    vars.insert("AC".into(), hex(config.accent_color));
    // QR geometry
    vars.insert("QR_X".into(),  qr_x.to_string());
    vars.insert("QR_Y".into(),  qr_y.to_string());
    vars.insert("QR_SZ".into(), qr_sz.to_string());
    // Text geometry
    vars.insert("TX".into(), tx.to_string());
    vars.insert("TA".into(), ta.to_string());
    for (i, &ty) in tys.iter().enumerate() {
        vars.insert(format!("TY{i}"), ty.to_string());
    }
    // Accent rect
    vars.insert("AX".into(), ax.to_string());
    vars.insert("AY".into(), ay.to_string());
    vars.insert("AW".into(), aw.to_string());
    vars.insert("AH".into(), ah.to_string());
    // Pre-built blocks
    vars.insert("QR_IMAGE".into(),
        if preview {
            qr_placeholder(qr_x, qr_y, qr_sz)
        } else if let Some(img) = qr_image {
            embed_rgba_image(img, qr_x, qr_y, qr_sz)
        } else {
            build_qr_image(matrix, profile, qr_x, qr_y, qr_sz, ec)
        });
    vars.insert("ACCENT_BLOCK".into(), build_accent_block(config, w, h, qr_x, qr_y, qr_sz));
    vars.insert("TEXT_BLOCK".into(),   build_text_block(config, w, h, qr_x, qr_y, qr_sz));
    // Field vars for simple {{F0}} (after preprocess_defaults already handled {{F0:default}})
    if fields.is_empty() {
        for (i, f) in config.fields.iter().enumerate() {
            vars.insert(format!("F{i}"), xml_escape(f));
        }
        for i in config.fields.len()..5 {
            vars.insert(format!("F{i}"), String::new());
        }
    } else {
        for tf in fields {
            vars.insert(tf.var.clone(),
                if !tf.visible { String::new() } else { xml_escape(&tf.value) });
        }
        for i in 0..5usize { vars.entry(format!("F{i}")).or_default(); }
    }

    vars
}

// ─── Geometry helpers ─────────────────────────────────────────────────────────

pub fn calc_qr_sz(config: &CardConfig, w: u32, h: u32) -> u32 {
    match config.layout {
        CardLayout::BusinessCard => (h as f32 * 0.82) as u32,
        CardLayout::Label        => (w as f32 * 0.58) as u32,
        CardLayout::Badge        => (h as f32 * 0.75) as u32,
        CardLayout::Flyer        => (w as f32 * 0.40) as u32,
    }
}

pub fn calc_qr_x(config: &CardConfig, w: u32, h: u32, qr_sz: u32) -> u32 {
    match config.layout {
        CardLayout::BusinessCard | CardLayout::Badge => (h as f32 * 0.09) as u32,
        _ => (w - qr_sz) / 2,
    }
}

pub fn calc_qr_y(config: &CardConfig, h: u32, qr_sz: u32) -> u32 {
    match config.layout {
        CardLayout::BusinessCard              => (h as f32 * 0.09) as u32,
        CardLayout::Label | CardLayout::Flyer => (h as f32 * 0.06) as u32,
        CardLayout::Badge                     => (h - qr_sz) / 2,
    }
}

fn calc_text_geometry(
    config: &CardConfig,
    w: u32, h: u32,
    qr_x: u32, qr_y: u32, qr_sz: u32,
) -> (u32, &'static str, [u32; 5]) {
    match config.layout {
        CardLayout::BusinessCard => {
            let tx  = qr_x + qr_sz + (h as f32 * 0.07) as u32;
            let ty0 = qr_y + 8;
            (tx, "start", [ty0, ty0+20, ty0+35, ty0+48, ty0+61])
        }
        CardLayout::Label => {
            let tx  = w / 2;
            let ty0 = qr_y + qr_sz + 14;
            (tx, "middle", [ty0, ty0+17, ty0+30, ty0+43, ty0+56])
        }
        CardLayout::Badge => {
            let tx  = qr_x + qr_sz + 16;
            let ty0 = (h as f32 * 0.32) as u32;
            (tx, "start", [ty0, ty0+20, ty0+33, ty0+45, ty0+57])
        }
        CardLayout::Flyer => {
            let tx  = w / 2;
            let ty0 = qr_y + qr_sz + 18;
            (tx, "middle", [ty0, ty0+24, ty0+41, ty0+55, ty0+69])
        }
    }
}

fn calc_accent_rect(
    config: &CardConfig,
    w: u32, _h: u32,
    _qr_x: u32, qr_y: u32, qr_sz: u32,
    tx: u32,
) -> (i32, u32, u32, u32) {
    // (ax, ay, aw, ah)
    match config.layout {
        CardLayout::BusinessCard => (tx as i32 - 6, qr_y, 3, qr_sz),
        CardLayout::Badge        => (0, 0, w, 12),
        _                        => (0, 0, 0, 0),
    }
}

// ─── SVG block builders ───────────────────────────────────────────────────────

fn qr_placeholder(qr_x: u32, qr_y: u32, qr_sz: u32) -> String {
    let cx = qr_x + qr_sz / 2;
    let cy = qr_y + qr_sz / 2;
    format!(
        "  <rect x=\"{qr_x}\" y=\"{qr_y}\" width=\"{qr_sz}\" height=\"{qr_sz}\" fill=\"#D0D0D0\" rx=\"4\"/>\n  <text x=\"{cx}\" y=\"{cy}\" text-anchor=\"middle\" dominant-baseline=\"middle\" fill=\"#888\" font-family=\"Arial,sans-serif\" font-size=\"14\">QR</text>"
    )
}

/// Encode a pre-rendered RGBA image as a base64 PNG and return an SVG `<image>` element.
/// Used when `qr_rendered_image` is already available — avoids a redundant re-render.
fn embed_rgba_image(img: &image::RgbaImage, qr_x: u32, qr_y: u32, qr_sz: u32) -> String {
    let mut png: Vec<u8> = Vec::new();
    let enc = image::codecs::png::PngEncoder::new(&mut png);
    let _ = enc.write_image(img.as_raw(), img.width(), img.height(), image::ColorType::Rgba8);
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&png);
    format!("  <image x=\"{qr_x}\" y=\"{qr_y}\" width=\"{qr_sz}\" height=\"{qr_sz}\" href=\"data:image/png;base64,{b64}\"/>")
}

fn build_qr_image(
    matrix:  Option<&QrMatrix>,
    profile: &StyleProfile,
    qr_x: u32, qr_y: u32, qr_sz: u32,
    ec: EcLevel,
) -> String {
    let Some(matrix) = matrix else {
        return qr_placeholder(qr_x, qr_y, qr_sz);
    };
    // Render with the profile exactly as configured so the result is
    // pixel-identical to the sidebar preview. The SVG <image> element
    // scales it to qr_sz, preserving relative logo size and padding.
    let img = renderer::render_ec(matrix, profile, ec);
    let mut png: Vec<u8> = Vec::new();
    let enc = image::codecs::png::PngEncoder::new(&mut png);
    let _ = enc.write_image(img.as_raw(), img.width(), img.height(), image::ColorType::Rgba8);
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&png);
    format!(
        "  <image x=\"{qr_x}\" y=\"{qr_y}\" width=\"{qr_sz}\" height=\"{qr_sz}\" href=\"data:image/png;base64,{b64}\"/>"
    )
}

fn build_accent_block(config: &CardConfig, w: u32, h: u32, qr_x: u32, qr_y: u32, qr_sz: u32) -> String {
    let acc = hex(config.accent_color);
    match config.layout {
        CardLayout::BusinessCard => {
            let tx    = qr_x + qr_sz + (h as f32 * 0.07) as u32;
            let bar_x = tx as i32 - 6;
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
    let (tx, ta, tys) = calc_text_geometry(config, w, h, qr_x, qr_y, qr_sz);

    let font_sizes: &[(usize, bool)] = match config.layout {
        CardLayout::BusinessCard => &[(14,true),(10,false),(9,false),(9,false),(9,false)],
        CardLayout::Label        => &[(13,false),(9,false),(9,false),(9,false),(9,false)],
        CardLayout::Badge        => &[(16,true),(10,false),(8,false),(8,false),(8,false)],
        CardLayout::Flyer        => &[(18,true),(12,true),(10,false),(10,false),(10,false)],
    };

    for (i, field) in config.fields.iter().enumerate() {
        if field.is_empty() { continue; }
        let &(fs, bold) = font_sizes.get(i).unwrap_or(&(9, false));
        let color  = if i == 1 { acc.as_str() } else { fg.as_str() };
        let weight = if bold { "bold" } else { "normal" };
        let ty     = tys[i.min(4)];
        out.push_str(&format!(
            "  <text x=\"{tx}\" y=\"{ty}\" text-anchor=\"{ta}\" dominant-baseline=\"auto\" \
             font-family=\"Arial,sans-serif\" font-size=\"{fs}\" font-weight=\"{weight}\" \
             fill=\"{color}\">{}</text>\n",
            xml_escape(field),
        ));
    }
    out
}

// ─── SVG → RGBA via resvg (for egui preview texture) ─────────────────────────

/// System font database loaded once for the process lifetime.
fn system_font_db() -> resvg::usvg::fontdb::Database {
    use std::sync::OnceLock;
    static DB: OnceLock<resvg::usvg::fontdb::Database> = OnceLock::new();
    DB.get_or_init(|| {
        let mut db = resvg::usvg::fontdb::Database::new();
        db.load_system_fonts();
        db
    }).clone()
}

/// Rasterize an SVG string to straight-alpha RGBA bytes, scaled to fit within
/// `max_w × max_h`. Returns `(rgba, width, height)` or `None` on error.
pub fn svg_to_rgba(svg_str: &str, max_w: u32, max_h: u32) -> Option<(Vec<u8>, u32, u32)> {
    use resvg::usvg;
    let mut opts = usvg::Options::default();
    *opts.fontdb_mut() = system_font_db();

    let tree = usvg::Tree::from_str(svg_str, &opts).ok()?;
    let sz   = tree.size();
    let sw   = sz.width();
    let sh   = sz.height();
    if sw <= 0.0 || sh <= 0.0 { return None; }

    let scale = (max_w as f32 / sw).min(max_h as f32 / sh).min(1.0);
    let w = ((sw * scale).ceil() as u32).max(1);
    let h = ((sh * scale).ceil() as u32).max(1);

    let mut pixmap = resvg::tiny_skia::Pixmap::new(w, h)?;
    resvg::render(&tree,
        resvg::tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut());

    // tiny-skia returns premultiplied RGBA; egui expects straight/unmultiplied
    let straight = unpremultiply(pixmap.data());
    Some((straight, w, h))
}

/// Rasterize SVG at an explicit pixel-per-SVG-unit scale factor (e.g. 300/96 for PDF at 300 DPI).
pub fn svg_to_rgba_scaled(svg_str: &str, scale: f32) -> Option<(Vec<u8>, u32, u32)> {
    use resvg::usvg;
    let mut opts = usvg::Options::default();
    *opts.fontdb_mut() = system_font_db();
    let tree = usvg::Tree::from_str(svg_str, &opts).ok()?;
    let sz   = tree.size();
    let sw   = sz.width();
    let sh   = sz.height();
    if sw <= 0.0 || sh <= 0.0 { return None; }

    let w = ((sw * scale).ceil() as u32).max(1);
    let h = ((sh * scale).ceil() as u32).max(1);

    let mut pixmap = resvg::tiny_skia::Pixmap::new(w, h)?;
    resvg::render(&tree,
        resvg::tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut());

    let straight = unpremultiply(pixmap.data());
    Some((straight, w, h))
}

/// Convert premultiplied RGBA (tiny-skia output) to straight-alpha RGBA (egui input).
fn unpremultiply(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len());
    for px in data.chunks_exact(4) {
        let (r, g, b, a) = (px[0], px[1], px[2], px[3]);
        if a == 0 {
            out.extend_from_slice(&[0, 0, 0, 0]);
        } else {
            let af = a as f32 / 255.0;
            out.push((r as f32 / af).min(255.0) as u8);
            out.push((g as f32 / af).min(255.0) as u8);
            out.push((b as f32 / af).min(255.0) as u8);
            out.push(a);
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

/// Fetch the remote template index. Call from a background thread.
pub fn fetch_remote_index() -> Result<Vec<RemoteTemplate>, String> {
    let url  = format!("{GITHUB_BASE}index.json");
    let resp = ureq::get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .call()
        .map_err(|e| e.to_string())?;
    let json: serde_json::Value =
        resp.into_json::<serde_json::Value>().map_err(|e| e.to_string())?;
    let arr = json.as_array().ok_or_else(|| "index.json invalide".to_string())?;
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

/// Download a single remote template SVG. Call from a background thread.
pub fn fetch_remote_svg(file: &str) -> Result<String, String> {
    let url = format!("{GITHUB_BASE}{file}");
    ureq::get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .call()
        .map_err(|e| e.to_string())?
        .into_string()
        .map_err(|e| e.to_string())
}
