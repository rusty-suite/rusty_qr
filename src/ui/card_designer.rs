//! Concepteur de cartes — mise en page QR (carte de visite, étiquette, badge, flyer).

use egui::Ui;
use crate::app::RustyQrApp;
use crate::card::{CardLayout, to_svg};
use crate::template;
use crate::theme;

const fn builtin_count() -> usize { template::BUILTIN.len() }
const fn custom_idx()    -> usize { template::BUILTIN.len() + 1 }
const fn remote_base()   -> usize { template::BUILTIN.len() + 2 }

pub fn show(app: &mut RustyQrApp, ui: &mut Ui) {
    ui.add_space(8.0);
    theme::title(ui, &format!("\u{1F5FA} {}", app.lang.t("card.title")));
    theme::hint(ui, &app.lang.t("card.hint"));
    ui.separator();
    ui.add_space(8.0);

    // Track if anything changed (to mark template preview dirty)
    let mut changed = false;

    ui.columns(2, |cols| {
        // ── Left: settings ───────────────────────────────────────────────────
        let ui = &mut cols[0];

        // ── Thème SVG ────────────────────────────────────────────────────────
        ui.label(egui::RichText::new(app.lang.t("card.svg_theme")).strong());
        ui.add_space(4.0);
        if show_template_selector(app, ui) { changed = true; }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(8.0);

        let card = &mut app.card;

        // ── Gabarit ──────────────────────────────────────────────────────────
        ui.label(egui::RichText::new(app.lang.t("card.layout")).strong());
        ui.add_space(4.0);

        egui::ComboBox::from_id_source("card_layout")
            .selected_text(app.lang.t(card.layout.label_key()))
            .show_ui(ui, |ui| {
                for &lay in CardLayout::ALL {
                    if ui.selectable_label(card.layout == lay, app.lang.t(lay.label_key())).clicked() {
                        // Preserve user-set colors and carry over field values
                        let old_fields  = card.fields.clone();
                        let old_bg      = card.bg_color;
                        let old_text    = card.text_color;
                        let old_accent  = card.accent_color;
                        *card = crate::card::CardConfig::new(lay);
                        card.bg_color     = old_bg;
                        card.text_color   = old_text;
                        card.accent_color = old_accent;
                        for (i, f) in old_fields.iter().enumerate() {
                            if let Some(dst) = card.fields.get_mut(i) {
                                if dst.is_empty() { *dst = f.clone(); }
                            }
                        }
                        changed = true;
                    }
                }
            });

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(8.0);

        // ── Couleurs ─────────────────────────────────────────────────────────
        ui.label(egui::RichText::new(app.lang.t("card.colors")).strong());
        ui.add_space(4.0);

        egui::Grid::new("card_colors")
            .num_columns(2)
            .spacing([8.0, 6.0])
            .show(ui, |ui| {
                ui.label(app.lang.t("card.bg"));
                if color_edit(ui, &mut card.bg_color) { changed = true; }
                ui.end_row();

                ui.label(app.lang.t("card.text"));
                if color_edit(ui, &mut card.text_color) { changed = true; }
                ui.end_row();

                ui.label(app.lang.t("card.accent"));
                if color_edit(ui, &mut card.accent_color) { changed = true; }
                ui.end_row();
            });

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(8.0);

        // ── Contenu ──────────────────────────────────────────────────────────
        // If a template with detected F-variables is active, show template fields.
        // Otherwise fall back to the card layout fields.
        if app.selected_template_idx > 0 && !app.template_field_data.is_empty() {
            ui.label(egui::RichText::new(app.lang.t("card.theme_fields")).strong());
            ui.add_space(4.0);
            theme::hint(ui, &app.lang.t("card.theme_fields_hint"));
            ui.add_space(4.0);

            let field_count = app.template_field_data.len();
            for i in 0..field_count {
                ui.horizontal(|ui| {
                    let vis  = &mut app.template_field_data[i].visible;
                    let icon = if *vis { "\u{1F441}" } else { "\u{1F648}" };
                    if ui.small_button(icon)
                        .on_hover_text(if *vis { app.lang.t("card.hide_field") } else { app.lang.t("card.show_field") })
                        .clicked()
                    {
                        *vis = !*vis;
                        changed = true;
                    }
                    // Labels are translation keys for layout fields, free text for template-defined ones.
                    let label = app.lang.t(&app.template_field_data[i].label);
                    ui.label(egui::RichText::new(&label).small().weak());
                });

                // Dim the input when the field is hidden; show template default as hint
                let visible  = app.template_field_data[i].visible;
                let hint_txt = app.template_field_data[i].default.clone();
                let dim      = egui::Color32::from_rgba_unmultiplied(180, 180, 180,
                    if visible { 220 } else { 80 });
                let r = ui.add(
                    egui::TextEdit::singleline(&mut app.template_field_data[i].value)
                        .hint_text(if hint_txt.is_empty() { app.lang.t("card.value_hint") } else { hint_txt.clone() })
                        .text_color(dim)
                        .desired_width(f32::INFINITY),
                );
                if r.changed() { changed = true; }
                ui.add_space(2.0);
            }
        } else {
            ui.label(egui::RichText::new(app.lang.t("card.content")).strong());
            ui.add_space(4.0);

            let labels = card.layout.field_label_keys();
            let field_count = card.fields.len();
            for i in 0..field_count {
                let label = app.lang.t(labels.get(i).copied().unwrap_or("card.field"));
                ui.label(egui::RichText::new(label).small().weak());
                if ui.text_edit_singleline(&mut card.fields[i]).changed() { changed = true; }
                ui.add_space(2.0);
            }
        }

        // ── Couleurs du thème (slots Cx) ─────────────────────────────────────
        if !app.template_color_data.is_empty() {
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(8.0);
            ui.label(egui::RichText::new(app.lang.t("card.theme_colors")).strong());
            ui.add_space(4.0);
            theme::hint(ui, &app.lang.t("card.theme_colors_hint"));
            ui.add_space(4.0);

            let color_count = app.template_color_data.len();
            egui::Grid::new("tpl_colors")
                .num_columns(2)
                .spacing([8.0, 6.0])
                .show(ui, |ui| {
                    for i in 0..color_count {
                        let label = app.template_color_data[i].label.clone();
                        ui.label(egui::RichText::new(&label).small());
                        if color_edit(ui, &mut app.template_color_data[i].value) {
                            changed = true;
                        }
                        ui.end_row();
                    }
                });
        }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(8.0);

        // ── Export ───────────────────────────────────────────────────────────
        ui.label(egui::RichText::new(app.lang.t("card.export")).strong());
        ui.add_space(4.0);

        if app.qr_matrix.is_none() {
            theme::status_warn(ui, &format!("\u{26A0} {}", app.lang.t("card.generate_first")));
        }

        let exporting = app.card_export_rx.is_some();
        if exporting {
            theme::hint(ui, &format!("\u{23F3} {}", app.lang.t("status.exporting")));
        } else {
            ui.horizontal(|ui| {
                if ui.button("SVG").on_hover_text(app.lang.t("card.svg_hint")).clicked() {
                    export_card(app, CardExportFmt::Svg);
                }
                if ui.button("PDF").on_hover_text(app.lang.t("card.pdf_hint")).clicked() {
                    export_card(app, CardExportFmt::Pdf);
                }
                if ui.button("PNG").on_hover_text(app.lang.t("card.png_hint")).clicked() {
                    export_card(app, CardExportFmt::Png);
                }
                if ui.button("JPEG").on_hover_text(app.lang.t("card.jpeg_hint")).clicked() {
                    export_card(app, CardExportFmt::Jpeg);
                }
            });
        }
        if let Some((ok, msg)) = &app.card_export_status {
            ui.add_space(4.0);
            if *ok { theme::status_ok(ui, msg); } else { theme::status_err(ui, msg); }
        }

        // ── Right: preview ────────────────────────────────────────────────
        let ui = &mut cols[1];

        if app.selected_template_idx > 0 {
            if let Some(tex) = &app.template_preview_texture {
                ui.label(egui::RichText::new(app.lang.t("card.theme_preview")).strong());
                ui.add_space(4.0);
                let avail = ui.available_width();
                let [tw, th] = [tex.size()[0] as f32, tex.size()[1] as f32];
                let scale   = (avail / tw).min(1.0);
                ui.add(egui::Image::new(tex).fit_to_exact_size(egui::vec2(tw * scale, th * scale)));
            } else {
                ui.label(egui::RichText::new(app.lang.t("card.preview")).strong());
                ui.add_space(4.0);
                theme::hint(ui, &app.lang.t("card.rendering"));
                show_egui_preview(app, ui);
            }
        } else {
            ui.label(egui::RichText::new(app.lang.t("card.preview")).strong());
            ui.add_space(4.0);
            show_egui_preview(app, ui);
        }
    });

    // Mark preview dirty if anything changed
    if changed {
        app.template_preview_dirty = true;
    }
}

