use egui::Ui;

use crate::app::RustyQrApp;
use crate::qr::types::{EcLevel, QrContentType, WifiSecurity};
use crate::theme;

pub fn show(app: &mut RustyQrApp, ui: &mut Ui) {
    ui.add_space(8.0);
    theme::title(ui, "Créer un code QR");
    ui.separator();
    ui.add_space(8.0);

    let mut changed = false;

    // ── Type selector ────────────────────────────────────────────────────────
    egui::Grid::new("type_row")
        .num_columns(2)
        .spacing([8.0, 4.0])
        .show(ui, |ui| {
            ui.label("Type de QR :");
            egui::ComboBox::from_id_source("qr_type")
                .selected_text(app.form.content_type.label())
                .show_ui(ui, |ui| {
                    for &t in QrContentType::ALL {
                        if ui.selectable_label(app.form.content_type == t, t.label()).clicked() {
                            app.form.content_type = t;
                            changed = true;
                        }
                    }
                });
            ui.end_row();

            ui.label("Correction :");
            egui::ComboBox::from_id_source("ec_level")
                .selected_text(app.form.ec_level.label())
                .show_ui(ui, |ui| {
                    for ec in [EcLevel::L, EcLevel::M, EcLevel::Q, EcLevel::H] {
                        if ui.selectable_label(app.form.ec_level == ec, ec.label()).clicked() {
                            app.form.ec_level = ec;
                            changed = true;
                        }
                    }
                });
            ui.end_row();

            ui.label("Micro QR :");
            if ui.checkbox(&mut app.form.use_micro_qr, "Utiliser Micro QR (M1–M4)").changed() {
                changed = true;
            }
            ui.end_row();
        });

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);

    // ── Form fields (dynamic per type) ──────────────────────────────────────
    egui::ScrollArea::vertical().show(ui, |ui| {
        let c = show_form(app, ui);
        if c { changed = true; }
    });

    ui.add_space(8.0);

    // ── Encoded string preview ───────────────────────────────────────────────
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
    if ui.button("▶ Générer le code QR").clicked() || changed {
        app.regenerate_qr();
    }
}

