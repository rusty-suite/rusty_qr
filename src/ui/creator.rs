use egui::Ui;

use crate::app::RustyQrApp;
use crate::history;
use crate::qr::types::{EcLevel, QrContentType, WifiSecurity};
use crate::theme;

pub fn show(app: &mut RustyQrApp, ui: &mut Ui) {
    ui.add_space(8.0);
    theme::title(ui, &format!("\u{1F4C4} {}", app.lang.t("creator.title")));
    ui.separator();
    ui.add_space(8.0);

    let mut changed = false;

    // ── Type + EC level ──────────────────────────────────────────────────────
    egui::Grid::new("type_row").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
        ui.label(app.lang.t("creator.type"));
        egui::ComboBox::from_id_salt("qr_type")
            .selected_text(app.lang.t(app.form.content_type.label_key()))
            .show_ui(ui, |ui| {
                for &t in QrContentType::ALL {
                    if ui.selectable_label(app.form.content_type == t, app.lang.t(t.label_key())).clicked() {
                        app.form.content_type = t; changed = true;
                    }
                }
            });
        ui.end_row();

        ui.label(app.lang.t("creator.ec"));
        egui::ComboBox::from_id_salt("ec_level")
            .selected_text(app.form.ec_level.label())
            .show_ui(ui, |ui| {
                for ec in [EcLevel::L, EcLevel::M, EcLevel::Q, EcLevel::H] {
                    if ui.selectable_label(app.form.ec_level == ec, ec.label()).clicked() {
                        app.form.ec_level = ec; changed = true;
                    }
                }
            });
        ui.end_row();

        ui.label(app.lang.t("creator.micro"));
        if ui.checkbox(&mut app.form.use_micro_qr, app.lang.t("creator.micro_use")).changed() {
            changed = true;
        }
        ui.end_row();
    });

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);

    // ── Formulaire dynamique ─────────────────────────────────────────────────
    if show_form(app, ui) { changed = true; }

    // ── Données encodées (preview) ───────────────────────────────────────────
    let encoded = app.form.to_qr_string();
    ui.separator();
    ui.add_space(4.0);
    theme::hint(ui, &app.lang.t("creator.encoded_data"));
    ui.add(
        egui::TextEdit::multiline(&mut encoded.as_str())
            .desired_rows(2)
            .desired_width(f32::INFINITY)
            .font(egui::TextStyle::Monospace),
    );

    ui.add_space(8.0);

    // ── Actions ──────────────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        if ui.button(app.lang.t("creator.generate")).clicked() || changed {
            app.regenerate_qr();
        }

        ui.add_space(8.0);

        // Bouton "Enregistrer dans la bibliothèque"
        let save_label = if app.loaded_library_id.is_some() {
            format!("\u{1F4BE} {}", app.lang.t("creator.resave"))
        } else {
            format!("\u{1F4BE} {}", app.lang.t("creator.save_to_library"))
        };
        if ui.button(save_label).clicked() {
            app.show_save_dialog = true;
            if app.save_name_input.is_empty() {
                // Use display_name_hint which omits password fields
                app.save_name_input = app.form.display_name_hint(&app.lang);
            }
        }
    });

    // ── Dialogue de sauvegarde ───────────────────────────────────────────────
    if app.show_save_dialog {
        egui::Window::new(format!("\u{1F4BE} {}", app.lang.t("creator.save_to_library")))
            .collapsible(false)
            .resizable(false)
            .auto_sized()
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |ui| {
                ui.add_space(4.0);
                ui.label(app.lang.t("creator.entry_name"));
                ui.add_space(2.0);
                let response = ui.add(
                    egui::TextEdit::singleline(&mut app.save_name_input)
                        .desired_width(320.0)
                        .hint_text(app.lang.t("creator.entry_name_hint")),
                );
                response.request_focus();

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    let name = app.save_name_input.trim().to_string();
                    let can_save = !name.is_empty();

                    if ui.add_enabled(can_save, egui::Button::new(format!("\u{2713} {}", app.lang.t("creator.save")))).clicked()
                        || (ui.input(|i| i.key_pressed(egui::Key::Enter)) && can_save)
                    {
                        let tpl_state = build_template_state(app);
                        history::add_entry(&mut app.library, name, app.form.clone(), tpl_state);
                        app.show_save_dialog = false;
                        app.save_name_input.clear();
                    }

                    if ui.button(format!("\u{2715} {}", app.lang.t("creator.cancel"))).clicked() {
                        app.show_save_dialog = false;
                    }
                });
                ui.add_space(4.0);
            });
    }

    // Indicateur "chargé depuis la bibliothèque"
    if app.loaded_library_id.is_some() {
        ui.add_space(4.0);
        theme::hint(ui, &format!("\u{270F} {}", app.lang.t("creator.loaded_hint")));
    }
}

