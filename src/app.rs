use egui::TextureHandle;

use crate::card::CardConfig;
use crate::export::ExportFormat;
use crate::history::{LibraryEntry, load_library};
use crate::lang::Lang;
use crate::qr::types::QrForm;
use crate::style::{profile::{StyleProfile, load_profiles, save_profiles}, renderer};
use crate::template::{RemoteTemplate, TemplateColor, TemplateField};
use crate::ui;

// ─── App theme ───────────────────────────────────────────────────────────────

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
    pub qr_error: Option<crate::qr::encoder::EncoderError>,
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

    // ── Thème SVG (concepteur de cartes) ─────────────────────────────────────
    /// 0 = aucun, 1..=BUILTIN.len() = intégré, BUILTIN.len()+1 = personnalisé, au-delà = distant
    // ── Logo URL download ─────────────────────────────────────────────────────
    pub logo_dl_rx: Option<std::sync::mpsc::Receiver<Result<std::path::PathBuf, String>>>,
    pub logo_dl_status: Option<(bool, String)>,

    // ── Cached rendered QR image (identical to right-panel preview) ───────────
    /// Kept in sync with preview_dirty; shared with the template preview so
    /// build_qr_image can embed it directly instead of re-rendering.
    pub qr_rendered_image: Option<image::RgbaImage>,

    // ── Card export (background thread) ──────────────────────────────────────
    pub card_export_rx: Option<std::sync::mpsc::Receiver<Result<String, String>>>,

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

    // ── Langue & répertoire de travail ────────────────────────────────────────
    pub lang: Lang,
    pub work_dir: std::path::PathBuf,

    // ── Charte Rusty Suite (rusty_kit) ───────────────────────────────────────
    pub settings: Settings,
    pub logo: Logo,
    pub toasts: ui_kit::Toasts,
    pub zones: ui_kit::Zones,
    pub show_language: bool,
    pub show_update: bool,
    pub online: Option<bool>,
    pub net_rx: Option<Receiver<bool>>,
    pub lang_rx: Option<Receiver<Result<usize, String>>>,
    pub update: Option<Release>,
    pub update_rx: Option<Receiver<Result<Option<Release>, String>>>,
    pub install_rx: Option<Receiver<Result<(), String>>>,
    pub args_rx: Receiver<Vec<String>>,
}

impl RustyQrApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        lang: Lang,
        work_dir: std::path::PathBuf,
        settings: Settings,
        args_rx: Receiver<Vec<String>>,
        rolled_back: Option<String>,
    ) -> Self {
        ui_kit::install(&cc.egui_ctx);
        let mut logo = Logo::new(LogoAnim::Loading);
        logo.play_launch(0.0);
        let mut app = Self {
            tab: Tab::Creator,
            form: QrForm::default(),
            qr_matrix: None,
            qr_error: None,
            preview_texture: None,
            preview_dirty: true,
            profiles: load_profiles(&lang),
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
            logo_dl_rx: None,
            logo_dl_status: None,
            qr_rendered_image: None,
            card_export_rx: None,
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
            lang,
            work_dir,
            settings,
            logo,
            toasts: ui_kit::Toasts::default(),
            zones: ui_kit::Zones::default(),
            show_language: false,
            show_update: false,
            online: None,
            net_rx: Some(rusty_core::net::check_in_background(user_agent())),
            lang_rx: None,
            update: None,
            update_rx: None,
            install_rx: None,
            args_rx,
        };
        app.apply_texts(&cc.egui_ctx);
        if let Some(v) = rolled_back {
            let text = app.lang.tf("kit.update_restored_text", &[("version", &v)]);
            app.toasts.push(Tone::Warning, app.lang.t("kit.update_restored"), text);
        }
        app
    }

    pub fn current_profile(&self) -> &StyleProfile {
        self.profiles
            .get(self.selected_profile)
            .or_else(|| self.profiles.first())
            .unwrap_or_else(|| Box::leak(Box::new(StyleProfile::default())))
    }

    /// Marque les deux previews comme obsolètes.
    /// À appeler dès que le QR ou le profil actif change.
    pub fn mark_qr_dirty(&mut self) {
        self.preview_dirty = true;
        self.qr_rendered_image = None; // invalidate cached render
        if self.selected_template_idx > 0 {
            self.template_preview_dirty = true;
        }
    }

    pub fn regenerate_qr(&mut self) {
        match crate::qr::encoder::encode(&self.form) {
            Ok(m)  => { self.qr_matrix = Some(m); self.qr_error = None; }
            Err(e) => { self.qr_matrix = None; self.qr_error = Some(e); }
        }
        self.mark_qr_dirty();
    }

    pub fn save_profiles(&self) {
        save_profiles(&self.profiles);
    }
}

