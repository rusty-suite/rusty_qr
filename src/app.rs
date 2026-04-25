use egui::TextureHandle;

use crate::card::CardConfig;
use crate::export::ExportFormat;
use crate::history::{LibraryEntry, load_library};
use crate::qr::types::QrForm;
use crate::style::profile::{StyleProfile, load_profiles, save_profiles};
use crate::template::{RemoteTemplate, TemplateColor, TemplateField};
use crate::ui;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Creator,
    Profiles,
    Library,
    CardDesigner,
    Export,
}

pub struct RustyQrApp {
    pub tab: Tab,

    // ── QR creator ────────────────────────────────────────────────────────────
    pub form: QrForm,
    pub qr_matrix: Option<Vec<Vec<bool>>>,
    pub qr_error: Option<String>,
    pub preview_texture: Option<TextureHandle>,
    pub preview_dirty: bool,

    // ── Profiles ──────────────────────────────────────────────────────────────
    pub profiles: Vec<StyleProfile>,
    pub selected_profile: usize,
    pub confirm_delete_profile: Option<usize>,
    pub profiles_dirty: bool,

    // ── Bibliothèque ──────────────────────────────────────────────────────────
    pub library: Vec<LibraryEntry>,
    pub loaded_library_id: Option<u64>,
    pub show_save_dialog: bool,
    pub save_name_input: String,

    // ── Concepteur de cartes ──────────────────────────────────────────────────
    pub card: CardConfig,
    pub card_export_status: Option<(bool, String)>,

    // ── Export ────────────────────────────────────────────────────────────────
    pub export_format: ExportFormat,
    pub export_path: String,
    pub export_status: Option<(bool, String)>,

    // ── À propos ─────────────────────────────────────────────────────────────
    pub show_about: bool,
    pub logo_texture: Option<TextureHandle>,

    // ── Thème SVG (concepteur de cartes) ─────────────────────────────────────
    /// 0 = aucun, 1..=BUILTIN.len() = intégré, BUILTIN.len()+1 = personnalisé, au-delà = distant
    pub selected_template_idx: usize,
    pub custom_template_svg: Option<String>,
    pub template_field_data: Vec<TemplateField>,
    pub template_color_data: Vec<TemplateColor>,
    pub template_preview_texture: Option<TextureHandle>,
    pub template_preview_dirty: bool,
    pub remote_templates: Vec<RemoteTemplate>,
    pub remote_fetch_status: Option<(bool, String)>,
    pub remote_fetch_rx: Option<std::sync::mpsc::Receiver<Result<Vec<RemoteTemplate>, String>>>,
    pub remote_svg_dl: Option<(usize, std::sync::mpsc::Receiver<Result<String, String>>)>,
}

