use egui::Ui;
use crate::app::RustyQrApp;
use crate::style::profile::StyleProfile;
use crate::theme;

pub fn show(app: &mut RustyQrApp, ui: &mut Ui) {
    ui.add_space(8.0);
    theme::title(ui, "🎨 Profils de style");
    theme::hint(ui, "Chaque profil définit les couleurs, le logo et sa mise en page sur le QR code.");
    ui.separator();
    ui.add_space(8.0);

    ui.columns(2, |cols| {
        // ── Colonne gauche : liste des profils ───────────────────────────────
        let ui = &mut cols[0];
        ui.label(egui::RichText::new("Profils").strong());
        ui.add_space(4.0);

        let mut to_delete: Option<usize> = None;
        let count = app.profiles.len();

        for i in 0..count {
            let selected = app.selected_profile == i;
            ui.horizontal(|ui| {
                // Color swatch
                let p = &app.profiles[i];
                let swatch = egui::Color32::from_rgb(p.fg[0], p.fg[1], p.fg[2]);
                let (sr, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                ui.painter().rect_filled(sr, 2.0, swatch);

                if ui.selectable_label(selected, &app.profiles[i].name).clicked() {
                    app.selected_profile = i;
                    app.preview_dirty = true;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if count > 1 {
                        if ui.add(
                            egui::Button::new(
                                egui::RichText::new("🗑").color(egui::Color32::from_rgb(200, 80, 80))
                            ).frame(false)
                        ).on_hover_text("Supprimer ce profil").clicked() {
                            to_delete = Some(i);
                        }
                    }
                });
            });
        }

        if let Some(idx) = to_delete {
            // Confirmation inline (affichée à la prochaine frame si l'index est marqué)
            if app.confirm_delete_profile == Some(idx) {
                app.profiles.remove(idx);
                if app.selected_profile >= app.profiles.len() {
                    app.selected_profile = app.profiles.len().saturating_sub(1);
                }
                app.save_profiles();
                app.preview_dirty = true;
                app.confirm_delete_profile = None;
            } else {
                app.confirm_delete_profile = Some(idx);
            }
        }

        // Confirmation dialog
        if let Some(idx) = app.confirm_delete_profile {
            if let Some(p) = app.profiles.get(idx) {
                let name = p.name.clone();
                ui.add_space(4.0);
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(60, 20, 20))
                    .rounding(4.0)
                    .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(format!("Supprimer « {name} » ?")).color(egui::Color32::from_rgb(240, 120, 120)));
                        ui.horizontal(|ui| {
                            if ui.button("✓ Confirmer").clicked() {
                                app.profiles.remove(idx);
                                if app.selected_profile >= app.profiles.len() {
                                    app.selected_profile = app.profiles.len().saturating_sub(1);
                                }
                                app.save_profiles();
                                app.preview_dirty = true;
                                app.confirm_delete_profile = None;
                            }
                            if ui.button("✕ Annuler").clicked() {
                                app.confirm_delete_profile = None;
                            }
                        });
                    });
            }
        }

        ui.add_space(8.0);
        if ui.button("＋ Nouveau profil").clicked() {
            app.profiles.push(StyleProfile::named(&format!("Profil {}", app.profiles.len() + 1)));
            app.selected_profile = app.profiles.len() - 1;
            app.confirm_delete_profile = None;
            app.save_profiles();
        }

        // ── Colonne droite : éditeur ─────────────────────────────────────────
        let ui = &mut cols[1];
        if let Some(profile) = app.profiles.get_mut(app.selected_profile) {
            profile_editor(ui, profile, &mut app.preview_dirty, &mut app.profiles_dirty);
        }
    });

    if app.profiles_dirty {
        app.save_profiles();
        app.profiles_dirty = false;
        app.preview_dirty = true;
    }
}

