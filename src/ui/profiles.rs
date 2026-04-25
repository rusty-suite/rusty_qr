use egui::Ui;
use crate::app::RustyQrApp;
use crate::qr::types::EcLevel;
use crate::style::{profile::StyleProfile, renderer};
use crate::theme;

pub fn show(app: &mut RustyQrApp, ui: &mut Ui) {
    ui.add_space(8.0);
    theme::title(ui, "\u{1F3A8} Profils de style");
    theme::hint(ui, "Chaque profil d\u{E9}finit les couleurs, le logo et sa mise en page sur le QR code.");
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
                    app.mark_qr_dirty();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if count > 1 {
                        if ui.add(
                            egui::Button::new(
                                egui::RichText::new("\u{1F5D1}").color(egui::Color32::from_rgb(200, 80, 80))
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
                app.mark_qr_dirty();
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
                            if ui.button("\u{2713} Confirmer").clicked() {
                                app.profiles.remove(idx);
                                if app.selected_profile >= app.profiles.len() {
                                    app.selected_profile = app.profiles.len().saturating_sub(1);
                                }
                                app.save_profiles();
                                app.mark_qr_dirty();
                                app.confirm_delete_profile = None;
                            }
                            if ui.button("\u{2715} Annuler").clicked() {
                                app.confirm_delete_profile = None;
                            }
                        });
                    });
            }
        }

        ui.add_space(8.0);
        if ui.button("+ Nouveau profil").clicked() {
            app.profiles.push(StyleProfile::named(&format!("Profil {}", app.profiles.len() + 1)));
            app.selected_profile = app.profiles.len() - 1;
            app.confirm_delete_profile = None;
            app.save_profiles();
        }

        // ── Colonne droite : éditeur ─────────────────────────────────────────
        let ui = &mut cols[1];
        let ec = app.form.ec_level;
        if let Some(profile) = app.profiles.get_mut(app.selected_profile) {
            profile_editor(
                ui, profile, ec,
                &mut app.profiles_dirty,
                &mut app.logo_dl_rx, &mut app.logo_dl_status,
            );
        }
    });

    if app.profiles_dirty {
        app.save_profiles();
        app.profiles_dirty = false;
        app.mark_qr_dirty();
    }
}

