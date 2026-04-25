use egui::Ui;

use crate::app::RustyQrApp;
use crate::history;
use crate::qr::types::{EcLevel, QrContentType, WifiSecurity};
use crate::theme;

pub fn show(app: &mut RustyQrApp, ui: &mut Ui) {
    ui.add_space(8.0);
    theme::title(ui, "📄 Créer un code QR");
    ui.separator();
    ui.add_space(8.0);

    let mut changed = false;

    // ── Type + EC level ──────────────────────────────────────────────────────
    egui::Grid::new("type_row").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
        ui.label("Type de QR :");
        egui::ComboBox::from_id_source("qr_type")
            .selected_text(app.form.content_type.label())
            .show_ui(ui, |ui| {
                for &t in QrContentType::ALL {
                    if ui.selectable_label(app.form.content_type == t, t.label()).clicked() {
                        app.form.content_type = t; changed = true;
                    }
                }
            });
        ui.end_row();

        ui.label("Correction d'erreur :");
        egui::ComboBox::from_id_source("ec_level")
            .selected_text(app.form.ec_level.label())
            .show_ui(ui, |ui| {
                for ec in [EcLevel::L, EcLevel::M, EcLevel::Q, EcLevel::H] {
                    if ui.selectable_label(app.form.ec_level == ec, ec.label()).clicked() {
                        app.form.ec_level = ec; changed = true;
                    }
                }
            });
        ui.end_row();

        ui.label("Micro QR :");
        if ui.checkbox(&mut app.form.use_micro_qr, "Utiliser Micro QR (M1–M4, données courtes)").changed() {
            changed = true;
        }
        ui.end_row();
    });

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);

    // ── Formulaire dynamique ─────────────────────────────────────────────────
    egui::ScrollArea::vertical().max_height(340.0).show(ui, |ui| {
        if show_form(app, ui) { changed = true; }
    });

    // ── Données encodées (preview) ───────────────────────────────────────────
    let encoded = app.form.to_qr_string();
    ui.separator();
    ui.add_space(4.0);
    theme::hint(ui, "Données encodées :");
    ui.add(
        egui::TextEdit::multiline(&mut encoded.as_str())
            .desired_rows(2)
            .desired_width(f32::INFINITY)
            .font(egui::TextStyle::Monospace),
    );

    ui.add_space(8.0);

    // ── Actions ──────────────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        if ui.button("▶ Générer").clicked() || changed {
            app.regenerate_qr();
        }

        ui.add_space(8.0);

        // Bouton "Enregistrer dans la bibliothèque"
        let save_label = if app.loaded_library_id.is_some() {
            "💾 Ré-enregistrer"
        } else {
            "💾 Enregistrer dans la bibliothèque"
        };
        if ui.button(save_label).clicked() {
            app.show_save_dialog = true;
            // Propose un nom basé sur le type et un extrait
            if app.save_name_input.is_empty() {
                let snippet = encoded.chars().take(30).collect::<String>();
                app.save_name_input = format!("{} — {}", app.form.content_type.label(), snippet);
            }
        }
    });

    // ── Dialogue de sauvegarde ───────────────────────────────────────────────
    if app.show_save_dialog {
        egui::Window::new("💾 Enregistrer dans la bibliothèque")
            .collapsible(false)
            .resizable(false)
            .auto_sized()
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |ui| {
                ui.add_space(4.0);
                ui.label("Nom de l'entrée :");
                ui.add_space(2.0);
                let response = ui.add(
                    egui::TextEdit::singleline(&mut app.save_name_input)
                        .desired_width(320.0)
                        .hint_text("Ex: QR WiFi bureau, carte de contact…"),
                );
                response.request_focus();

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    let name = app.save_name_input.trim().to_string();
                    let can_save = !name.is_empty();

                    if ui.add_enabled(can_save, egui::Button::new("✓ Enregistrer")).clicked()
                        || (ui.input(|i| i.key_pressed(egui::Key::Enter)) && can_save)
                    {
                        history::add_entry(&mut app.library, name, app.form.clone());
                        app.show_save_dialog = false;
                        app.save_name_input.clear();
                    }

                    if ui.button("✕ Annuler").clicked() {
                        app.show_save_dialog = false;
                    }
                });
                ui.add_space(4.0);
            });
    }

    // Indicateur "chargé depuis la bibliothèque"
    if app.loaded_library_id.is_some() {
        ui.add_space(4.0);
        theme::hint(ui, "✎ Configuration chargée depuis la bibliothèque — modifiez puis ré-enregistrez.");
    }
}