impl RustyQrApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            tab: Tab::Creator,
            form: QrForm::default(),
            qr_matrix: None,
            qr_error: None,
            preview_texture: None,
            preview_dirty: true,
            profiles: load_profiles(),
            selected_profile: 0,
            confirm_delete_profile: None,
            profiles_dirty: false,
            library: load_library(),
            loaded_library_id: None,
            show_save_dialog: false,
            save_name_input: String::new(),
            card: CardConfig::default(),
            card_export_status: None,
            export_format: ExportFormat::Png,
            export_path: String::new(),
            export_status: None,
            show_about: false,
            logo_texture: None,
            selected_template_idx: 0,
            custom_template_svg: None,
            template_field_data: Vec::new(),
            template_color_data: Vec::new(),
            template_preview_texture: None,
            template_preview_dirty: false,
            remote_templates: Vec::new(),
            remote_fetch_status: None,
            remote_fetch_rx: None,
            remote_svg_dl: None,
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
            Ok(m)  => { self.qr_matrix = Some(m); self.qr_error = None; }
            Err(e) => { self.qr_matrix = None; self.qr_error = Some(e.to_string()); }
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

        // ── Poll background template fetches ─────────────────────────────────
        if self.remote_fetch_rx.is_some() {
            let done = self.remote_fetch_rx.as_ref()
                .and_then(|rx| rx.try_recv().ok());
            if let Some(result) = done {
                self.remote_fetch_status = Some(match result {
                    Ok(list) => {
                        let n = list.len();
                        self.remote_templates = list;
                        (true, format!("✓ {n} thème(s) chargé(s) depuis GitHub"))
                    }
                    Err(e) => (false, format!("✗ {e}")),
                });
                self.remote_fetch_rx = None;
            }
        }
        if self.remote_svg_dl.is_some() {
            let done = self.remote_svg_dl.as_ref()
                .and_then(|(idx, rx)| rx.try_recv().ok().map(|r| (*idx, r)));
            if let Some((idx, result)) = done {
                self.remote_fetch_status = Some(match result {
                    Ok(svg) => {
                        if let Some(t) = self.remote_templates.get_mut(idx) {
                            t.svg = Some(svg);
                        }
                        (true, "✓ Thème téléchargé".into())
                    }
                    Err(e) => (false, format!("✗ {e}")),
                });
                self.remote_svg_dl = None;
            }
        }

        // ── Template SVG preview (re-render when dirty) ──────────────────────
        if self.template_preview_dirty {
            self.template_preview_dirty = false;
            if self.selected_template_idx == 0 {
                self.template_preview_texture = None;
            } else {
                // Retrieve the template SVG string without keeping a borrow on self
                let tpl_svg: Option<String> = {
                    let n = crate::template::BUILTIN.len();
                    let ci = n + 1;
                    let rb = n + 2;
                    if self.selected_template_idx <= n {
                        Some(crate::template::BUILTIN[self.selected_template_idx - 1].svg.to_string())
                    } else if self.selected_template_idx == ci {
                        self.custom_template_svg.clone()
                    } else {
                        let ri = self.selected_template_idx - rb;
                        self.remote_templates.get(ri).and_then(|t| t.svg.clone())
                    }
                };
                if let Some(svg_str) = tpl_svg {
                    let preview_svg = crate::template::render_preview(
                        &svg_str, &self.card,
                        &self.template_field_data, &self.template_color_data,
                    );
                    if let Some((rgba, w, h)) = crate::template::svg_to_rgba(&preview_svg, 400, 320) {
                        let img = egui::ColorImage::from_rgba_unmultiplied(
                            [w as usize, h as usize], &rgba,
                        );
                        self.template_preview_texture = Some(
                            ctx.load_texture("tmpl_preview", img, egui::TextureOptions::LINEAR),
                        );
                    }
                }
            }
        }

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

        // ── Logo texture (lazy) ───────────────────────────────────────────────
        if self.logo_texture.is_none() {
            let rgba = crate::logo::generate_rgba(96);
            let img  = egui::ColorImage::from_rgba_unmultiplied([96, 96], &rgba);
            self.logo_texture = Some(ctx.load_texture("logo", img, egui::TextureOptions::LINEAR));
        }

        // ── Modal "À propos" ─────────────────────────────────────────────────
        if self.show_about {
            egui::Window::new("À propos de RustyQR")
                .collapsible(false)
                .resizable(false)
                .fixed_size([360.0, 340.0])
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.add_space(8.0);
                    if let Some(tex) = &self.logo_texture {
                        ui.vertical_centered(|ui| {
                            ui.add(egui::Image::new(tex).fit_to_exact_size(egui::vec2(96.0, 96.0)));
                            ui.add_space(6.0);
                            ui.label(egui::RichText::new("RustyQR").size(20.0).strong());
                        });
                    }
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(8.0);
                    egui::Grid::new("about_grid").num_columns(2).spacing([16.0, 6.0]).show(ui, |ui| {
                        row(ui, "Application :", "Rusty QR");
                        row(ui, "Version :",     "v1.0.0");
                        row(ui, "Auteur :",      "rusty-suite");
                        row(ui, "Licence :",     "PolyForm-Noncommercial");
                        row(ui, "Description :", "Générateur de codes QR multi-formats\navec profils de style et export vectoriel.");
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
                        if ui.button("  Fermer  ").clicked() { self.show_about = false; }
                    });
                    ui.add_space(4.0);
                });
        }

        // ── Sidebar gauche ───────────────────────────────────────────────────
        egui::SidePanel::left("nav").resizable(false).exact_width(190.0).show(ctx, |ui| {
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            nav_btn(ui, &mut self.tab, Tab::Creator,      "📄 Créer QR");
            nav_btn(ui, &mut self.tab, Tab::Profiles,     "🎨 Profils de style");
            nav_btn(ui, &mut self.tab, Tab::Library,      "📚 Bibliothèque");
            nav_btn(ui, &mut self.tab, Tab::CardDesigner, "\u{1F5FA} Concepteur de cartes");
            nav_btn(ui, &mut self.tab, Tab::Export,       "💾 Exporter");

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            // Indicateur bibliothèque
            let count = self.library.len();
            if count > 0 {
                ui.label(egui::RichText::new(format!("{count} entrée(s) sauvegardée(s)")).small().weak());
                ui.add_space(4.0);
            }

            // Profil actif
            ui.label(egui::RichText::new("Profil actif").small().weak());
            ui.add_space(2.0);
            let names: Vec<String> = self.profiles.iter().map(|p| p.name.clone()).collect();
            for (i, name) in names.iter().enumerate() {
                if ui.selectable_label(self.selected_profile == i, name).clicked() {
                    self.selected_profile = i;
                    self.preview_dirty = true;
                }
            }
        });

        // ── Panneau droit : aperçu QR ────────────────────────────────────────
        egui::SidePanel::right("preview").resizable(true).default_width(300.0).show(ctx, |ui| {
            ui::preview::show_preview(self, ui, ctx);
        });

        // ── Zone centrale ────────────────────────────────────────────────────
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.tab {
                Tab::Creator      => ui::creator::show(self, ui),
                Tab::Profiles     => ui::profiles::show(self, ui),
                Tab::Library      => ui::library::show(self, ui),
                Tab::CardDesigner => ui::card_designer::show(self, ui),
                Tab::Export       => ui::export_ui::show(self, ui),
            }
        });
    }
}

fn row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.label(egui::RichText::new(label).weak());
    ui.label(value);
    ui.end_row();
}

fn nav_btn(ui: &mut egui::Ui, current: &mut Tab, target: Tab, label: &str) {
    if ui.selectable_label(*current == target, label).clicked() {
        *current = target;
    }
}
