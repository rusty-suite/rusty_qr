use egui::Ui;

use crate::app::RustyQrApp;
use crate::style::profile::StyleProfile;
use crate::theme;

pub fn show(app: &mut RustyQrApp, ui: &mut Ui) {
    ui.add_space(8.0);
    theme::title(ui, "Profils de style");
    ui.separator();
    ui.add_space(8.0);

    ui.columns(2, |cols| {
        // ── Left: profile list ───────────────────────────────────────────────
        cols[0].label(egui::RichText::new("Profils").strong());
        cols[0].add_space(4.0);

        let mut to_delete: Option<usize> = None;
        let count = app.profiles.len();

        for i in 0..count {
            let selected = app.selected_profile == i;
            cols[0].horizontal(|ui| {
                if ui.selectable_label(selected, &app.profiles[i].name).clicked() {
                    app.selected_profile = i;
                    app.preview_dirty = true;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if count > 1 {
                        if ui.add(egui::Button::new("✕").frame(false)).clicked() {
                            to_delete = Some(i);
                        }
                    }
                });
            });
        }

        if let Some(idx) = to_delete {
            app.profiles.remove(idx);
            if app.selected_profile >= app.profiles.len() {
                app.selected_profile = app.profiles.len().saturating_sub(1);
            }
            app.save_profiles();
            app.preview_dirty = true;
        }

        cols[0].add_space(8.0);
        if cols[0].button("+ Nouveau profil").clicked() {
            app.profiles.push(StyleProfile::named(&format!("Profil {}", app.profiles.len() + 1)));
            app.selected_profile = app.profiles.len() - 1;
            app.save_profiles();
        }

        // ── Right: profile editor ────────────────────────────────────────────
        if let Some(profile) = app.profiles.get_mut(app.selected_profile) {
            cols[1].label(egui::RichText::new("Éditer le profil").strong());
            cols[1].add_space(4.0);

            let mut dirty = false;

            egui::Grid::new("profile_editor")
                .num_columns(2)
                .spacing([8.0, 6.0])
                .show(&mut cols[1], |ui| {
                    ui.label("Nom :");
                    if ui.text_edit_singleline(&mut profile.name).changed() { dirty = true; }
                    ui.end_row();

                    ui.label("Couleur modules :");
                    let mut fg = [
                        profile.fg[0] as f32 / 255.0,
                        profile.fg[1] as f32 / 255.0,
                        profile.fg[2] as f32 / 255.0,
                    ];
                    if ui.color_edit_button_rgb(&mut fg).changed() {
                        profile.fg = [
                            (fg[0] * 255.0) as u8,
                            (fg[1] * 255.0) as u8,
                            (fg[2] * 255.0) as u8,
                        ];
                        dirty = true;
                    }
                    ui.end_row();

                    ui.label("Couleur fond :");
                    let mut bg = [
                        profile.bg[0] as f32 / 255.0,
                        profile.bg[1] as f32 / 255.0,
                        profile.bg[2] as f32 / 255.0,
                    ];
                    if ui.color_edit_button_rgb(&mut bg).changed() {
                        profile.bg = [
                            (bg[0] * 255.0) as u8,
                            (bg[1] * 255.0) as u8,
                            (bg[2] * 255.0) as u8,
                        ];
                        dirty = true;
                    }
                    ui.end_row();

                    ui.label("Taille module (px) :");
                    let mut px = profile.module_px as i32;
                    if ui.add(egui::Slider::new(&mut px, 4..=20)).changed() {
                        profile.module_px = px as u32;
                        dirty = true;
                    }
                    ui.end_row();

                    ui.label("Zone silencieuse :");
                    let mut qz = profile.quiet_zone as i32;
                    if ui.add(egui::Slider::new(&mut qz, 0..=10)).changed() {
                        profile.quiet_zone = qz as u32;
                        dirty = true;
                    }
                    ui.end_row();

                    ui.label("Logo (chemin) :");
                    ui.horizontal(|ui| {
                        if ui.text_edit_singleline(&mut profile.logo_path).changed() { dirty = true; }
                        if ui.button("📁").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Image", &["png", "jpg", "jpeg", "webp", "bmp"])
                                .pick_file()
                            {
                                profile.logo_path = path.to_string_lossy().into_owned();
                                dirty = true;
                            }
                        }
                        if !profile.logo_path.is_empty() {
                            if ui.add(egui::Button::new("✕").frame(false)).clicked() {
                                profile.logo_path.clear();
                                dirty = true;
                            }
                        }
                    });
                    ui.end_row();

                    ui.label("Taille logo :");
                    let mut r = profile.logo_ratio;
                    if ui.add(egui::Slider::new(&mut r, 0.0..=0.30).suffix(" (ratio)")).changed() {
                        profile.logo_ratio = r;
                        dirty = true;
                    }
                    ui.end_row();
                });

            if dirty {
                app.save_profiles();
                app.preview_dirty = true;
            }
        }
    });
}