fn show_form(app: &mut RustyQrApp, ui: &mut Ui) -> bool {
    let mut changed = false;

    match app.form.content_type {
        QrContentType::Url => {
            field2(ui, "URL :", &mut app.form.url, &mut changed);
            theme::hint(ui, "Ex: https://example.com");
        }

        QrContentType::Text => {
            ui.label("Texte :");
            if ui.add(egui::TextEdit::multiline(&mut app.form.text)
                .desired_rows(4).desired_width(f32::INFINITY)).changed() { changed = true; }
        }

        QrContentType::Wifi => {
            egui::Grid::new("wifi").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                lf(ui, "SSID :",          &mut app.form.wifi_ssid,     &mut changed);
                lf(ui, "Mot de passe :",  &mut app.form.wifi_password, &mut changed);
                ui.label("Sécurité :");
                egui::ComboBox::from_id_source("wifi_sec")
                    .selected_text(app.form.wifi_security.label())
                    .show_ui(ui, |ui| {
                        for s in [WifiSecurity::Wpa, WifiSecurity::Wep, WifiSecurity::None] {
                            if ui.selectable_label(app.form.wifi_security == s, s.label()).clicked() {
                                app.form.wifi_security = s; changed = true;
                            }
                        }
                    });
                ui.end_row();
                ui.label("Réseau caché :");
                if ui.checkbox(&mut app.form.wifi_hidden, "").changed() { changed = true; }
                ui.end_row();
            });
        }

        QrContentType::Sms => {
            egui::Grid::new("sms").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                lf(ui, "Numéro :", &mut app.form.sms_number, &mut changed);
            });
            ui.label("Message :");
            if ui.add(egui::TextEdit::multiline(&mut app.form.sms_message)
                .desired_rows(3).desired_width(f32::INFINITY)).changed() { changed = true; }
        }

        QrContentType::Tel => {
            field2(ui, "Numéro :", &mut app.form.tel_number, &mut changed);
            theme::hint(ui, "Ex: +33612345678");
        }

        QrContentType::Email => {
            egui::Grid::new("email").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                lf(ui, "Destinataire :", &mut app.form.email_to,      &mut changed);
                lf(ui, "Sujet :",        &mut app.form.email_subject, &mut changed);
            });
            ui.label("Corps :");
            if ui.add(egui::TextEdit::multiline(&mut app.form.email_body)
                .desired_rows(4).desired_width(f32::INFINITY)).changed() { changed = true; }
        }

        QrContentType::Vcard => {
            egui::Grid::new("vcard").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                lf(ui, "Nom :",          &mut app.form.vcard_name,    &mut changed);
                lf(ui, "Organisation :", &mut app.form.vcard_org,     &mut changed);
                lf(ui, "Téléphone :",    &mut app.form.vcard_phone,   &mut changed);
                lf(ui, "Email :",        &mut app.form.vcard_email,   &mut changed);
                lf(ui, "Site web :",     &mut app.form.vcard_url,     &mut changed);
                lf(ui, "Adresse :",      &mut app.form.vcard_address, &mut changed);
                lf(ui, "Note :",         &mut app.form.vcard_note,    &mut changed);
            });
        }

        QrContentType::Mecard => {
            egui::Grid::new("mecard").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                lf(ui, "Nom :",        &mut app.form.mecard_name,  &mut changed);
                lf(ui, "Téléphone :", &mut app.form.mecard_phone, &mut changed);
                lf(ui, "Email :",     &mut app.form.mecard_email, &mut changed);
                lf(ui, "URL :",       &mut app.form.mecard_url,   &mut changed);
            });
        }

        QrContentType::Geo => {
            egui::Grid::new("geo").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                lf(ui, "Latitude :",       &mut app.form.geo_lat, &mut changed);
                lf(ui, "Longitude :",      &mut app.form.geo_lon, &mut changed);
                lf(ui, "Altitude (opt) :", &mut app.form.geo_alt, &mut changed);
            });
            theme::hint(ui, "Ex: 48.8566, 2.3522  (Paris)");
        }

        QrContentType::Gs1 => {
            ui.label("Données GS1 (Application Identifiers + valeurs) :");
            if ui.add(egui::TextEdit::multiline(&mut app.form.gs1_data)
                .desired_rows(3).desired_width(f32::INFINITY)).changed() { changed = true; }
            theme::hint(ui, "Ex: (01)09521234543213(17)210630(10)ABC123");
        }

        QrContentType::TwoDoc => {
            egui::Grid::new("twodoc").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
                lf(ui, "ID Certificat :", &mut app.form.twod_cert_id, &mut changed);
                lf(ui, "Données (C40) :", &mut app.form.twod_c40,    &mut changed);
            });
            theme::status_warn(ui, "⚠ 2D-Doc généré sans signature cryptographique (informatif).");
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