// ─── Template selector ────────────────────────────────────────────────────────

/// Returns true if the template selection changed (triggers preview refresh).
fn show_template_selector(app: &mut RustyQrApp, ui: &mut Ui) -> bool {
    let n   = builtin_count();
    let ci  = custom_idx();
    let rb  = remote_base();
    let mut changed = false;

    // Label for current selection
    let cur_label: String = if app.selected_template_idx == 0 {
        app.lang.t("card.theme_none")
    } else if app.selected_template_idx <= n {
        let name = builtin_name(app, &template::BUILTIN[app.selected_template_idx - 1]);
        app.lang.tf("card.theme_builtin", &[("name", &name)])
    } else if app.selected_template_idx == ci {
        app.lang.t("card.theme_custom")
    } else {
        let ri = app.selected_template_idx - rb;
        app.remote_templates.get(ri)
            .map(|t| app.lang.tf("card.theme_github", &[("name", &t.name)]))
            .unwrap_or_else(|| "—".into())
    };

    egui::ComboBox::from_id_source("tmpl_select")
        .selected_text(&cur_label)
        .width(220.0)
        .show_ui(ui, |ui| {
            // None
            if ui.selectable_label(app.selected_template_idx == 0, app.lang.t("card.theme_none")).clicked() {
                apply_template(app, 0);
                changed = true;
            }
            // Built-ins
            ui.separator();
            ui.label(egui::RichText::new(app.lang.t("card.builtin_group")).weak().small());
            for i in 0..n {
                let t   = &template::BUILTIN[i];
                let idx = i + 1;
                let lbl = format!("{} — {}", builtin_name(app, t), app.lang.t(&format!("template.{}.desc", t.id)));
                if ui.selectable_label(app.selected_template_idx == idx, lbl).clicked() {
                    apply_template(app, idx);
                    changed = true;
                }
            }
            // Custom
            if app.custom_template_svg.is_some() {
                ui.separator();
                if ui.selectable_label(app.selected_template_idx == ci,
                    app.lang.t("card.theme_custom")).clicked()
                {
                    apply_template(app, ci);
                    changed = true;
                }
            }
            // Remote
            if !app.remote_templates.is_empty() {
                ui.separator();
                ui.label(egui::RichText::new("GitHub").weak().small());
                for i in 0..app.remote_templates.len() {
                    let idx  = rb + i;
                    let name = app.remote_templates[i].name.clone();
                    let desc = app.remote_templates[i].description.clone();
                    let ready = app.remote_templates[i].svg.is_some();
                    let lbl = if ready {
                        format!("{name} — {desc}")
                    } else {
                        format!("{name} — {desc} ⬇")
                    };
                    if ui.selectable_label(app.selected_template_idx == idx, lbl).clicked() {
                        apply_template(app, idx);
                        // Auto-download if not yet fetched
                        if !ready && app.remote_svg_dl.is_none() {
                            let file = app.remote_templates[i].file.clone();
                            let (tx, rx) = std::sync::mpsc::channel();
                            std::thread::spawn(move || {
                                let _ = tx.send(crate::template::fetch_remote_svg(&file));
                            });
                            app.remote_svg_dl = Some((i, rx));
                            app.remote_fetch_status = Some((true, app.lang.t("status.theme_downloading")));
                        }
                        changed = true;
                    }
                }
            }
        });

    // ── Auto-trigger remote index fetch on first visit ───────────────────
    if app.remote_fetch_rx.is_none() && app.remote_fetch_status.is_none() {
        trigger_remote_fetch(app);
    }

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        // Load custom SVG file
        if ui.button(app.lang.t("card.svg_file")).clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter(app.lang.t("export_format.svg_template"), &["svg"])
                .pick_file()
            {
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        app.custom_template_svg = Some(content);
                        apply_template(app, ci);
                        changed = true;
                    }
                    Err(e) => {
                        app.card_export_status = Some((false, format!("\u{2717} {}", app.lang.tf("status.read_error", &[("error", &e.to_string())]))));
                    }
                }
            }
        }

        // Manual refresh button
        let fetching = app.remote_fetch_rx.is_some();
        let btn_label = if fetching { "\u{23F3}" } else { "\u{1F504}" };
        if ui.add_enabled(!fetching, egui::Button::new(btn_label))
            .on_hover_text(app.lang.t("card.recheck_themes"))
            .clicked()
        {
            // Reset so we can fetch again even if already done
            app.remote_templates.clear();
            trigger_remote_fetch(app);
        }
    });

    // ── Connectivity / remote status strip ───────────────────────────────
    if let Some((ok, msg)) = &app.remote_fetch_status {
        ui.add_space(2.0);
        let fetching = app.remote_fetch_rx.is_some();
        if fetching {
            theme::hint(ui, &format!("\u{23F3} {msg}"));
        } else if *ok {
            ui.label(egui::RichText::new(msg).small()
                .color(egui::Color32::from_rgb(80, 200, 120)));
        } else {
            ui.label(egui::RichText::new(msg).small()
                .color(egui::Color32::from_rgb(220, 100, 60)));
        }
    }

    changed
}

