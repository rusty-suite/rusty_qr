//! Concepteur de cartes — mise en page QR (carte de visite, étiquette, badge, flyer).

use egui::Ui;
use crate::app::RustyQrApp;
use crate::card::{CardLayout, to_svg, to_pdf};
use crate::template;
use crate::theme;

// Index constants derived from the number of built-in templates
const fn custom_idx() -> usize { template::BUILTIN.len() + 1 }
const fn remote_base() -> usize { template::BUILTIN.len() + 2 }

pub fn show(app: &mut RustyQrApp, ui: &mut Ui) {
    ui.add_space(8.0);
    theme::title(ui, "🪪 Concepteur de cartes");
    theme::hint(ui, "Mettez en page votre code QR sur une carte, étiquette ou flyer — puis exportez.");
    ui.separator();
    ui.add_space(8.0);

    ui.columns(2, |cols| {
        // ── Left: settings ───────────────────────────────────────────────────
        let ui = &mut cols[0];

        // ── Thème SVG ────────────────────────────────────────────────────────
        ui.label(egui::RichText::new("Thème SVG").strong());
        ui.add_space(4.0);
        show_template_selector(app, ui);

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
                color_edit(ui, &mut card.bg_color);
                ui.end_row();

                ui.label("Texte :");
                color_edit(ui, &mut card.text_color);
                ui.end_row();

                ui.label("Accent :");
                color_edit(ui, &mut card.accent_color);
                ui.end_row();
            });

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(8.0);

        // ── Contenu ──────────────────────────────────────────────────────────
        ui.label(egui::RichText::new("Contenu").strong());
        ui.add_space(4.0);

        let labels = card.layout.field_labels();
        let field_count = card.fields.len();
        for i in 0..field_count {
            let label = labels.get(i).copied().unwrap_or("Champ");
            ui.label(egui::RichText::new(label).small().weak());
            ui.text_edit_singleline(&mut card.fields[i]);
            ui.add_space(2.0);
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
        ui.label(egui::RichText::new("Aperçu").strong());
        ui.add_space(4.0);

        show_preview(app, ui);
    });
}

// ─── Template selector ────────────────────────────────────────────────────────

