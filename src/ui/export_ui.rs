use egui::Ui;

use crate::app::RustyQrApp;
use crate::export::{self, ExportFormat};
use crate::theme;

pub fn show(app: &mut RustyQrApp, ui: &mut Ui) {
    ui.add_space(8.0);
    theme::title(ui, "Exporter le code QR");
    ui.separator();
    ui.add_space(8.0);

    if app.qr_matrix.is_none() {
        theme::status_warn(ui, "⚠ Générez d'abord un code QR dans l'onglet Créer.");
        return;
    }

    egui::Grid::new("export_grid")
        .num_columns(2)
        .spacing([8.0, 8.0])
        .show(ui, |ui| {
            // Format
            ui.label("Format :");
            egui::ComboBox::from_id_source("export_fmt")
                .selected_text(app.export_format.label())
                .show_ui(ui, |ui| {
                    for &fmt in ExportFormat::ALL {
                        ui.selectable_value(&mut app.export_format, fmt, fmt.label());
                    }
                });
            ui.end_row();

            // Profil
            ui.label("Profil de style :");
            let names: Vec<String> = app.profiles.iter().map(|p| p.name.clone()).collect();
            egui::ComboBox::from_id_source("export_profile")
                .selected_text(names.get(app.selected_profile).cloned().unwrap_or_default())
                .show_ui(ui, |ui| {
                    for (i, name) in names.iter().enumerate() {
                        if ui.selectable_label(app.selected_profile == i, name).clicked() {
                            app.selected_profile = i;
                            app.preview_dirty = true;
                        }
                    }
                });
            ui.end_row();

            // Path
            ui.label("Chemin :");
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut app.export_path);
                if ui.button("📁 Parcourir").clicked() {
                    let ext = app.export_format.extension();
                    let name = app.export_format.filter_name();
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter(name, &[ext])
                        .set_file_name(&format!("qrcode.{ext}"))
                        .save_file()
                    {
                        app.export_path = path.to_string_lossy().into_owned();
                    }
                }
            });
            ui.end_row();
        });

    ui.add_space(8.0);

    if ui.button("💾 Exporter").clicked() {
        if app.export_path.is_empty() {
            app.export_status = Some((false, "Choisissez un chemin de fichier.".into()));
        } else if let Some(matrix) = &app.qr_matrix {
            let profile = app.current_profile().clone();
            let result = export::export(matrix, &profile, app.export_format, &app.export_path);
            app.export_status = Some(match result {
                Ok(_) => (true, format!("✓ Exporté : {}", app.export_path)),
                Err(e) => (false, format!("✗ Erreur : {e}")),
            });
        }
    }

    if let Some((ok, msg)) = &app.export_status {
        ui.add_space(4.0);
        if *ok { theme::status_ok(ui, msg); } else { theme::status_err(ui, msg); }
    }
}