/// Name of a built-in template in the user language.
fn builtin_name(app: &RustyQrApp, t: &template::BuiltinTemplate) -> String {
    app.lang.t(&format!("template.{}.name", t.id))
}

fn trigger_remote_fetch(app: &mut RustyQrApp) {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(crate::template::fetch_remote_index());
    });
    app.remote_fetch_rx = Some(rx);
    app.remote_fetch_status = Some((true, app.lang.t("status.checking_online")));
}

/// Apply a template selection: update index, detect fields + colors, mark preview dirty.
fn apply_template(app: &mut RustyQrApp, idx: usize) {
    app.selected_template_idx = idx;
    app.template_preview_dirty = true;

    let svg_opt: Option<String> = {
        let n  = builtin_count();
        let ci = custom_idx();
        let rb = remote_base();
        if idx == 0 {
            None
        } else if idx <= n {
            Some(template::BUILTIN[idx - 1].svg.to_string())
        } else if idx == ci {
            app.custom_template_svg.clone()
        } else {
            let ri = idx - rb;
            app.remote_templates.get(ri).and_then(|t| t.svg.clone())
        }
    };

    if let Some(svg) = &svg_opt {
        // Apply suggested palette (BG/FG/AC) if present in template
        let (bg, fg, ac) = template::detect_palette_defaults(svg);
        if let Some(c) = bg { app.card.bg_color     = c; }
        if let Some(c) = fg { app.card.text_color   = c; }
        if let Some(c) = ac { app.card.accent_color = c; }

        // Detect {{Fx}} text fields
        let labels   = app.card.layout.field_label_keys();
        let detected = template::detect_fields(svg, labels);
        let old_f    = std::mem::replace(&mut app.template_field_data, detected);
        for tf in &mut app.template_field_data {
            if let Some(prev) = old_f.iter().find(|o| o.var == tf.var) {
                tf.value   = prev.value.clone();
                tf.visible = prev.visible;
            }
        }

        // Detect {{Cx:#hex|label}} color slots
        let detected_c = template::detect_colors(svg);
        let old_c      = std::mem::replace(&mut app.template_color_data, detected_c);
        for tc in &mut app.template_color_data {
            if let Some(prev) = old_c.iter().find(|o| o.var == tc.var) {
                tc.value = prev.value;
            }
        }
    } else if idx == 0 {
        app.template_field_data.clear();
        app.template_color_data.clear();
        app.template_preview_texture = None;
    }
}