fn show_template_selector(app: &mut RustyQrApp, ui: &mut Ui) {
    let builtin_count = template::BUILTIN.len();
    let custom_idx    = custom_idx();
    let remote_base   = remote_base();

    // Current label for the combo box
    let current_label: String = if app.selected_template_idx == 0 {
        "Aucun (défaut)".into()
    } else if app.selected_template_idx <= builtin_count {
        let t = &template::BUILTIN[app.selected_template_idx - 1];
        format!("Intégré : {}", t.name)
    } else if app.selected_template_idx == custom_idx {
        "Personnalisé (fichier chargé)".into()
    } else {
        let ri = app.selected_template_idx - remote_base;
        app.remote_templates.get(ri)
            .map(|t| format!("GitHub : {}", t.name))
            .unwrap_or_else(|| "—".into())
    };

    egui::ComboBox::from_id_source("template_select")
        .selected_text(&current_label)
        .show_ui(ui, |ui| {
            if ui.selectable_label(app.selected_template_idx == 0, "Aucun (défaut)").clicked() {
                app.selected_template_idx = 0;
            }
            ui.separator();
            ui.label(egui::RichText::new("Intégrés").weak().small());
            for (i, t) in template::BUILTIN.iter().enumerate() {
                let idx = i + 1;
                if ui.selectable_label(app.selected_template_idx == idx,
                    format!("{} — {}", t.name, t.description)).clicked()
                {
                    app.selected_template_idx = idx;
                }
            }
            if app.custom_template_svg.is_some() {
                ui.separator();
                if ui.selectable_label(app.selected_template_idx == custom_idx,
                    "Personnalisé (fichier chargé)").clicked()
                {
                    app.selected_template_idx = custom_idx;
                }
            }
            if !app.remote_templates.is_empty() {
                ui.separator();
                ui.label(egui::RichText::new("GitHub").weak().small());
                for (i, t) in app.remote_templates.iter().enumerate() {
                    let idx = remote_base + i;
                    let label = if t.svg.is_some() {
                        format!("{} — {}", t.name, t.description)
                    } else {
                        format!("{} — {} (non téléchargé)", t.name, t.description)
                    };
                    if ui.selectable_label(app.selected_template_idx == idx, label).clicked() {
                        app.selected_template_idx = idx;
                    }
                }
            }
        });

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        // Load custom file
        if ui.button("📁 Charger un fichier SVG").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("SVG template", &["svg"])
                .pick_file()
            {
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        app.custom_template_svg = Some(content);
                        app.selected_template_idx = custom_idx;
                    }
                    Err(e) => {
                        app.card_export_status = Some((false, format!("✗ Lecture : {e}")));
                    }
                }
            }
        }

        // Fetch remote index
        if app.remote_fetch_rx.is_none() {
            if ui.button("🌐 GitHub").on_hover_text("Actualiser la liste des thèmes distants").clicked() {
                let (tx, rx) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    let _ = tx.send(crate::template::fetch_remote_index());
                });
                app.remote_fetch_rx = Some(rx);
                app.remote_fetch_status = Some((true, "Connexion à GitHub…".into()));
            }
        } else {
            ui.add_enabled(false, egui::Button::new("🌐 Chargement…"));
        }
    });

    // Download button for a selected remote template that hasn't been fetched yet
    if app.selected_template_idx >= remote_base && app.remote_svg_dl.is_none() {
        let ri = app.selected_template_idx - remote_base;
        if let Some(rt) = app.remote_templates.get(ri) {
            if rt.svg.is_none() {
                let file = rt.file.clone();
                if ui.button("⬇ Télécharger ce thème").clicked() {
                    let (tx, rx) = std::sync::mpsc::channel();
                    std::thread::spawn(move || {
                        let _ = tx.send(crate::template::fetch_remote_svg(&file));
                    });
                    app.remote_svg_dl = Some((ri, rx));
                }
            } else {
                theme::status_ok(ui, "✓ Thème disponible");
            }
        }
    }

    if let Some((ok, msg)) = &app.remote_fetch_status {
        if *ok { theme::hint(ui, msg); } else { theme::status_err(ui, msg); }
    }
}

// ─── Color editor helper ──────────────────────────────────────────────────────

fn color_edit(ui: &mut Ui, c: &mut [u8; 3]) {
    let mut f = [c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0];
    if ui.color_edit_button_rgb(&mut f).changed() {
        *c = [(f[0] * 255.0) as u8, (f[1] * 255.0) as u8, (f[2] * 255.0) as u8];
    }
}

// ─── Preview (egui painter — layout-based, not template-based) ───────────────