fn profile_editor(
    ui: &mut Ui,
    profile: &mut StyleProfile,
    ec: EcLevel,
    save_dirty: &mut bool,
    logo_dl_rx: &mut Option<std::sync::mpsc::Receiver<Result<std::path::PathBuf, String>>>,
    logo_dl_status: &mut Option<(bool, String)>,
) {
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
                let r = ui.add(egui::Slider::new(&mut px, 4..=24));
                if r.changed() { profile.module_px = px as u32; }
                if r.drag_stopped() || (r.changed() && !r.dragged()) { *save_dirty = true; }
                ui.end_row();

                ui.label("Zone silencieuse :");
                let mut qz = profile.quiet_zone as i32;
                let r = ui.add(egui::Slider::new(&mut qz, 0..=10));
                if r.changed() { profile.quiet_zone = qz as u32; }
                if r.drag_stopped() || (r.changed() && !r.dragged()) { *save_dirty = true; }
                ui.end_row();
            });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(6.0);

        // ── Logo / image incrustée ────────────────────────────────────────────
        ui.label(egui::RichText::new("Logo / image incrust\u{E9}e").strong());
        ui.add_space(4.0);

        egui::Grid::new("prof_logo")
            .num_columns(2).spacing([8.0, 6.0])
            .show(ui, |ui| {
                // ── Fichier local ─────────────────────────────────────────────
                ui.label("Fichier local :");
                ui.horizontal(|ui| {
                    let path_resp = ui.add(
                        egui::TextEdit::singleline(&mut profile.logo_path)
                            .hint_text("chemin / drop ici…")
                            .desired_width(160.0),
                    );
                    if path_resp.changed() { *save_dirty = true; profile.logo_url.clear(); }

                    // Bouton dossier
                    if ui.button("\u{1F4C1}").on_hover_text("Choisir un fichier image").clicked() {
                        if let Some(p) = rfd::FileDialog::new()
                            .add_filter("Image", &["png","jpg","jpeg","webp","bmp","gif"])
                            .pick_file()
                        {
                            profile.logo_path = p.to_string_lossy().into_owned();
                            profile.logo_url.clear();
                            *save_dirty = true;
                        }
                    }
                    if !profile.logo_path.is_empty() {
                        if ui.add(egui::Button::new("\u{2715}").frame(false))
                            .on_hover_text("Supprimer le logo")
                            .clicked()
                        {
                            profile.logo_path.clear();
                            profile.logo_url.clear();
                            profile.logo_ratio = 0.0;
                            *logo_dl_status = None;
                            *save_dirty = true;
                        }
                    }
                });
                ui.end_row();

                // ── URL distante ──────────────────────────────────────────────
                ui.label("URL image :");
                ui.horizontal(|ui| {
                    let url_resp = ui.add(
                        egui::TextEdit::singleline(&mut profile.logo_url)
                            .hint_text("https://…/logo.png")
                            .desired_width(160.0),
                    );
                    if url_resp.changed() { *save_dirty = true; }

                    let downloading = logo_dl_rx.is_some();
                    let btn = ui.add_enabled(
                        !downloading && !profile.logo_url.is_empty(),
                        egui::Button::new(
                            if downloading { "\u{23F3}" } else { "\u{1F4E5}" }
                        ),
                    ).on_hover_text("T\u{E9}l\u{E9}charger et utiliser comme logo");

                    if btn.clicked() {
                        let url = profile.logo_url.clone();
                        let (tx, rx) = std::sync::mpsc::channel();
                        std::thread::spawn(move || {
                            let _ = tx.send(
                                crate::style::profile::StyleProfile::download_logo_to_cache(&url)
                            );
                        });
                        *logo_dl_rx = Some(rx);
                        *logo_dl_status = Some((true, "T\u{E9}l\u{E9}chargement…".into()));
                    }
                });
                ui.end_row();

                // ── Statut du téléchargement ──────────────────────────────────
                if let Some((ok, msg)) = logo_dl_status.as_ref() {
                    ui.label("");
                    if *ok {
                        ui.label(egui::RichText::new(msg).small().color(egui::Color32::from_rgb(80,200,80)));
                    } else {
                        ui.label(egui::RichText::new(msg).small().color(egui::Color32::from_rgb(220,80,80)));
                    }
                    ui.end_row();
                }

                // ── Taille du logo avec plafond dynamique basé sur EC ─────────
                let max_r = renderer::max_logo_ratio(ec);
                ui.label("Taille (ratio) :");
                ui.vertical(|ui| {
                    // Forcer le ratio dans la limite EC si déjà trop grand
                    if profile.logo_ratio > max_r {
                        profile.logo_ratio = max_r;
                        *save_dirty = true;
                    }
                    let mut r = profile.logo_ratio;
                    let resp = ui.add(
                        egui::Slider::new(&mut r, 0.0..=max_r)
                            .custom_formatter(|v, _| format!("{:.0}%", v * 100.0))
                    );
                    if resp.changed() { profile.logo_ratio = r; }
                    if resp.drag_stopped() || (resp.changed() && !resp.dragged()) { *save_dirty = true; }
                    ui.label(
                        egui::RichText::new(format!(
                            "Max EC {} : {:.0}%  \u{2014}  actuel : {:.0}%",
                            ec.label(), max_r * 100.0, profile.logo_ratio * 100.0
                        ))
                        .small()
                        .color(egui::Color32::from_rgb(140, 180, 220)),
                    );
                });
                ui.end_row();

                ui.label("Marge fond blanc :");
                let mut pad = profile.logo_padding as i32;
                let r = ui.add(
                    egui::Slider::new(&mut pad, 0..=20)
                        .custom_formatter(|v, _| {
                            if v == 0.0 { "0 \u{2014} aucun fond".into() }
                            else { format!("{v:.0} px") }
                        })
                );
                if r.changed() { profile.logo_padding = pad as u32; }
                if r.drag_stopped() || (r.changed() && !r.dragged()) { *save_dirty = true; }
                ui.end_row();

                if profile.logo_padding == 0 {
                    ui.label("");
                    ui.label(
                        egui::RichText::new("\u{2192} logo superpos\u{E9} sans fond blanc")
                            .small()
                            .color(egui::Color32::from_rgb(140, 180, 220)),
                    );
                    ui.end_row();
                }
            });

        ui.add_space(8.0);

        // ── Grille de position 3×3 ────────────────────────────────────────────
        ui.label(egui::RichText::new("Position du logo").small().weak());
        ui.add_space(4.0);

        let pos_grid = [
            (0.0f32, 0.0f32, "\u{2196}"), (0.5, 0.0, "\u{2191}"), (1.0, 0.0, "\u{2197}"),
            (0.0,    0.5,    "\u{2190}"), (0.5, 0.5, "\u{B7}"),   (1.0, 0.5, "\u{2192}"),
            (0.0,    1.0,    "\u{2199}"), (0.5, 1.0, "\u{2193}"), (1.0, 1.0, "\u{2198}"),
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
        theme::hint(ui, "Cliquez sur une cellule pour positionner le logo.\nLe centre est la position recommand\u{E9}e (\u{2022}).");

        // Safe zone info
        // ── Indicateur lisibilité ─────────────────────────────────────────────
        if profile.logo_ratio > 0.005 {
            ui.add_space(4.0);
            let ratio  = profile.logo_ratio;
            let max_ec = renderer::max_logo_ratio(ec);
            let (icon, text, color) = if ratio <= max_ec {
                ("\u{2713}", format!(
                    "QR lisible avec {} (logo \u{2264} {:.0}%)",
                    ec.label(), max_ec * 100.0
                ), egui::Color32::from_rgb(80, 200, 80))
            } else {
                ("\u{26A0}", format!(
                    "Logo trop grand pour {} \u{2014} r\u{E9}duisez ou passez en EC H",
                    ec.label()
                ), egui::Color32::from_rgb(220, 100, 60))
            };
            ui.label(
                egui::RichText::new(format!("{icon} {text}"))
                    .small()
                    .color(color),
            );
        }

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
