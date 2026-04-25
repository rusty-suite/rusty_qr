use egui::TextureHandle;

use crate::export::ExportFormat;
use crate::qr::types::QrForm;
use crate::style::profile::{StyleProfile, load_profiles, save_profiles};
use crate::ui;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Creator,
    Profiles,
    Export,
}

pub struct RustyQrApp {
    pub tab: Tab,

    // QR creator
    pub form: QrForm,
    pub qr_matrix: Option<Vec<Vec<bool>>>,
    pub qr_error: Option<String>,
    pub preview_texture: Option<TextureHandle>,
    pub preview_dirty: bool,

    // Profiles
    pub profiles: Vec<StyleProfile>,
    pub selected_profile: usize,

    // Export
    pub export_format: ExportFormat,
    pub export_path: String,
    pub export_status: Option<(bool, String)>, // (ok, message)

    // À propos
    pub show_about: bool,
    pub logo_texture: Option<egui::TextureHandle>,
}

impl RustyQrApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let profiles = load_profiles();
        Self {
            tab: Tab::Creator,
            form: QrForm::default(),
            qr_matrix: None,
            qr_error: None,
            preview_texture: None,
            preview_dirty: true,
            profiles,
            selected_profile: 0,
            export_format: ExportFormat::Png,
            export_path: String::new(),
            export_status: None,
            show_about: false,
            logo_texture: None,
        }
    }

    pub fn current_profile(&self) -> &StyleProfile {
        self.profiles
            .get(self.selected_profile)
            .or_else(|| self.profiles.first())
            .unwrap_or_else(|| Box::leak(Box::new(StyleProfile::default())))
    }

    pub fn regenerate_qr(&mut self) {
        match crate::qr::encoder::encode(&self.form) {
            Ok(matrix) => {
                self.qr_matrix = Some(matrix);
                self.qr_error = None;
            }
            Err(e) => {
                self.qr_matrix = None;
                self.qr_error = Some(e.to_string());
            }
        }
        self.preview_dirty = true;
    }

    pub fn save_profiles(&self) {
        save_profiles(&self.profiles);
    }
}

impl eframe::App for RustyQrApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());

        // ── Top bar ──────────────────────────────────────────────────────────
        egui::TopBottomPanel::top("topbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(4.0);
                ui.label(egui::RichText::new("RustyQR").strong());
                ui.label(egui::RichText::new("v1.0.0").small().weak());

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(4.0);
                    if ui.add(egui::Button::new("⚙").frame(false)).on_hover_text("À propos").clicked() {
                        self.show_about = true;
                    }
                });
            });
        });

        // ── Chargement lazy du logo (nécessite ctx, donc fait ici) ──────────
        if self.logo_texture.is_none() {
            let rgba = crate::logo::generate_rgba(96);
            let img = egui::ColorImage::from_rgba_unmultiplied([96, 96], &rgba);
            self.logo_texture = Some(ctx.load_texture("logo", img, egui::TextureOptions::LINEAR));
        }

        // ── Modal "À propos" ─────────────────────────────────────────────────
        if self.show_about {
            egui::Window::new("À propos de RustyQR")
                .collapsible(false)
                .resizable(false)
                .auto_sized()
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.add_space(8.0);

                    // Logo centré
                    if let Some(tex) = &self.logo_texture {
                        ui.vertical_centered(|ui| {
                            ui.add(
                                egui::Image::new(tex)
                                    .fit_to_exact_size(egui::vec2(96.0, 96.0)),
                            );
                            ui.add_space(6.0);
                            ui.label(
                                egui::RichText::new("RustyQR")
                                    .size(20.0)
                                    .strong(),
                            );
                        });
                    }

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(8.0);

                    egui::Grid::new("about_grid")
                        .num_columns(2)
                        .spacing([16.0, 6.0])
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("Application :").weak());
                            ui.label("Rusty QR");
                            ui.end_row();

                            ui.label(egui::RichText::new("Version :").weak());
                            ui.label(egui::RichText::new("v1.0.0").strong());
                            ui.end_row();

                            ui.label(egui::RichText::new("Auteur :").weak());
                            ui.label("rusty-suite");
                            ui.end_row();

                            ui.label(egui::RichText::new("Licence :").weak());
                            ui.label("PolyForm-Noncommercial");
                            ui.end_row();

                            ui.label(egui::RichText::new("Description :").weak());
                            ui.label("Générateur de codes QR multi-formats\navec profils de style et export vectoriel.");
                            ui.end_row();

                            ui.label(egui::RichText::new("Dépôt :").weak());
                            ui.add(egui::Hyperlink::from_label_and_url(
                                "github.com/rusty-suite/rusty_qr",
                                "https://github.com/rusty-suite/rusty_qr",
                            ));
                            ui.end_row();
                        });

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(6.0);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(4.0);
                        if ui.button("  Fermer  ").clicked() {
                            self.show_about = false;
                        }
                    });
                    ui.add_space(4.0);
                });
        }

        // Sidebar left — navigation + profile list
        egui::SidePanel::left("nav")
            .resizable(false)
            .exact_width(180.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);

                if ui.selectable_label(self.tab == Tab::Creator,  "📄 Créer QR").clicked() {
                    self.tab = Tab::Creator;
                }
                if ui.selectable_label(self.tab == Tab::Profiles, "🎨 Profils de style").clicked() {
                    self.tab = Tab::Profiles;
                }
                if ui.selectable_label(self.tab == Tab::Export,   "💾 Exporter").clicked() {
                    self.tab = Tab::Export;
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);
                ui.label(egui::RichText::new("Profil actif").small().weak());
                ui.add_space(2.0);

                let names: Vec<String> = self.profiles.iter().map(|p| p.name.clone()).collect();
                for (i, name) in names.iter().enumerate() {
                    let selected = self.selected_profile == i;
                    if ui.selectable_label(selected, name).clicked() {
                        self.selected_profile = i;
                        self.preview_dirty = true;
                    }
                }
            });

        // Right panel — QR preview (always visible)
        egui::SidePanel::right("preview")
            .resizable(true)
            .default_width(300.0)
            .show(ctx, |ui| {
                ui::preview::show_preview(self, ui, ctx);
            });

        // Central panel — content based on active tab
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.tab {
                Tab::Creator  => ui::creator::show(self, ui),
                Tab::Profiles => ui::profiles::show(self, ui),
                Tab::Export   => ui::export_ui::show(self, ui),
            }
        });
    }
}