fn show_form(app: &mut RustyQrApp, ui: &mut Ui) -> bool {
    let mut changed = false;

    match app.form.content_type {
        QrContentType::Url => {
            field2(ui, &app.lang.t("creator.url"), &mut app.form.url, &mut changed);
            theme::hint(ui, "Ex: https://example.com");
        }

        QrContentType::Text => {
            ui.label(app.lang.t("creator.text"));
            if ui.add(egui::TextEdit::multiline(&mut app.form.text)
                .desired_rows(4).desired_width(f32::INFINITY)).changed() { changed = true; }
        }

        QrContentType::Wifi => {
            egui::Grid::new("wifi").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                lf(ui, &app.lang.t("creator.ssid"),          &mut app.form.wifi_ssid,     &mut changed);
                lf(ui, &app.lang.t("creator.password"),  &mut app.form.wifi_password, &mut changed);
                ui.label(app.lang.t("creator.security"));
                egui::ComboBox::from_id_salt("wifi_sec")
                    .selected_text(app.lang.t(app.form.wifi_security.label_key()))
                    .show_ui(ui, |ui| {
                        for s in [WifiSecurity::Wpa, WifiSecurity::Wep, WifiSecurity::None] {
                            if ui.selectable_label(app.form.wifi_security == s, app.lang.t(s.label_key())).clicked() {
                                app.form.wifi_security = s; changed = true;
                            }
                        }
                    });
                ui.end_row();
                ui.label(app.lang.t("creator.hidden_network"));
                if ui.checkbox(&mut app.form.wifi_hidden, "").changed() { changed = true; }
                ui.end_row();
            });
        }

        QrContentType::Sms => {
            egui::Grid::new("sms").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                lf(ui, &app.lang.t("creator.number"), &mut app.form.sms_number, &mut changed);
            });
            ui.label(app.lang.t("creator.message"));
            if ui.add(egui::TextEdit::multiline(&mut app.form.sms_message)
                .desired_rows(3).desired_width(f32::INFINITY)).changed() { changed = true; }
        }

        QrContentType::Tel => {
            field2(ui, &app.lang.t("creator.number"), &mut app.form.tel_number, &mut changed);
            theme::hint(ui, "Ex: +33612345678");
        }

        QrContentType::Email => {
            egui::Grid::new("email").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                lf(ui, &app.lang.t("creator.recipient"), &mut app.form.email_to,      &mut changed);
                lf(ui, &app.lang.t("creator.subject"),        &mut app.form.email_subject, &mut changed);
            });
            ui.label(app.lang.t("creator.body"));
            if ui.add(egui::TextEdit::multiline(&mut app.form.email_body)
                .desired_rows(4).desired_width(f32::INFINITY)).changed() { changed = true; }
        }

        QrContentType::Vcard => {
            egui::Grid::new("vcard").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                lf(ui, &app.lang.t("creator.name"),          &mut app.form.vcard_name,    &mut changed);
                lf(ui, &app.lang.t("creator.organization"), &mut app.form.vcard_org,     &mut changed);
                lf(ui, &app.lang.t("creator.phone"),    &mut app.form.vcard_phone,   &mut changed);
                lf(ui, &app.lang.t("creator.email"),        &mut app.form.vcard_email,   &mut changed);
                lf(ui, &app.lang.t("creator.website"),     &mut app.form.vcard_url,     &mut changed);
                lf(ui, &app.lang.t("creator.address"),      &mut app.form.vcard_address, &mut changed);
                lf(ui, &app.lang.t("creator.note"),         &mut app.form.vcard_note,    &mut changed);
            });
        }

        QrContentType::Mecard => {
            egui::Grid::new("mecard").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                lf(ui, &app.lang.t("creator.name"),        &mut app.form.mecard_name,  &mut changed);
                lf(ui, &app.lang.t("creator.phone"), &mut app.form.mecard_phone, &mut changed);
                lf(ui, &app.lang.t("creator.email"),     &mut app.form.mecard_email, &mut changed);
                lf(ui, &app.lang.t("creator.url"),       &mut app.form.mecard_url,   &mut changed);
            });
        }

        QrContentType::Geo => {
            egui::Grid::new("geo").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                lf(ui, &app.lang.t("creator.latitude"),       &mut app.form.geo_lat, &mut changed);
                lf(ui, &app.lang.t("creator.longitude"),      &mut app.form.geo_lon, &mut changed);
                lf(ui, &app.lang.t("creator.altitude"), &mut app.form.geo_alt, &mut changed);
            });
            theme::hint(ui, &app.lang.t("creator.geo_hint"));
        }

        QrContentType::Gs1 => {
            ui.label(app.lang.t("creator.gs1_data"));
            if ui.add(egui::TextEdit::multiline(&mut app.form.gs1_data)
                .desired_rows(3).desired_width(f32::INFINITY)).changed() { changed = true; }
            theme::hint(ui, &app.lang.t("creator.gs1_hint"));
        }

        QrContentType::TwoDoc => {
            egui::Grid::new("twodoc").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                lf(ui, &app.lang.t("creator.cert_id"), &mut app.form.twod_cert_id, &mut changed);
                lf(ui, &app.lang.t("creator.c40_data"), &mut app.form.twod_c40,    &mut changed);
            });
            theme::status_warn(ui, &format!("\u{26A0} {}", app.lang.t("creator.twodoc_warning")));
        }
    }

    changed
}

