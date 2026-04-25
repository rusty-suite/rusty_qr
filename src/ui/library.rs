//! Bibliothèque — liste des configurations QR sauvegardées.

use egui::Ui;
use crate::app::RustyQrApp;
use crate::history;
use crate::theme;

pub fn show(app: &mut RustyQrApp, ui: &mut Ui) {
    ui.add_space(8.0);
    theme::title(ui, "📚 Bibliothèque");
    theme::hint(ui, "Vos configurations QR enregistrées — cliquez pour recharger et modifier.");
    ui.separator();
    ui.add_space(8.0);

    if app.library.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.label(egui::RichText::new("Aucune entrée enregistrée.").weak());
            ui.add_space(8.0);
            theme::hint(ui, "Depuis l'onglet « Créer QR », utilisez\n« 💾 Enregistrer dans la bibliothèque ».");
        });
        return;
    }

    let mut to_delete: Option<u64> = None;
    let mut to_load:   Option<usize> = None;

    egui::ScrollArea::vertical().show(ui, |ui| {
        for (i, entry) in app.library.iter().enumerate() {
            let is_loaded = app.loaded_library_id == Some(entry.id);

            egui::Frame::none()
                .fill(if is_loaded {
                    egui::Color32::from_rgb(30, 60, 30)
                } else {
                    egui::Color32::from_gray(28)
                })
                .rounding(4.0)
                .inner_margin(egui::Margin::symmetric(10.0, 6.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Left: info
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(&entry.name).strong());
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} · {}",
                                    entry.form.content_type.label(),
                                    entry.date
                                ))
                                .small()
                                .weak(),
                            );
                            // Preview of encoded string
                            let preview = {
                                let s = entry.form.to_qr_string();
                                if s.chars().count() > 60 {
                                    format!("{}…", s.chars().take(60).collect::<String>())
                                } else {
                                    s
                                }
                            };
                            ui.label(
                                egui::RichText::new(preview)
                                    .small()
                                    .weak()
                                    .font(egui::FontId::monospace(10.0)),
                            );
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_space(4.0);
                            // Delete
                            if ui.add(
                                egui::Button::new(
                                    egui::RichText::new("🗑").color(egui::Color32::from_rgb(200, 80, 80))
                                ).frame(false)
                            ).on_hover_text("Supprimer").clicked() {
                                to_delete = Some(entry.id);
                            }
                            // Load
                            let load_label = if is_loaded { "✓ Chargé" } else { "⬆ Charger" };
                            let load_color = if is_loaded {
                                egui::Color32::from_rgb(80, 200, 80)
                            } else {
                                egui::Color32::from_rgb(100, 160, 240)
                            };
                            if ui.add(
                                egui::Button::new(egui::RichText::new(load_label).color(load_color))
                                    .frame(false)
                            ).on_hover_text("Charger dans l'éditeur").clicked() {
                                to_load = Some(i);
                            }
                        });
                    });
                });

            ui.add_space(4.0);
        }
    });

    // Apply actions after iteration (borrow rules)
    if let Some(id) = to_delete {
        history::remove_entry(&mut app.library, id);
        if app.loaded_library_id == Some(id) {
            app.loaded_library_id = None;
        }
    }
    if let Some(i) = to_load {
        if let Some(entry) = app.library.get(i) {
            let id = entry.id;
            app.form = entry.form.clone();
            app.loaded_library_id = Some(id);
            app.tab = crate::app::Tab::Creator;
            app.regenerate_qr();
        }
    }
}