impl eframe::App for RustyQrApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.zones.begin_frame(ctx);
        self.poll_kit(ctx);

        // ── Poll logo URL download ────────────────────────────────────────────
        if let Some(rx) = &self.logo_dl_rx {
            if let Ok(result) = rx.try_recv() {
                self.logo_dl_rx = None;
                match result {
                    Ok(path) => {
                        let p = path.to_string_lossy().into_owned();
                        if let Some(profile) = self.profiles.get_mut(self.selected_profile) {
                            profile.logo_path = p.clone();
                        }
                        self.logo_dl_status = Some((true, format!("\u{2713} {}", self.lang.tf("status.image_loaded", &[("path", &p)]))));
                        self.profiles_dirty = true;
                        self.mark_qr_dirty();
                    }
                    Err(e) => {
                        self.logo_dl_status = Some((false, format!("\u{2717} {e}")));
                    }
                }
            }
        }

        // ── Poll background template fetches ─────────────────────────────────
        let fetching_index = self.remote_fetch_rx.is_some();
        let fetching_svg   = self.remote_svg_dl.is_some();
        if fetching_index || fetching_svg {
            // Keep repainting so we notice when the thread completes
            ctx.request_repaint_after(std::time::Duration::from_millis(80));
        }

        if fetching_index {
            let done = self.remote_fetch_rx.as_ref()
                .and_then(|rx| rx.try_recv().ok());
            if let Some(result) = done {
                self.remote_fetch_status = Some(match result {
                    Ok(list) => {
                        // Keep only themes not already compiled into BUILTIN
                        let extras: Vec<_> = list.into_iter()
                            .filter(|t| !crate::template::is_builtin_id(&t.id))
                            .collect();
                        let n = extras.len();
                        self.remote_templates = extras;
                        if n == 0 {
                            (true, format!("\u{1F7E2} {}", self.lang.t("status.themes_online_none")))
                        } else {
                            (true, format!("\u{1F7E2} {}", self.lang.tf("status.themes_online_n", &[("n", &n.to_string())])))
                        }
                    }
                    Err(e) => {
                        let offline = e.to_lowercase().contains("dns")
                            || e.to_lowercase().contains("connect")
                            || e.to_lowercase().contains("timeout")
                            || e.to_lowercase().contains("network")
                            || e.to_lowercase().contains("refused");
                        let msg = if offline {
                            format!("\u{1F534} {}", self.lang.t("status.themes_offline"))
                        } else {
                            format!("\u{26A0} {}", self.lang.tf("status.github_error", &[("error", &e)]))
                        };
                        (false, msg)
                    }
                });
                self.remote_fetch_rx = None;
            }
        }

        if fetching_svg {
            let done = self.remote_svg_dl.as_ref()
                .and_then(|(idx, rx)| rx.try_recv().ok().map(|r| (*idx, r)));
            if let Some((idx, result)) = done {
                self.remote_fetch_status = Some(match result {
                    Ok(svg) => {
                        if let Some(t) = self.remote_templates.get_mut(idx) {
                            t.svg = Some(svg.clone());
                        }
                        // Re-apply palette + field detection now that the SVG is ready
                        let rb = crate::template::BUILTIN.len() + 2;
                        let selected_remote_idx = self.selected_template_idx.saturating_sub(rb);
                        if self.selected_template_idx >= rb && selected_remote_idx == idx {
                            let (bg, fg, ac) = crate::template::detect_palette_defaults(&svg);
                            if let Some(c) = bg { self.card.bg_color     = c; }
                            if let Some(c) = fg { self.card.text_color   = c; }
                            if let Some(c) = ac { self.card.accent_color = c; }
                            let labels   = self.card.layout.field_label_keys();
                            let detected = crate::template::detect_fields(&svg, labels);
                            let old_f    = std::mem::replace(&mut self.template_field_data, detected);
                            for tf in &mut self.template_field_data {
                                if let Some(prev) = old_f.iter().find(|o| o.var == tf.var) {
                                    tf.value   = prev.value.clone();
                                    tf.visible = prev.visible;
                                }
                            }
                            let detected_c = crate::template::detect_colors(&svg);
                            let old_c      = std::mem::replace(&mut self.template_color_data, detected_c);
                            for tc in &mut self.template_color_data {
                                if let Some(prev) = old_c.iter().find(|o| o.var == tc.var) {
                                    tc.value = prev.value;
                                }
                            }
                        }
                        self.template_preview_dirty = true;
                        (true, format!("\u{2B07} {} \u{2713}", self.lang.t("status.theme_downloaded")))
                    }
                    Err(e) => (false, format!("\u{2717} {}", self.lang.tf("status.download_error", &[("error", &e)]))),
                });
                self.remote_svg_dl = None;
            }
        }

        // ── Poll card export background thread ───────────────────────────────
        if let Some(rx) = &self.card_export_rx {
            if let Ok(result) = rx.try_recv() {
                self.card_export_rx = None;
                self.card_export_status = Some(match result {
                    Ok(msg)  => (true,  msg),
                    Err(msg) => (false, msg),
                });
            }
        }

        // ── Template SVG preview (re-render when dirty) ──────────────────────
        if self.template_preview_dirty {
            self.template_preview_dirty = false;
            if self.selected_template_idx == 0 {
                self.template_preview_texture = None;
            } else {
                // If the QR image cache was invalidated, re-render it now so the
                // template preview uses the freshest render without waiting for
                // the right-panel preview.rs to run later this frame.
                if self.qr_rendered_image.is_none() {
                    if let Some(matrix) = &self.qr_matrix {
                        let profile = self.current_profile().clone();
                        let ec = self.form.ec_level;
                        self.qr_rendered_image = Some(renderer::render_ec(matrix, &profile, ec));
                    }
                }

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
                    let profile    = self.current_profile().clone();
                    let matrix_ref = self.qr_matrix.as_ref();
                    let qr_img_ref = self.qr_rendered_image.as_ref();
                    let preview_svg = crate::template::render_preview(
                        &svg_str, &self.card,
                        &self.template_field_data, &self.template_color_data,
                        matrix_ref, &profile,
                        self.form.ec_level,
                        qr_img_ref,
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

        // ── Châssis de la charte : barre d'état, menu latéral, barre d'outils ──
        let languages = self.lang.tf("kit.languages_count", &[("count", &rusty_core::lang::STEMS.len().to_string())]);
        let update_version = self.update.as_ref().map(|r| r.version.clone());
        let mut action = ui_kit::status_bar(ctx, &ui_kit::StatusInfo { online: self.online, languages: &languages, update: update_version.as_deref(), version: VERSION });
        let info = ui_kit::AppInfo { name: "Rusty QR".into(), description: self.lang.t("app.description"), version: VERSION.into() };
        let labels: Vec<String> = SECTIONS.iter().map(|(_, key, _)| self.lang.t(key)).collect();
        let items: Vec<ui_kit::SidebarItem> = SECTIONS.iter().zip(&labels).map(|((_, _, icon), label)| ui_kit::SidebarItem { icon, label }).collect();
        let mut selected = SECTIONS.iter().position(|(t, _, _)| *t == self.tab).unwrap_or(0);
        let collapsed_before = self.settings.common.sidebar_collapsed;
        let a = ui_kit::sidebar(ctx, &mut self.logo, &info, &items, &mut selected, &mut self.settings.common.sidebar_collapsed);
        if a != ui_kit::ChromeAction::None {
            action = a;
        }
        self.tab = SECTIONS[selected.min(SECTIONS.len() - 1)].0;
        if self.settings.common.sidebar_collapsed != collapsed_before {
            self.save_settings();
        }
        // Profil de style actif : liste dans la barre d'outils.
        let title = labels[selected.min(labels.len() - 1)].clone();
        let names: Vec<String> = self.profiles.iter().map(|p| p.name.clone()).collect();
        let mut profile_idx = self.selected_profile;
        let profile_label = self.lang.t("nav.active_profile");
        let a = ui_kit::toolbar(ctx, &title, |ui| {
            ui_kit::select(ui, "active_profile", &mut profile_idx, &names);
            ui.label(ui_kit::Text::Label.rich(&profile_label).color(ui_kit::tokens::TEXT_SECONDARY));
        });
        if a != ui_kit::ChromeAction::None {
            action = a;
        }
        if profile_idx != self.selected_profile {
            self.selected_profile = profile_idx;
            self.mark_qr_dirty();
        }
        match action {
            ui_kit::ChromeAction::Settings | ui_kit::ChromeAction::About => self.show_about = true,
            ui_kit::ChromeAction::Language => self.show_language = true,
            ui_kit::ChromeAction::Update => self.show_update = true,
            ui_kit::ChromeAction::None => {}
        }

        // ── Panneau droit : aperçu QR ────────────────────────────────────────
        egui::SidePanel::right("preview").resizable(true).default_width(300.0).show(ctx, |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                ui::preview::show_preview(self, ui, ctx);
            });
        });

        // ── Zone centrale ────────────────────────────────────────────────────
        ui_kit::content(ctx, false, |ui| match self.tab {
            Tab::Creator => ui::creator::show(self, ui),
            Tab::Profiles => ui::profiles::show(self, ui),
            Tab::Library => ui::library::show(self, ui),
            Tab::CardDesigner => ui::card_designer::show(self, ui),
            Tab::Export => ui::export_ui::show(self, ui),
        });

        self.kit_windows(ctx);
        self.toasts.show(ctx);
        self.zones.end_frame(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        update::mark_healthy(&self.work_dir);
    }
}

