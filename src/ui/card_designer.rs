//! Concepteur de cartes — mise en page QR (carte de visite, étiquette, badge, flyer).

use egui::Ui;
use crate::app::RustyQrApp;
use crate::card::{CardLayout, to_svg, to_pdf};
use crate::template;
use crate::theme;

const fn builtin_count() -> usize { template::BUILTIN.len() }
const fn custom_idx()    -> usize { template::BUILTIN.len() + 1 }
const fn remote_base()   -> usize { template::BUILTIN.len() + 2 }

pub fn show(app: &mut RustyQrApp, ui: &mut Ui) {
    ui.add_space(8.0);
    theme::title(ui, "\u{1F5FA} Concepteur de cartes");
    theme::hint(ui, "Mettez en page votre code QR sur une carte, étiquette ou flyer — puis exportez.");
    ui.separator();
    ui.add_space(8.0);

    // Track if anything changed (to mark template preview dirty)
    let mut changed = false;

    ui.columns(2, |cols| {
        // ── Left: settings ───────────────────────────────────────────────────
        let ui = &mut cols[0];

        // ── Thème SVG ────────────────────────────────────────────────────────
        ui.label(egui::RichText::new("Thème SVG").strong());
        ui.add_space(4.0);
        if show_template_selector(app, ui) { changed = true; }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(8.0);

        let card = &mut app.card;

        // ── Gabarit ──────────────────────────────────────────────────────────
        ui.label(egui::RichText::new("Gabarit").strong());
        ui.add_space(4.0);

        egui::ComboBox::from_id_source("card_layout")
            .selected_text(card.layout.label())
            .show_ui(ui, |ui| {
                for &lay in CardLayout::ALL {
                    if ui.selectable_label(card.layout == lay, lay.label()).clicked() {
                        let old_fields = card.fields.clone();
                        *card = crate::card::CardConfig::new(lay);
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
        ui.label(egui::RichText::new("Couleurs").strong());
        ui.add_space(4.0);

        egui::Grid::new("card_colors")
            .num_columns(2)
            .spacing([8.0, 6.0])
            .show(ui, |ui| {
                ui.label("Fond :");
                if color_edit(ui, &mut card.bg_color) { changed = true; }
                ui.end_row();

                ui.label("Texte :");
                if color_edit(ui, &mut card.text_color) { changed = true; }
                ui.end_row();

                ui.label("Accent :");
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
            ui.label(egui::RichText::new("Zones de texte du thème").strong());
            ui.add_space(4.0);
            theme::hint(ui, "Champs détectés dans le thème SVG sélectionné.");
            ui.add_space(4.0);

            let field_count = app.template_field_data.len();
            for i in 0..field_count {
                ui.horizontal(|ui| {
                    let vis  = &mut app.template_field_data[i].visible;
                    let icon = if *vis { "\u{1F441}" } else { "\u{1F648}" };
                    if ui.small_button(icon)
                        .on_hover_text(if *vis { "Masquer ce champ" } else { "Afficher ce champ" })
                        .clicked()
                    {
                        *vis = !*vis;
                        changed = true;
                    }
                    let label = app.template_field_data[i].label.clone();
                    ui.label(egui::RichText::new(&label).small().weak());
                });

                // Dim the input when the field is hidden; show template default as hint
                let visible  = app.template_field_data[i].visible;
                let hint_txt = app.template_field_data[i].default.clone();
                let dim      = egui::Color32::from_rgba_unmultiplied(180, 180, 180,
                    if visible { 220 } else { 80 });
                let r = ui.add(
                    egui::TextEdit::singleline(&mut app.template_field_data[i].value)
                        .hint_text(if hint_txt.is_empty() { "Valeur…" } else { &hint_txt })
                        .text_color(dim)
                        .desired_width(f32::INFINITY),
                );
                if r.changed() { changed = true; }
                ui.add_space(2.0);
            }
        } else {
            ui.label(egui::RichText::new("Contenu").strong());
            ui.add_space(4.0);

            let labels = card.layout.field_labels();
            let field_count = card.fields.len();
            for i in 0..field_count {
                let label = labels.get(i).copied().unwrap_or("Champ");
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
            ui.label(egui::RichText::new("Couleurs du thème").strong());
            ui.add_space(4.0);
            theme::hint(ui, "Couleurs supplémentaires définies par le thème SVG.");
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
        ui.label(egui::RichText::new("Exporter la carte").strong());
        ui.add_space(4.0);

        if app.qr_matrix.is_none() {
            theme::status_warn(ui, "⚠ Générez d'abord un QR dans l'onglet Créer.");
        }

        ui.horizontal(|ui| {
            if ui.button("SVG").clicked() { export_card_svg(app); }
            if ui.button("PDF").clicked() { export_card_pdf(app); }
        });
        if let Some((ok, msg)) = &app.card_export_status {
            ui.add_space(4.0);
            if *ok { theme::status_ok(ui, msg); } else { theme::status_err(ui, msg); }
        }

        // ── Right: preview ────────────────────────────────────────────────
        let ui = &mut cols[1];

        if app.selected_template_idx > 0 {
            if let Some(tex) = &app.template_preview_texture {
                ui.label(egui::RichText::new("Aperçu du thème").strong());
                ui.add_space(4.0);
                let avail = ui.available_width();
                let [tw, th] = [tex.size()[0] as f32, tex.size()[1] as f32];
                let scale   = (avail / tw).min(1.0);
                ui.add(egui::Image::new(tex).fit_to_exact_size(egui::vec2(tw * scale, th * scale)));
            } else {
                ui.label(egui::RichText::new("Aperçu").strong());
                ui.add_space(4.0);
                theme::hint(ui, "Rendu en cours…");
                show_egui_preview(app, ui);
            }
        } else {
            ui.label(egui::RichText::new("Aperçu").strong());
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
        "Aucun (défaut)".into()
    } else if app.selected_template_idx <= n {
        format!("Intégré : {}", template::BUILTIN[app.selected_template_idx - 1].name)
    } else if app.selected_template_idx == ci {
        "Personnalisé (fichier)".into()
    } else {
        let ri = app.selected_template_idx - rb;
        app.remote_templates.get(ri)
            .map(|t| format!("GitHub : {}", t.name))
            .unwrap_or_else(|| "—".into())
    };

    egui::ComboBox::from_id_source("tmpl_select")
        .selected_text(&cur_label)
        .width(220.0)
        .show_ui(ui, |ui| {
            // None
            if ui.selectable_label(app.selected_template_idx == 0, "Aucun (défaut)").clicked() {
                apply_template(app, 0);
                changed = true;
            }
            // Built-ins
            ui.separator();
            ui.label(egui::RichText::new("Intégrés").weak().small());
            for i in 0..n {
                let t   = &template::BUILTIN[i];
                let idx = i + 1;
                let lbl = format!("{} — {}", t.name, t.description);
                if ui.selectable_label(app.selected_template_idx == idx, lbl).clicked() {
                    apply_template(app, idx);
                    changed = true;
                }
            }
            // Custom
            if app.custom_template_svg.is_some() {
                ui.separator();
                if ui.selectable_label(app.selected_template_idx == ci,
                    "Personnalisé (fichier)").clicked()
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
                            app.remote_fetch_status = Some((true, "Téléchargement du thème…".into()));
                        }
                        changed = true;
                    }
                }
            }
        });

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        // Load custom SVG file
        if ui.button("\u{1F4C1} Fichier SVG").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("SVG template", &["svg"])
                .pick_file()
            {
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        app.custom_template_svg = Some(content);
                        apply_template(app, ci);
                        changed = true;
                    }
                    Err(e) => {
                        app.card_export_status = Some((false, format!("✗ Lecture : {e}")));
                    }
                }
            }
        }

        // Refresh remote template list from GitHub
        let fetching = app.remote_fetch_rx.is_some();
        let btn_label = if fetching { "\u{23F3}" } else { "\u{1F504}" };
        let btn = ui.add_enabled(
            !fetching,
            egui::Button::new(btn_label),
        ).on_hover_text("Actualiser les thèmes GitHub");
        if btn.clicked() {
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let _ = tx.send(crate::template::fetch_remote_index());
            });
            app.remote_fetch_rx = Some(rx);
            app.remote_fetch_status = Some((true, "Connexion à GitHub…".into()));
        }
    });

    // Remote status message
    if let Some((ok, msg)) = &app.remote_fetch_status {
        ui.add_space(2.0);
        if *ok { theme::hint(ui, msg); } else { theme::status_err(ui, msg); }
    }

    changed
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
        let labels   = app.card.layout.field_labels();
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
    };
    let qr_x = rect.left() + match app.card.layout {
        CardLayout::BusinessCard | CardLayout::Badge => dh * 0.09,
        _ => (dw - qr_sz) / 2.0,
    };
    let qr_y = rect.top() + match app.card.layout {
        CardLayout::BusinessCard              => dh * 0.09,
        CardLayout::Label | CardLayout::Flyer => dh * 0.06,
        CardLayout::Badge                     => (dh - qr_sz) / 2.0,
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

    if app.card.layout == CardLayout::BusinessCard {
        let bar_x = qr_x + qr_sz + dh * 0.04;
        painter.rect_filled(
            egui::Rect::from_min_size(egui::pos2(bar_x, qr_y), egui::vec2(2.0, qr_sz)),
            1.0, acc);
    }
    if app.card.layout == CardLayout::Badge {
        painter.rect_filled(
            egui::Rect::from_min_size(rect.left_top(), egui::vec2(dw, 8.0)),
            0.0, acc);
    }

    // Determine which fields to show
    let preview_fields: Vec<(&str, bool)> = if app.selected_template_idx > 0 && !app.template_field_data.is_empty() {
        app.template_field_data.iter().map(|tf| (tf.value.as_str(), tf.visible)).collect()
    } else {
        app.card.fields.iter().map(|f| (f.as_str(), true)).collect()
    };
    let labels = app.card.layout.field_labels();

    let text_x = match app.card.layout {
        CardLayout::BusinessCard => qr_x + qr_sz + dh * 0.07,
        CardLayout::Badge        => qr_x + qr_sz + 10.0,
        _ => rect.left() + dw / 2.0 - 60.0,
    };
    let mut ty = match app.card.layout {
        CardLayout::BusinessCard              => qr_y + 6.0,
        CardLayout::Badge                     => rect.top() + dh * 0.28,
        CardLayout::Label | CardLayout::Flyer => qr_y + qr_sz + 10.0,
    };

    for (i, (field, visible)) in preview_fields.iter().enumerate() {
        if !visible { continue; }
        let display = if field.is_empty() {
            labels.get(i).copied().unwrap_or("").to_string()
        } else {
            field.to_string()
        };
        let (fs, color) = match (app.card.layout, i) {
            (CardLayout::BusinessCard, 0) | (CardLayout::Badge, 0) | (CardLayout::Flyer, 0) => (14.0, fg_col),
            (_, 1) => (10.0, acc),
            _ => (9.0, fg_col),
        };
        let alpha = if field.is_empty() { 80u8 } else { 220u8 };
        let col   = egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha);
        let anchor = match app.card.layout {
            CardLayout::Label | CardLayout::Flyer => egui::Align2::CENTER_TOP,
            _ => egui::Align2::LEFT_TOP,
        };
        let tx = match app.card.layout {
            CardLayout::Label | CardLayout::Flyer => rect.center().x,
            _ => text_x,
        };
        painter.text(egui::pos2(tx, ty), anchor, &display, egui::FontId::proportional(fs), col);
        ty += fs + 3.0;
    }
}