/// Ligne de grille label + champ texte
fn lf(ui: &mut egui::Ui, label: &str, field: &mut String, changed: &mut bool) {
    ui.label(label);
    if ui.text_edit_singleline(field).changed() { *changed = true; }
    ui.end_row();
}

/// Champ simple hors grille
fn field2(ui: &mut egui::Ui, label: &str, field: &mut String, changed: &mut bool) {
    egui::Grid::new(label).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
        lf(ui, label, field, changed);
    });
}

fn build_template_state(app: &crate::app::RustyQrApp) -> crate::history::SavedTemplateState {
    use crate::history::{SavedColor, SavedField, SavedTemplateState};
    use crate::template::BUILTIN;

    let builtin_id = if app.selected_template_idx > 0
        && app.selected_template_idx <= BUILTIN.len()
    {
        Some(BUILTIN[app.selected_template_idx - 1].id.to_string())
    } else {
        None
    };

    let custom_svg = if app.selected_template_idx == BUILTIN.len() + 1 {
        app.custom_template_svg.clone()
    } else {
        None
    };

    let fields = app.template_field_data.iter().map(|f| SavedField {
        var:     f.var.clone(),
        value:   f.value.clone(),
        visible: f.visible,
    }).collect();

    let colors = app.template_color_data.iter().map(|c| SavedColor {
        var:   c.var.clone(),
        value: c.value,
    }).collect();

    let card = Some(app.card.clone());

    SavedTemplateState { builtin_id, custom_svg, fields, colors, card }
}