// ── Charte Rusty Suite : réglages, langues, réseau, mise à jour ─────────────

use rusty_core::settings::CommonSettings;
use rusty_core::update::{self, Release};
use rusty_ui as ui_kit;
use rusty_ui::rusty_logo::{Logo, LogoAnim};
use rusty_ui::Tone;
use std::sync::mpsc::{self, Receiver};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const LICENSE: &str = env!("CARGO_PKG_LICENSE");
const REPOSITORY: &str = "https://github.com/rusty-suite/rusty_qr";

/// Agent HTTP générique (jamais d'information personnelle).
pub fn user_agent() -> String {
    format!("{}/{VERSION}", crate::workdir::APP_ID)
}

/// Réglages persistants (langue, menu réduit, mises à jour).
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub common: CommonSettings,
}

/// Sections du menu latéral : onglet, clé du libellé, icône de la charte.
const SECTIONS: [(Tab, &str, &str); 5] = [
    (Tab::Creator, "nav.create", ui_kit::icons::LINK),
    (Tab::Profiles, "nav.profiles", ui_kit::icons::FOLDER),
    (Tab::Library, "nav.library", ui_kit::icons::LOG),
    (Tab::CardDesigner, "nav.card_designer", ui_kit::icons::FILE),
    (Tab::Export, "nav.export", ui_kit::icons::DOWNLOAD),
];