// ─── Color editor ─────────────────────────────────────────────────────────────

fn color_edit(ui: &mut Ui, c: &mut [u8; 3]) -> bool {
    let mut f = [c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0];
    let changed = ui.color_edit_button_rgb(&mut f).changed();
    if changed {
        *c = [(f[0] * 255.0) as u8, (f[1] * 255.0) as u8, (f[2] * 255.0) as u8];
    }
    changed
}

// ─── egui painter preview (used when no template or template not yet rendered) ─

fn show_egui_preview(app: &RustyQrApp, ui: &mut Ui) {
    let (w_px, h_px) = app.card.layout.canvas_px();
    let avail = ui.available_width() - 16.0;
    let scale = (avail / w_px as f32).min(1.0).min(600.0 / h_px as f32);
    let dw = w_px as f32 * scale;
    let dh = h_px as f32 * scale;

    let (rect, _) = ui.allocate_exact_size(egui::vec2(dw, dh), egui::Sense::hover());
    let painter = ui.painter();

    let bg     = egui::Color32::from_rgb(app.card.bg_color[0], app.card.bg_color[1], app.card.bg_color[2]);
    let acc    = egui::Color32::from_rgb(app.card.accent_color[0], app.card.accent_color[1], app.card.accent_color[2]);
    let fg_col = egui::Color32::from_rgb(app.card.text_color[0], app.card.text_color[1], app.card.text_color[2]);

    painter.rect_filled(rect, 4.0, bg);
    painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0, egui::Color32::from_gray(80)));

    let qr_sz = match app.card.layout {
        CardLayout::BusinessCard => dh * 0.82,
        CardLayout::Label        => dw * 0.58,
        CardLayout::Badge        => dh * 0.75,
        CardLayout::Flyer        => dw * 0.40,
        CardLayout::Square       => dw * 0.70,
        CardLayout::Poster       => dw * 0.45,
        CardLayout::Ticket       => dh * 0.78,
    };
    let qr_x = rect.left() + match app.card.layout {
        CardLayout::BusinessCard | CardLayout::Badge => dh * 0.09,
        CardLayout::Ticket                           => dh * 0.08,
        _                                            => (dw - qr_sz) / 2.0,
    };
    let qr_y = rect.top() + match app.card.layout {
        CardLayout::BusinessCard => dh * 0.09,
        CardLayout::Label | CardLayout::Flyer | CardLayout::Square | CardLayout::Poster
            => dh * 0.06,
        CardLayout::Badge | CardLayout::Ticket => (dh - qr_sz) / 2.0,
    };
    let qr_rect = egui::Rect::from_min_size(egui::pos2(qr_x, qr_y), egui::vec2(qr_sz, qr_sz));

    if let Some(tex) = &app.preview_texture {
        painter.image(tex.id(), qr_rect,
            egui::Rect::from_min_max(egui::pos2(0.0,0.0), egui::pos2(1.0,1.0)),
            egui::Color32::WHITE);
    } else {
        painter.rect_filled(qr_rect, 2.0, egui::Color32::from_gray(180));
        painter.text(qr_rect.center(), egui::Align2::CENTER_CENTER,
            "QR", egui::FontId::proportional(12.0), egui::Color32::from_gray(100));
    }

    match app.card.layout {
        CardLayout::BusinessCard => {
            let bar_x = qr_x + qr_sz + dh * 0.04;
            painter.rect_filled(
                egui::Rect::from_min_size(egui::pos2(bar_x, qr_y), egui::vec2(2.0, qr_sz)),
                1.0, acc);
        }
        CardLayout::Badge => {
            painter.rect_filled(
                egui::Rect::from_min_size(rect.left_top(), egui::vec2(dw, 8.0)),
                0.0, acc);
        }
        CardLayout::Poster => {
            painter.rect_filled(
                egui::Rect::from_min_size(rect.left_top(), egui::vec2(dw, 10.0)),
                0.0, acc);
        }
        CardLayout::Ticket => {
            painter.rect_filled(
                egui::Rect::from_min_size(rect.left_top(), egui::vec2(dw, 5.0)),
                0.0, acc);
        }
        _ => {}
    }

    // Determine which fields to show
    let preview_fields: Vec<(&str, bool)> = if app.selected_template_idx > 0 && !app.template_field_data.is_empty() {
        app.template_field_data.iter().map(|tf| (tf.value.as_str(), tf.visible)).collect()
    } else {
        app.card.fields.iter().map(|f| (f.as_str(), true)).collect()
    };
    let labels = app.card.layout.field_label_keys();

    let text_x = match app.card.layout {
        CardLayout::BusinessCard => qr_x + qr_sz + dh * 0.07,
        CardLayout::Badge        => qr_x + qr_sz + 10.0,
        CardLayout::Ticket       => qr_x + qr_sz + 12.0,
        _                        => rect.left() + dw / 2.0 - 60.0,
    };
    let mut ty = match app.card.layout {
        CardLayout::BusinessCard => qr_y + 6.0,
        CardLayout::Badge        => rect.top() + dh * 0.28,
        CardLayout::Ticket       => rect.top() + dh * 0.30,
        _                        => qr_y + qr_sz + 10.0, // Label, Flyer, Square, Poster
    };

    for (i, (field, visible)) in preview_fields.iter().enumerate() {
        if !visible { continue; }
        let display = if field.is_empty() {
            labels.get(i).copied().unwrap_or("").to_string()
        } else {
            field.to_string()
        };
        let (fs, color) = match (app.card.layout, i) {
            (CardLayout::BusinessCard, 0)
            | (CardLayout::Badge, 0)
            | (CardLayout::Flyer, 0)
            | (CardLayout::Square, 0)
            | (CardLayout::Ticket, 0) => (14.0, fg_col),
            (CardLayout::Poster, 0) => (18.0, fg_col),
            (CardLayout::Poster, 1) => (12.0, acc),
            (_, 1) => (10.0, acc),
            _ => (9.0, fg_col),
        };
        let alpha = if field.is_empty() { 80u8 } else { 220u8 };
        let col   = egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha);
        let anchor = match app.card.layout {
            CardLayout::Label | CardLayout::Flyer
            | CardLayout::Square | CardLayout::Poster => egui::Align2::CENTER_TOP,
            _ => egui::Align2::LEFT_TOP,
        };
        let tx = match app.card.layout {
            CardLayout::Label | CardLayout::Flyer
            | CardLayout::Square | CardLayout::Poster => rect.center().x,
            _ => text_x,
        };
        painter.text(egui::pos2(tx, ty), anchor, &display, egui::FontId::proportional(fs), col);
        ty += fs + 3.0;
    }
}

