//! Concepteur de cartes — mise en page QR (carte de visite, étiquette, badge, flyer).

use egui::Ui;
use crate::app::RustyQrApp;
use crate::card::{CardLayout, to_svg, to_pdf};
use crate::theme;

pub fn show(app: &mut RustyQrApp, ui: &mut Ui) {
    ui.add_space(8.0);
    theme::title(ui, "🪪 Concepteur de cartes");
    theme::hint(ui, "Mettez en page votre code QR sur une carte, étiquette ou flyer — puis exportez.");
    ui.separator();
    ui.add_space(8.0);

    ui.columns(2, |cols| {
        // ── Left: settings ───────────────────────────────────────────────────
        let ui = &mut cols[0];
        let card = &mut app.card;

        ui.label(egui::RichText::new("Gabarit").strong());
        ui.add_space(4.0);

        egui::ComboBox::from_id_source("card_layout")
            .selected_text(card.layout.label())
            .show_ui(ui, |ui| {
                for &lay in CardLayout::ALL {
                    if ui.selectable_label(card.layout == lay, lay.label()).clicked() {
                        let old_fields = card.fields.clone();
                        *card = crate::card::CardConfig::new(lay);
                        // Preserve filled fields if count matches
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

        // Couleurs
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

        // Champs texte
        ui.label(egui::RichText::new("Contenu").strong());
        ui.add_space(4.0);

        let labels = card.layout.field_labels();
        // need to separate borrow
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

        // Export
        ui.label(egui::RichText::new("Exporter la carte").strong());
        ui.add_space(4.0);

        if app.qr_matrix.is_none() {
            theme::status_warn(ui, "⚠ Générez d'abord un QR dans l'onglet Créer.");
        }

        ui.horizontal(|ui| {
            if ui.button("SVG").clicked() {
                export_card_svg(app);
            }
            if ui.button("PDF").clicked() {
                export_card_pdf(app);
            }
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

fn color_edit(ui: &mut Ui, c: &mut [u8; 3]) {
    let mut f = [c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0];
    if ui.color_edit_button_rgb(&mut f).changed() {
        *c = [(f[0] * 255.0) as u8, (f[1] * 255.0) as u8, (f[2] * 255.0) as u8];
    }
}

fn show_preview(app: &RustyQrApp, ui: &mut Ui) {
    let (w_px, h_px) = app.card.layout.canvas_px();
    let avail = ui.available_width() - 16.0;
    let scale = (avail / w_px as f32).min(1.0).min(600.0 / h_px as f32);
    let dw = w_px as f32 * scale;
    let dh = h_px as f32 * scale;

    let (rect, _) = ui.allocate_exact_size(egui::vec2(dw, dh), egui::Sense::hover());
    let painter = ui.painter();

    // Background
    let bg = egui::Color32::from_rgb(
        app.card.bg_color[0], app.card.bg_color[1], app.card.bg_color[2],
    );
    let acc = egui::Color32::from_rgb(
        app.card.accent_color[0], app.card.accent_color[1], app.card.accent_color[2],
    );
    let fg_col = egui::Color32::from_rgb(
        app.card.text_color[0], app.card.text_color[1], app.card.text_color[2],
    );
    painter.rect_filled(rect, 4.0, bg);
    painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0, egui::Color32::from_gray(80)));

    // QR zone placeholder or texture
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
        painter.image(tex.id(), qr_rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
    } else {
        painter.rect_filled(qr_rect, 2.0, egui::Color32::from_gray(180));
        painter.text(qr_rect.center(), egui::Align2::CENTER_CENTER, "QR", egui::FontId::proportional(12.0), egui::Color32::from_gray(100));
    }

    // Accent bar (business card layout)
    if app.card.layout == CardLayout::BusinessCard {
        let bar_x = qr_x + qr_sz + dh * 0.04;
        painter.rect_filled(
            egui::Rect::from_min_size(egui::pos2(bar_x, qr_y), egui::vec2(2.0, qr_sz)),
            1.0, acc,
        );
    }

    // Badge accent strip
    if app.card.layout == CardLayout::Badge {
        painter.rect_filled(
            egui::Rect::from_min_size(rect.left_top(), egui::vec2(dw, 8.0)),
            0.0, acc,
        );
    }

    // Text zones preview
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

fn export_card_svg(app: &mut RustyQrApp) {
    let ext = "svg";
    let Some(path) = rfd::FileDialog::new()
        .add_filter("SVG Vector", &[ext])
        .set_file_name(&format!("carte_qr.{ext}"))
        .save_file()
    else { return; };

    let matrix_ref = app.qr_matrix.as_ref();
    let profile = app.current_profile().clone();
    let svg = to_svg(&app.card, matrix_ref, &profile);
    let p = path.to_string_lossy().into_owned();
    app.card_export_status = Some(match std::fs::write(&p, svg) {
        Ok(_)  => (true,  format!("✓ SVG exporté : {p}")),
        Err(e) => (false, format!("✗ {e}")),
    });
}

fn export_card_pdf(app: &mut RustyQrApp) {
    let ext = "pdf";
    let Some(path) = rfd::FileDialog::new()
        .add_filter("PDF Document", &[ext])
        .set_file_name(&format!("carte_qr.{ext}"))
        .save_file()
    else { return; };

    let matrix_ref = app.qr_matrix.as_ref();
    let profile = app.current_profile().clone();
    let p = path.to_string_lossy().into_owned();
    app.card_export_status = Some(match to_pdf(&app.card, matrix_ref, &profile) {
        Ok(bytes) => match std::fs::write(&p, bytes) {
            Ok(_)  => (true,  format!("✓ PDF exporté : {p}")),
            Err(e) => (false, format!("✗ écriture : {e}")),
        },
        Err(e) => (false, format!("✗ {e}")),
    });
}