fn profile_editor(ui: &mut Ui, profile: &mut StyleProfile, preview_dirty: &mut bool, save_dirty: &mut bool) {
    ui.label(egui::RichText::new("Éditer le profil").strong());
    ui.add_space(4.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        // ── Nom ──────────────────────────────────────────────────────────────
        egui::Grid::new("prof_base")
            .num_columns(2).spacing([8.0, 6.0])
            .show(ui, |ui| {
                ui.label("Nom :");
                if ui.text_edit_singleline(&mut profile.name).changed() { *save_dirty = true; }
                ui.end_row();

                ui.label("Couleur modules :");
                if color_btn(ui, &mut profile.fg) { *save_dirty = true; }
                ui.end_row();

                ui.label("Couleur fond :");
                if color_btn(ui, &mut profile.bg) { *save_dirty = true; }
                ui.end_row();

                ui.label("Taille module (px) :");
                let mut px = profile.module_px as i32;
                if ui.add(egui::Slider::new(&mut px, 4..=24)).changed() {
                    profile.module_px = px as u32; *save_dirty = true;
                }
                ui.end_row();

                ui.label("Zone silencieuse :");
                let mut qz = profile.quiet_zone as i32;
                if ui.add(egui::Slider::new(&mut qz, 0..=10)).changed() {
                    profile.quiet_zone = qz as u32; *save_dirty = true;
                }
                ui.end_row();
            });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(6.0);

        // ── Logo / image incrustée ────────────────────────────────────────────
        ui.label(egui::RichText::new("Logo / image incrustée").strong());
        ui.add_space(4.0);

        egui::Grid::new("prof_logo")
            .num_columns(2).spacing([8.0, 6.0])
            .show(ui, |ui| {
                ui.label("Fichier :");
                ui.horizontal(|ui| {
                    if ui.text_edit_singleline(&mut profile.logo_path).changed() { *save_dirty = true; }
                    if ui.button("📁").clicked() {
                        if let Some(p) = rfd::FileDialog::new()
                            .add_filter("Image", &["png","jpg","jpeg","webp","bmp"])
                            .pick_file()
                        {
                            profile.logo_path = p.to_string_lossy().into_owned();
                            *save_dirty = true;
                        }
                    }
                    if !profile.logo_path.is_empty() {
                        if ui.add(egui::Button::new("✕").frame(false)).clicked() {
                            profile.logo_path.clear(); profile.logo_ratio = 0.0;
                            *save_dirty = true;
                        }
                    }
                });
                ui.end_row();

                ui.label("Taille (ratio) :");
                let mut r = profile.logo_ratio;
                if ui.add(
                    egui::Slider::new(&mut r, 0.0..=0.30)
                        .custom_formatter(|v, _| format!("{:.0}%", v * 100.0))
                ).changed() {
                    profile.logo_ratio = r; *save_dirty = true;
                }
                ui.end_row();

                ui.label("Marge blanche (px) :");
                let mut pad = profile.logo_padding as i32;
                if ui.add(egui::Slider::new(&mut pad, 0..=20)).changed() {
                    profile.logo_padding = pad as u32; *save_dirty = true;
                }
                ui.end_row();
            });

        ui.add_space(8.0);

        // ── Grille de position 3×3 ────────────────────────────────────────────
        ui.label(egui::RichText::new("Position du logo").small().weak());
        ui.add_space(4.0);

        let pos_grid = [
            (0.0f32, 0.0f32, "↖"), (0.5, 0.0, "↑"), (1.0, 0.0, "↗"),
            (0.0,    0.5,    "←"), (0.5, 0.5, "·"), (1.0, 0.5, "→"),
            (0.0,    1.0,    "↙"), (0.5, 1.0, "↓"), (1.0, 1.0, "↘"),
        ];

        let cell = 28.0;
        let grid_size = egui::vec2(cell * 3.0, cell * 3.0);
        let (grid_rect, _) = ui.allocate_exact_size(grid_size, egui::Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(grid_rect, 3.0, egui::Color32::from_gray(35));

        for (i, &(px, py, label)) in pos_grid.iter().enumerate() {
            let col = i % 3;
            let row = i / 3;
            let cx  = grid_rect.left() + col as f32 * cell + cell / 2.0;
            let cy  = grid_rect.top()  + row as f32 * cell + cell / 2.0;
            let btn_rect = egui::Rect::from_center_size(egui::pos2(cx, cy), egui::vec2(cell - 2.0, cell - 2.0));

            let selected = (profile.logo_pos_x - px).abs() < 0.01
                        && (profile.logo_pos_y - py).abs() < 0.01;

            let fill = if selected {
                egui::Color32::from_rgb(50, 170, 50)
            } else {
                egui::Color32::from_gray(55)
            };

            painter.rect_filled(btn_rect, 3.0, fill);
            painter.text(
                egui::pos2(cx, cy),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(13.0),
                egui::Color32::WHITE,
            );

            if ui.interact(btn_rect, egui::Id::new(("logo_pos", i)), egui::Sense::click()).clicked() {
                profile.logo_pos_x = px;
                profile.logo_pos_y = py;
                *save_dirty = true;
            }
        }

        ui.add_space(4.0);
        theme::hint(ui, "Cliquez sur une cellule pour positionner le logo.\nLe centre est la position recommandée (•).");

        // Safe zone info
        ui.add_space(4.0);
        let pct = (profile.logo_ratio * 100.0) as u32;
        let msg = match pct {
            0 => None,
            1..=7  => Some(("✓ Zone sûre (EC L minimum)", egui::Color32::from_rgb(80, 200, 80))),
            8..=15 => Some(("✓ Zone sûre (EC M minimum)", egui::Color32::from_rgb(80, 200, 80))),
            16..=25=> Some(("⚠ Utilisez EC Q ou H", egui::Color32::from_rgb(220, 160, 60))),
            26..=30=> Some(("⚠ Utilisez EC H uniquement", egui::Color32::from_rgb(220, 120, 60))),
            _      => Some(("✗ Logo trop grand — lisibilité compromise", egui::Color32::from_rgb(220, 80, 80))),
        };
        if let Some((text, color)) = msg {
            ui.label(egui::RichText::new(text).small().color(color));
        }

        *preview_dirty = true;
    });
}

fn color_btn(ui: &mut Ui, c: &mut [u8; 3]) -> bool {
    let mut f = [c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0];
    let changed = ui.color_edit_button_rgb(&mut f).changed();
    if changed {
        *c = [(f[0] * 255.0) as u8, (f[1] * 255.0) as u8, (f[2] * 255.0) as u8];
    }
    changed
}