fn show_preview(app: &RustyQrApp, ui: &mut Ui) {
    let (w_px, h_px) = app.card.layout.canvas_px();
    let avail = ui.available_width() - 16.0;
    let scale = (avail / w_px as f32).min(1.0).min(600.0 / h_px as f32);
    let dw = w_px as f32 * scale;
    let dh = h_px as f32 * scale;

    let (rect, _) = ui.allocate_exact_size(egui::vec2(dw, dh), egui::Sense::hover());
    let painter = ui.painter();

    let bg = egui::Color32::from_rgb(app.card.bg_color[0], app.card.bg_color[1], app.card.bg_color[2]);
    let acc = egui::Color32::from_rgb(app.card.accent_color[0], app.card.accent_color[1], app.card.accent_color[2]);
    let fg_col = egui::Color32::from_rgb(app.card.text_color[0], app.card.text_color[1], app.card.text_color[2]);

    painter.rect_filled(rect, 4.0, bg);
    painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0, egui::Color32::from_gray(80)));

    // Template indicator overlay
    if app.selected_template_idx > 0 {
        let label = get_selected_template_label(app);
        painter.text(
            rect.right_top() + egui::vec2(-4.0, 4.0),
            egui::Align2::RIGHT_TOP,
            format!("🎨 {label}"),
            egui::FontId::proportional(9.0),
            egui::Color32::from_rgba_unmultiplied(200, 200, 100, 180),
        );
    }

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
        CardLayout::BusinessCard => dh * 0.09,
        CardLayout::Label | CardLayout::Flyer => dh * 0.06,
        CardLayout::Badge => (dh - qr_sz) / 2.0,
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
            1.0, acc,
        );
    }
    if app.card.layout == CardLayout::Badge {
        painter.rect_filled(
            egui::Rect::from_min_size(rect.left_top(), egui::vec2(dw, 8.0)),
            0.0, acc,
        );
    }

    let text_x = match app.card.layout {
        CardLayout::BusinessCard => qr_x + qr_sz + dh * 0.07,
        CardLayout::Badge        => qr_x + qr_sz + 10.0,
        _ => rect.left() + dw / 2.0 - 60.0,
    };
    let text_y_start = match app.card.layout {
        CardLayout::BusinessCard => qr_y + 6.0,
        CardLayout::Badge        => rect.top() + dh * 0.28,
        CardLayout::Label | CardLayout::Flyer => qr_y + qr_sz + 10.0,
    };

    let labels = app.card.layout.field_labels();
    let mut ty = text_y_start;
    for (i, field) in app.card.fields.iter().enumerate() {
        let display = if field.is_empty() {
            labels.get(i).copied().unwrap_or("").to_string()
        } else {
            field.clone()
        };
        let (fs, color) = match (app.card.layout, i) {
            (CardLayout::BusinessCard, 0) | (CardLayout::Badge, 0) | (CardLayout::Flyer, 0) => (14.0, fg_col),
            (_, 1) => (10.0, acc),
            _ => (9.0, fg_col),
        };
        let alpha = if field.is_empty() { 80 } else { 220 };
        let col = egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha);
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

// ─── Template resolution ─────────────────────────────────────────────────────

fn get_selected_template_svg(app: &RustyQrApp) -> Option<&str> {
    let builtin_count = template::BUILTIN.len();
    let custom_idx    = custom_idx();
    let remote_base   = remote_base();

    if app.selected_template_idx == 0 {
        None
    } else if app.selected_template_idx <= builtin_count {
        Some(template::BUILTIN[app.selected_template_idx - 1].svg)
    } else if app.selected_template_idx == custom_idx {
        app.custom_template_svg.as_deref()
    } else {
        let ri = app.selected_template_idx - remote_base;
        app.remote_templates.get(ri).and_then(|t| t.svg.as_deref())
    }
}

fn get_selected_template_label(app: &RustyQrApp) -> String {
    let builtin_count = template::BUILTIN.len();
    let custom_idx    = custom_idx();
    let remote_base   = remote_base();

    if app.selected_template_idx == 0 {
        "Aucun".into()
    } else if app.selected_template_idx <= builtin_count {
        template::BUILTIN[app.selected_template_idx - 1].name.to_string()
    } else if app.selected_template_idx == custom_idx {
        "Personnalisé".into()
    } else {
        let ri = app.selected_template_idx - remote_base;
        app.remote_templates.get(ri).map(|t| t.name.clone()).unwrap_or_default()
    }
}

// ─── Export functions ─────────────────────────────────────────────────────────

fn export_card_svg(app: &mut RustyQrApp) {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("SVG Vector", &["svg"])
        .set_file_name("carte_qr.svg")
        .save_file()
    else { return; };

    let matrix_ref = app.qr_matrix.as_ref();
    let profile    = app.current_profile().clone();

    let svg = match get_selected_template_svg(app) {
        Some(tpl) => template::render(tpl, &app.card, matrix_ref, &profile),
        None      => to_svg(&app.card, matrix_ref, &profile),
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
