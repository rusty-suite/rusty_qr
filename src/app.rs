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

        // Sidebar left — navigation + profile list
        egui::SidePanel::left("nav")
            .resizable(false)
            .exact_width(180.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.label(egui::RichText::new("RustyQR").size(18.0).strong());
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