// ─── Export functions ─────────────────────────────────────────────────────────

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

fn export_card_svg(app: &mut RustyQrApp) {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("SVG Vector", &["svg"])
        .set_file_name("carte_qr.svg")
        .save_file()
    else { return; };

    let matrix_ref = app.qr_matrix.as_ref();
    let profile    = app.current_profile().clone();

    let svg = match get_active_template_svg(app) {
        Some(tpl) => template::render(
            tpl, &app.card, matrix_ref, &profile,
            &app.template_field_data, &app.template_color_data,
        ),
        None => to_svg(&app.card, matrix_ref, &profile),
    };

    let p = path.to_string_lossy().into_owned();
    app.card_export_status = Some(match std::fs::write(&p, svg) {
        Ok(_)  => (true,  format!("✓ SVG exporté : {p}")),
        Err(e) => (false, format!("✗ {e}")),
    });
}

fn export_card_pdf(app: &mut RustyQrApp) {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("PDF Document", &["pdf"])
        .set_file_name("carte_qr.pdf")
        .save_file()
    else { return; };

    let matrix_ref = app.qr_matrix.as_ref();
    let profile    = app.current_profile().clone();
    let p          = path.to_string_lossy().into_owned();

    app.card_export_status = Some(match to_pdf(&app.card, matrix_ref, &profile) {
        Ok(bytes) => match std::fs::write(&p, bytes) {
            Ok(_)  => (true,  format!("✓ PDF exporté : {p}")),
            Err(e) => (false, format!("✗ écriture : {e}")),
        },
        Err(e) => (false, format!("✗ {e}")),
    });
}