impl RustyQrApp {
    fn apply_texts(&self, ctx: &egui::Context) {
        let lang = self.lang.clone();
        ui_kit::set_texts(ctx, ui_kit::KitTexts::from_lookup(|k| lang.t(k)));
    }

    fn save_settings(&self) {
        if let Err(e) = rusty_core::settings::save(&self.work_dir, &self.settings) {
            log::warn!("Réglages non enregistrés : {e}");
        }
    }

    fn reload_language(&mut self, ctx: &egui::Context) {
        self.lang = crate::lang::load(&self.work_dir, &self.settings.common.language);
        self.apply_texts(ctx);
        self.profiles_dirty = true;
        self.preview_dirty = true;
    }

    /// Réseau, langues distantes, mise à jour, second lancement.
    fn poll_kit(&mut self, ctx: &egui::Context) {
        if let Some(online) = self.net_rx.as_ref().and_then(|rx| rx.try_recv().ok()) {
            self.net_rx = None;
            self.online = Some(online);
            if online {
                let (dir, ua) = (self.work_dir.clone(), user_agent());
                let (tx, rx) = mpsc::channel();
                std::thread::spawn(move || {
                    let _ = tx.send(rusty_core::lang::refresh_remote(crate::workdir::APP_ID, crate::lang::PACKS, &dir, &ua));
                });
                self.lang_rx = Some(rx);
                if self.settings.common.check_updates {
                    let (tx, rx) = mpsc::channel();
                    std::thread::spawn(move || {
                        let id = crate::workdir::APP_ID;
                        let _ = tx.send(update::check(id, id, VERSION, &user_agent()).map_err(|e| e.to_string()));
                    });
                    self.update_rx = Some(rx);
                }
            }
        }
        if let Some(Ok(n)) = self.lang_rx.as_ref().and_then(|rx| rx.try_recv().ok()) {
            self.lang_rx = None;
            if n > 0 {
                self.reload_language(ctx);
            }
        }
        if let Some(result) = self.update_rx.as_ref().and_then(|rx| rx.try_recv().ok()) {
            self.update_rx = None;
            match result {
                Ok(release) => self.update = release,
                Err(e) => log::warn!("Vérification des mises à jour impossible : {e}"),
            }
        }
        if let Some(result) = self.install_rx.as_ref().and_then(|rx| rx.try_recv().ok()) {
            self.install_rx = None;
            match result {
                Ok(()) => {
                    if let Ok(exe) = std::env::current_exe() {
                        let _ = std::process::Command::new(exe).spawn();
                    }
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                Err(e) => {
                    self.show_update = false;
                    self.toasts.push(Tone::Error, self.lang.t("kit.update_failed"), e);
                }
            }
        }
        // Second lancement : ramène la fenêtre existante au premier plan.
        if self.args_rx.try_iter().count() > 0 {
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }
        if self.net_rx.is_some() || self.lang_rx.is_some() || self.update_rx.is_some() || self.install_rx.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(250));
        }
    }

    /// Fenêtres communes : langue, à propos, mise à jour.
    fn kit_windows(&mut self, ctx: &egui::Context) {
        let lang = self.lang.clone();
        let tr = |k: &str| lang.t(k);
        let langs: Vec<(String, String)> = rusty_core::lang::STEMS.iter().map(|s| (s.to_string(), rusty_core::lang::language_name(crate::lang::PACKS, s))).collect();
        let folder = self.work_dir.join("lang").display().to_string();
        if let Some(stem) = ui_kit::language_window(ctx, &mut self.show_language, &tr, &lang.stem, &langs, &folder) {
            self.settings.common.language = stem;
            self.save_settings();
            self.reload_language(ctx);
        }
        let info = ui_kit::AppInfo { name: "Rusty QR".into(), description: lang.t("app.description"), version: VERSION.into() };
        ui_kit::about_window(ctx, &mut self.show_about, &tr, &info, LICENSE, REPOSITORY, &lang.t("app.author"));
        if let Some(v) = self.update.as_ref().map(|r| r.version.clone()) {
            let installing = self.install_rx.is_some();
            if ui_kit::update_window(ctx, &mut self.show_update, &tr, &v, installing) == ui_kit::UpdateChoice::Install && !installing {
                self.start_install();
            }
        }
    }

    fn start_install(&mut self) {
        let Some(release) = self.update.clone() else { return };
        let (dir, ua) = (self.work_dir.clone(), user_agent());
        let checksum_text = self.lang.t("kit.update_checksum");
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let r = update::install(&release, VERSION, &dir, &ua).map_err(|e| match e {
                update::UpdateError::Checksum => checksum_text,
                other => other.to_string(),
            });
            let _ = tx.send(r);
        });
        self.install_rx = Some(rx);
    }
}
