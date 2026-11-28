//! Bibliothèque — liste des configurations QR sauvegardées.

use egui::Ui;
use crate::app::RustyQrApp;
use crate::history;
use crate::theme;

pub fn show(app: &mut RustyQrApp, ui: &mut Ui) {
    ui.add_space(8.0);
    theme::title(ui, &format!("\u{1F4DA} {}", app.lang.t("library.title")));
    theme::hint(ui, &app.lang.t("library.hint"));
    ui.separator();
    ui.add_space(8.0);

    if app.library.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.label(egui::RichText::new(app.lang.t("library.empty")).weak());
            ui.add_space(8.0);
            theme::hint(ui, &app.lang.t("library.empty_hint"));
        });
        return;
    }

    let mut to_delete: Option<u64> = None;
    let mut to_load:   Option<usize> = None;

    for (i, entry) in app.library.iter().enumerate() {
            let is_loaded = app.loaded_library_id == Some(entry.id);

            egui::Frame::new()
                .fill(if is_loaded {
                    rusty_ui::tokens::SUCCESS_BG
                } else {
                    rusty_ui::tokens::BG_PANEL
                })
                .corner_radius(rusty_ui::theme::radius(rusty_ui::tokens::RADIUS_MD))
                .inner_margin(egui::Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Left: info
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(&entry.name).strong());
                            let mut meta = format!("{} \u{B7} {}", app.lang.t(entry.form.content_type.label_key()), entry.date);
                            if let Some(ref bid) = entry.template.builtin_id {
                                meta.push_str(&format!("  \u{2022}  \u{1F5FA} {bid}"));
                            } else if entry.template.custom_svg.is_some() {
                                meta.push_str(&format!("  \u{2022}  \u{1F5FA} {}", app.lang.t("library.custom_theme")));
                            }
                            ui.label(egui::RichText::new(meta).small().weak());
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
                                    egui::RichText::new("\u{1F5D1}").color(rusty_ui::tokens::DANGER)
                                ).frame(false)
                            ).on_hover_text(app.lang.t("library.delete")).clicked() {
                                to_delete = Some(entry.id);
                            }
                            // Load
                            let load_label = if is_loaded { app.lang.t("library.loaded") } else { app.lang.t("library.load") };
                            let load_color = if is_loaded {
                                rusty_ui::tokens::SUCCESS
                            } else {
                                rusty_ui::tokens::ACCENT_TEAL_TEXT
                            };
                            if ui.add(
                                egui::Button::new(egui::RichText::new(load_label).color(load_color))
                            ).on_hover_text(app.lang.t("library.load_hint")).clicked() {
                                to_load = Some(i);
                            }
                        });
                    });
                });

        ui.add_space(4.0);
    }

    // Apply actions after iteration (borrow rules)
    if let Some(id) = to_delete {
        history::remove_entry(&mut app.library, id);
        if app.loaded_library_id == Some(id) {
            app.loaded_library_id = None;
        }
    }
    if let Some(i) = to_load {
        if let Some(entry) = app.library.get(i) {
            let id       = entry.id;
            let tpl      = entry.template.clone();
            app.form     = entry.form.clone();
            app.loaded_library_id = Some(id);
            app.regenerate_qr();

            // Restore card config
            if let Some(card) = tpl.card {
                app.card = card;
            }

            // Restore template selection
            if let Some(ref bid) = tpl.builtin_id {
                let idx = crate::template::BUILTIN.iter()
                    .position(|t| t.id == bid.as_str())
                    .map(|i| i + 1)
                    .unwrap_or(0);
                app.selected_template_idx = idx;
            } else if tpl.custom_svg.is_some() {
                app.custom_template_svg = tpl.custom_svg.clone();
                app.selected_template_idx = crate::template::BUILTIN.len() + 1;
            } else {
                app.selected_template_idx = 0;
            }

            // Restore field values
            if app.selected_template_idx > 0 {
                let svg_opt: Option<String> = if app.selected_template_idx <= crate::template::BUILTIN.len() {
                    Some(crate::template::BUILTIN[app.selected_template_idx - 1].svg.to_string())
                } else {
                    app.custom_template_svg.clone()
                };
                if let Some(svg) = svg_opt {
                    let labels = app.card.layout.field_label_keys();
                    let mut detected = crate::template::detect_fields(&svg, labels);
                    for df in &mut detected {
                        if let Some(sf) = tpl.fields.iter().find(|f| f.var == df.var) {
                            df.value   = sf.value.clone();
                            df.visible = sf.visible;
                        }
                    }
                    app.template_field_data = detected;

                    let mut det_c = crate::template::detect_colors(&svg);
                    for dc in &mut det_c {
                        if let Some(sc) = tpl.colors.iter().find(|c| c.var == dc.var) {
                            dc.value = sc.value;
                        }
                    }
                    app.template_color_data = det_c;
                }
            } else {
                app.template_field_data.clear();
                app.template_color_data.clear();
            }
            app.template_preview_dirty = true;
            app.tab = crate::app::Tab::Creator;
        }
    }
}