// ─── Export helpers ───────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
enum CardExportFmt { Svg, Pdf, Png, Jpeg }

fn get_active_template_svg(app: &RustyQrApp) -> Option<&str> {
    let n  = builtin_count();
    let ci = custom_idx();
    let rb = remote_base();
    if app.selected_template_idx == 0 {
        None
    } else if app.selected_template_idx <= n {
        Some(template::BUILTIN[app.selected_template_idx - 1].svg)
    } else if app.selected_template_idx == ci {
        app.custom_template_svg.as_deref()
    } else {
        let ri = app.selected_template_idx - rb;
        app.remote_templates.get(ri).and_then(|t| t.svg.as_deref())
    }
}

/// Single entry point for all card export formats.
/// SVG is written synchronously (fast string write).
/// PDF / PNG / JPEG are dispatched to a background thread so the UI stays responsive.
fn export_card(app: &mut RustyQrApp, fmt: CardExportFmt) {
    if app.card_export_rx.is_some() { return; } // already exporting

    let (ext, filter) = match fmt {
        CardExportFmt::Svg  => ("svg",  "export_format.svg"),
        CardExportFmt::Pdf  => ("pdf",  "export_format.pdf"),
        CardExportFmt::Png  => ("png",  "export_format.png"),
        CardExportFmt::Jpeg => ("jpg",  "export_format.jpg"),
    };

    let Some(path) = rfd::FileDialog::new()
        .add_filter(app.lang.t(filter), &[ext])
        .set_file_name(&format!("{}.{ext}", app.lang.t("card.export_file_name")))
        .save_file()
    else { return; };
    let path_str = path.to_string_lossy().into_owned();
    // Messages are resolved now: the export thread has no access to the language.
    let rasterize_err = app.lang.t("status.rasterize_error");
    let exported_msg  = app.lang.tf("status.exported", &[("path", &path_str)]);

    // Build the SVG string (always fast — pure string substitution + QR embed)
    let matrix_ref = app.qr_matrix.as_ref();
    let profile    = app.current_profile().clone();
    let svg = match get_active_template_svg(app) {
        Some(tpl) => template::render(
            tpl, &app.card, matrix_ref, &profile,
            &app.template_field_data, &app.template_color_data,
            app.form.ec_level,
        ),
        None => to_svg(&app.card, matrix_ref, &profile),
    };

    match fmt {
        // ── SVG: plain write, no rasterization needed ─────────────────────
        CardExportFmt::Svg => {
            app.card_export_status = Some(match std::fs::write(&path_str, &svg) {
                Ok(_)  => (true,  format!("\u{2713} {}", app.lang.tf("status.exported", &[("path", &path_str)]))),
                Err(e) => (false, format!("\u{2717} {e}")),
            });
        }

        // ── Raster / PDF: heavy work → background thread ──────────────────
        _ => {
            let (orig_w, orig_h) = app.card.layout.canvas_px();
            let (tx, rx) = std::sync::mpsc::channel();
            app.card_export_rx    = Some(rx);
            app.card_export_status = Some((true, format!("\u{23F3} {}", app.lang.t("status.exporting"))));

            std::thread::spawn(move || {
                const DPI_SCALE: f32 = 300.0 / 96.0; // 300 DPI output

                let result: Result<(), String> = match fmt {
                    CardExportFmt::Pdf => {
                        match crate::template::svg_to_rgba_scaled(&svg, DPI_SCALE) {
                            Some((rgba, w, h)) =>
                                crate::export::pdf::export_from_rgba(&rgba, w, h, orig_w, orig_h, &path_str),
                            None => Err(rasterize_err.clone()),
                        }
                    }
                    CardExportFmt::Png => {
                        match crate::template::svg_to_rgba_scaled(&svg, DPI_SCALE) {
                            Some((rgba, w, h)) =>
                                image::save_buffer(&path_str, &rgba, w, h, image::ColorType::Rgba8)
                                    .map_err(|e| e.to_string()),
                            None => Err(rasterize_err.clone()),
                        }
                    }
                    CardExportFmt::Jpeg => {
                        match crate::template::svg_to_rgba_scaled(&svg, DPI_SCALE) {
                            Some((rgba, w, h)) => {
                                // Alpha-composite onto white before JPEG encoding (no alpha in JPEG)
                                let rgb: Vec<u8> = rgba.chunks_exact(4).flat_map(|p| {
                                    let a = p[3] as f32 / 255.0;
                                    [
                                        (p[0] as f32 * a + 255.0 * (1.0 - a)) as u8,
                                        (p[1] as f32 * a + 255.0 * (1.0 - a)) as u8,
                                        (p[2] as f32 * a + 255.0 * (1.0 - a)) as u8,
                                    ]
                                }).collect();
                                image::save_buffer(&path_str, &rgb, w, h, image::ColorType::Rgb8)
                                    .map_err(|e| e.to_string())
                            }
                            None => Err(rasterize_err.clone()),
                        }
                    }
                    CardExportFmt::Svg => unreachable!(),
                };

                let msg = match result {
                    Ok(_)  => Ok(format!("\u{2713} {}", exported_msg)),
                    Err(e) => Err(format!("\u{2717} {e}")),
                };
                let _ = tx.send(msg);
            });
        }
    }
}