fn show_form(app: &mut RustyQrApp, ui: &mut Ui) -> bool {
    let mut changed = false;

    macro_rules! field {
        ($label:expr, $field:expr) => {{
            egui::Grid::new(concat!("grid_", $label))
                .num_columns(2)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    ui.label($label);
                    if ui.text_edit_singleline(&mut $field).changed() { changed = true; }
                    ui.end_row();
                });
        }};
        ($label:expr, $field:expr, multiline) => {{
            ui.label($label);
            if ui.add(egui::TextEdit::multiline(&mut $field).desired_rows(3).desired_width(f32::INFINITY)).changed() {
                changed = true;
            }
            ui.add_space(4.0);
        }};
    }

    match app.form.content_type {
        QrContentType::Url => {
            field!("URL :", app.form.url);
            theme::hint(ui, "Ex: https://example.com");
        }

        QrContentType::Text => {
            ui.label("Texte :");
            if ui.add(egui::TextEdit::multiline(&mut app.form.text)
                .desired_rows(4)
                .desired_width(f32::INFINITY))
                .changed() { changed = true; }
        }

        QrContentType::Wifi => {
            egui::Grid::new("wifi_grid")
                .num_columns(2)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    ui.label("SSID :");
                    if ui.text_edit_singleline(&mut app.form.wifi_ssid).changed() { changed = true; }
                    ui.end_row();

                    ui.label("Mot de passe :");
                    if ui.text_edit_singleline(&mut app.form.wifi_password).changed() { changed = true; }
                    ui.end_row();

                    ui.label("Sécurité :");
                    egui::ComboBox::from_id_source("wifi_sec")
                        .selected_text(app.form.wifi_security.label())
                        .show_ui(ui, |ui| {
                            for s in [WifiSecurity::Wpa, WifiSecurity::Wep, WifiSecurity::None] {
                                if ui.selectable_label(app.form.wifi_security == s, s.label()).clicked() {
                                    app.form.wifi_security = s;
                                    changed = true;
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
            field!("Numéro :", app.form.sms_number);
            field!("Message :", app.form.sms_message, multiline);
        }

        QrContentType::Tel => {
            field!("Numéro :", app.form.tel_number);
            theme::hint(ui, "Ex: +33612345678");
        }

        QrContentType::Email => {
            field!("Destinataire :", app.form.email_to);
            field!("Sujet :", app.form.email_subject);
            ui.label("Corps :");
            if ui.add(egui::TextEdit::multiline(&mut app.form.email_body)
                .desired_rows(4)
                .desired_width(f32::INFINITY))
                .changed() { changed = true; }
        }

        QrContentType::Vcard => {
            egui::Grid::new("vcard_grid")
                .num_columns(2)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Nom :");
                    if ui.text_edit_singleline(&mut app.form.vcard_name).changed() { changed = true; }
                    ui.end_row();
                    ui.label("Organisation :");
                    if ui.text_edit_singleline(&mut app.form.vcard_org).changed() { changed = true; }
                    ui.end_row();
                    ui.label("Téléphone :");
                    if ui.text_edit_singleline(&mut app.form.vcard_phone).changed() { changed = true; }
                    ui.end_row();
                    ui.label("Email :");
                    if ui.text_edit_singleline(&mut app.form.vcard_email).changed() { changed = true; }
                    ui.end_row();
                    ui.label("Site web :");
                    if ui.text_edit_singleline(&mut app.form.vcard_url).changed() { changed = true; }
                    ui.end_row();
                    ui.label("Adresse :");
                    if ui.text_edit_singleline(&mut app.form.vcard_address).changed() { changed = true; }
                    ui.end_row();
                    ui.label("Note :");
                    if ui.text_edit_singleline(&mut app.form.vcard_note).changed() { changed = true; }
                    ui.end_row();
                });
        }

        QrContentType::Mecard => {
            egui::Grid::new("mecard_grid")
                .num_columns(2)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Nom :");
                    if ui.text_edit_singleline(&mut app.form.mecard_name).changed() { changed = true; }
                    ui.end_row();
                    ui.label("Téléphone :");
                    if ui.text_edit_singleline(&mut app.form.mecard_phone).changed() { changed = true; }
                    ui.end_row();
                    ui.label("Email :");
                    if ui.text_edit_singleline(&mut app.form.mecard_email).changed() { changed = true; }
                    ui.end_row();
                    ui.label("URL :");
                    if ui.text_edit_singleline(&mut app.form.mecard_url).changed() { changed = true; }
                    ui.end_row();
                });
        }

        QrContentType::Geo => {
            egui::Grid::new("geo_grid")
                .num_columns(2)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Latitude :");
                    if ui.text_edit_singleline(&mut app.form.geo_lat).changed() { changed = true; }
                    ui.end_row();
                    ui.label("Longitude :");
                    if ui.text_edit_singleline(&mut app.form.geo_lon).changed() { changed = true; }
                    ui.end_row();
                    ui.label("Altitude (opt.) :");
                    if ui.text_edit_singleline(&mut app.form.geo_alt).changed() { changed = true; }
                    ui.end_row();
                });
            theme::hint(ui, "Ex: 48.8566, 2.3522");
        }

        QrContentType::Gs1 => {
            ui.label("Données GS1 (AI + valeurs) :");
            if ui.add(egui::TextEdit::multiline(&mut app.form.gs1_data)
                .desired_rows(3)
                .desired_width(f32::INFINITY))
                .changed() { changed = true; }
            theme::hint(ui, "Ex: (01)09521234543213(17)210630(10)ABC123");
        }

        QrContentType::TwoDoc => {
            egui::Grid::new("twodoc_grid")
                .num_columns(2)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    ui.label("ID Certificat :");
                    if ui.text_edit_singleline(&mut app.form.twod_cert_id).changed() { changed = true; }
                    ui.end_row();
                    ui.label("Données (C40) :");
                    if ui.text_edit_singleline(&mut app.form.twod_c40).changed() { changed = true; }
                    ui.end_row();
                });
            theme::status_warn(ui, "⚠ 2D-Doc sans signature cryptographique (informatif).");
        }
    }

    changed
}
