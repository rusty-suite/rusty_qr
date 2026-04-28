use egui::TextureHandle;

use crate::card::CardConfig;
use crate::export::ExportFormat;
use crate::history::{LibraryEntry, load_library};
use crate::lang::{Lang, RemoteLangInfo};
use crate::qr::types::QrForm;
use crate::style::{profile::{StyleProfile, load_profiles, save_profiles}, renderer};
use crate::template::{RemoteTemplate, TemplateColor, TemplateField};
use crate::ui;

// ─── App theme ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum AppTheme {
    #[default]
    System,
    Light,
    Dark,
}

impl AppTheme {
    fn cycle(self) -> Self {
        match self {
            AppTheme::System => AppTheme::Light,
            AppTheme::Light  => AppTheme::Dark,
            AppTheme::Dark   => AppTheme::System,
        }
    }
    fn icon(self) -> &'static str {
        match self {
            AppTheme::System => "\u{1F4BB}", // 💻
            AppTheme::Light  => "\u{2600}",  // ☀
            AppTheme::Dark   => "\u{1F319}", // 🌙
        }
    }
    pub fn tooltip(self, lang: &Lang) -> String {
        match self {
            AppTheme::System => lang.t("theme.system"),
            AppTheme::Light  => lang.t("theme.light"),
            AppTheme::Dark   => lang.t("theme.dark"),
        }
    }
}

fn theme_path() -> std::path::PathBuf {
    crate::workdir::work_dir().join("theme.json")
}

pub fn load_theme() -> AppTheme {
    std::fs::read_to_string(theme_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_theme(theme: AppTheme) {
    if let Ok(json) = serde_json::to_string(&theme) {
        let _ = std::fs::write(theme_path(), json);
    }
}

// ─── Tabs ────────────────────────────────────────────────────────────────────

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

    // ── Thème de l'interface ──────────────────────────────────────────────────
    pub app_theme: AppTheme,

    // ── Langue & répertoire de travail ────────────────────────────────────────
    pub lang: Lang,
    pub work_dir: std::path::PathBuf,
    /// Message d'erreur affiché si le chargement de la langue a échoué.
    pub lang_error: Option<String>,
    pub show_lang_settings: bool,
    pub remote_langs: Vec<RemoteLangInfo>,
    pub lang_remote_fetch_status: Option<(bool, String)>,
    pub lang_remote_fetch_rx: Option<std::sync::mpsc::Receiver<Result<Vec<RemoteLangInfo>, String>>>,
    pub lang_download_rx: Option<std::sync::mpsc::Receiver<Result<(String, std::path::PathBuf), String>>>,
}

impl RustyQrApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        lang: Lang,
        work_dir: std::path::PathBuf,
        lang_error: Option<String>,
    ) -> Self {
        setup_fonts(&cc.egui_ctx);
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
            app_theme: load_theme(),
            lang,
            work_dir,
            lang_error,
            show_lang_settings: false,
            remote_langs: Vec::new(),
            lang_remote_fetch_status: None,
            lang_remote_fetch_rx: None,
            lang_download_rx: None,
        }
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
            Err(e) => { self.qr_matrix = None; self.qr_error = Some(e.to_string()); }
        }
        self.mark_qr_dirty();
    }

    pub fn save_profiles(&self) {
        save_profiles(&self.profiles);
    }
}

impl eframe::App for RustyQrApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme — System = follow eframe/OS, otherwise override
        match self.app_theme {
            AppTheme::Dark   => ctx.set_visuals(egui::Visuals::dark()),
            AppTheme::Light  => ctx.set_visuals(egui::Visuals::light()),
            AppTheme::System => {} // laisser eframe suivre le thème OS
        }

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
                        self.logo_dl_status = Some((true, format!("\u{2713} Image charg\u{E9}e : {p}")));
                        self.profiles_dirty = true;
                        self.mark_qr_dirty();
                    }
                    Err(e) => {
                        self.logo_dl_status = Some((false, format!("\u{2717} {e}")));
                    }
                }
            }
        }

        if self.lang_remote_fetch_rx.is_some() || self.lang_download_rx.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(80));
        }

        if let Some(rx) = &self.lang_remote_fetch_rx {
            if let Ok(result) = rx.try_recv() {
                self.lang_remote_fetch_rx = None;
                self.lang_remote_fetch_status = Some(match result {
                    Ok(list) => {
                        let count = list.len();
                        self.remote_langs = list;
                        (true, format!("\u{1F7E2} GitHub — {count} langue(s) repo"))
                    }
                    Err(e) => {
                        self.remote_langs.clear();
                        let offline = e.to_lowercase().contains("dns")
                            || e.to_lowercase().contains("connect")
                            || e.to_lowercase().contains("timeout")
                            || e.to_lowercase().contains("network")
                            || e.to_lowercase().contains("refused");
                        let msg = if offline {
                            "\u{1F534} Hors-ligne — langues locales uniquement".into()
                        } else {
                            format!("\u{26A0} GitHub : {e}")
                        };
                        (false, msg)
                    }
                });
            }
        }

        if let Some(rx) = &self.lang_download_rx {
            if let Ok(result) = rx.try_recv() {
                self.lang_download_rx = None;
                match result {
                    Ok((stem, path)) => match crate::lang::Lang::load_file(&path) {
                        Ok(new_lang) => {
                            crate::lang::Lang::save_choice(&self.work_dir, &stem);
                            self.lang = new_lang;
                            self.lang_remote_fetch_status = Some((true, format!("\u{2B07} Langue install\u{E9}e : {stem}")));
                        }
                        Err(e) => {
                            self.lang_remote_fetch_status = Some((false, format!("\u{2717} Chargement langue : {e}")));
                        }
                    },
                    Err(e) => {
                        self.lang_remote_fetch_status = Some((false, format!("\u{2717} T\u{E9}l\u{E9}chargement langue : {e}")));
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
                            (true, "\u{1F7E2} En ligne \u{2014} aucun th\u{E8}me suppl\u{E9}mentaire".into())
                        } else {
                            (true, format!("\u{1F7E2} En ligne \u{2014} {n} th\u{E8}me(s) suppl\u{E9}mentaire(s)"))
                        }
                    }
                    Err(e) => {
                        let offline = e.to_lowercase().contains("dns")
                            || e.to_lowercase().contains("connect")
                            || e.to_lowercase().contains("timeout")
                            || e.to_lowercase().contains("network")
                            || e.to_lowercase().contains("refused");
                        let msg = if offline {
                            "\u{1F534} Hors-ligne \u{2014} th\u{E8}mes int\u{E9}gr\u{E9}s uniquement".into()
                        } else {
                            format!("\u{26A0} GitHub : {e}")
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
                            let labels   = self.card.layout.field_labels();
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
                        (true, "\u{2B07} Th\u{E8}me t\u{E9}l\u{E9}charg\u{E9} \u{2713}".into())
                    }
                    Err(e) => (false, format!("\u{2717} T\u{E9}l\u{E9}chargement : {e}")),
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

        // ── Erreur de langue (modal au premier démarrage sans fichier) ────────
        if self.lang_error.is_some() {
            let err_msg = self.lang_error.clone().unwrap_or_default();
            let mut dismiss = false;
            egui::Window::new("\u{26A0} Language / Langue")
                .collapsible(false)
                .resizable(false)
                .fixed_size([360.0, 110.0])
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.add_space(8.0);
                    ui.label(&err_msg);
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(4.0);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(4.0);
                        if ui.button("  OK  ").clicked() { dismiss = true; }
                    });
                    ui.add_space(4.0);
                });
            if dismiss { self.lang_error = None; }
        }

        // ── Top bar ──────────────────────────────────────────────────────────
        let theme_tooltip    = self.app_theme.tooltip(&self.lang);
        let lang_tooltip     = self.lang.t("nav.lang_settings");
        let active_lang_disp = {
            let stem = &self.lang.active_stem;
            if stem.is_empty() { "—".to_string() } else { stem.clone() }
        };
        egui::TopBottomPanel::top("topbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(4.0);
                ui.label(egui::RichText::new("RustyQR").strong());
                ui.label(egui::RichText::new("v1.0.0").small().weak());

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(4.0);
                    if ui.add(egui::Button::new("\u{2699}").frame(false))
                        .on_hover_text("\u{00C0} propos")
                        .clicked()
                    {
                        self.show_about = true;
                    }
                    ui.add_space(4.0);
                    if ui.add(egui::Button::new(self.app_theme.icon()).frame(false))
                        .on_hover_text(&theme_tooltip)
                        .clicked()
                    {
                        self.app_theme = self.app_theme.cycle();
                        save_theme(self.app_theme);
                    }
                    ui.add_space(4.0);
                    if ui.add(egui::Button::new("\u{1F310}").frame(false))
                        .on_hover_text(format!("{lang_tooltip}  [{active_lang_disp}]"))
                        .clicked()
                    {
                        self.show_lang_settings = true;
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
            let about_title   = self.lang.t("about.window_title");
            let lbl_app       = self.lang.t("about.app_label");
            let lbl_ver       = self.lang.t("about.version_label");
            let lbl_author    = self.lang.t("about.author_label");
            let lbl_license   = self.lang.t("about.license_label");
            let lbl_desc      = self.lang.t("about.description_label");
            let lbl_repo      = self.lang.t("about.repo_label");
            let desc_val      = self.lang.t("app.description");
            let repo_display  = self.lang.t("app.repo_display");
            let repo_url      = self.lang.t("app.repo_url");
            let app_author    = self.lang.t("app.author");
            let app_license   = self.lang.t("app.license");
            let app_version   = self.lang.t("app.version");
            let btn_close     = self.lang.t("about.close_button");
            egui::Window::new(about_title)
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
                        row(ui, &lbl_app,     "Rusty QR");
                        row(ui, &lbl_ver,     &app_version);
                        row(ui, &lbl_author,  &app_author);
                        row(ui, &lbl_license, &app_license);
                        row(ui, &lbl_desc,    &desc_val);
                        ui.label(egui::RichText::new(&lbl_repo).weak());
                        ui.add(egui::Hyperlink::from_label_and_url(&repo_display, &repo_url));
                        ui.end_row();
                    });
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(6.0);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(4.0);
                        if ui.button(&btn_close).clicked() { self.show_about = false; }
                    });
                    ui.add_space(4.0);
                });
        }

        // ── Modal "Langue" (sélection + liste) ───────────────────────────────
        if self.show_lang_settings {
            if self.lang_remote_fetch_rx.is_none()
                && self.lang_remote_fetch_status.is_none()
                && self.remote_langs.is_empty()
            {
                trigger_remote_lang_fetch(self);
            }

            // Prépare toutes les données AVANT la closure pour éviter les conflits d'emprunt
            let lang_infos    = crate::lang::Lang::list_available(&self.work_dir, &self.remote_langs);
            let lang_dir_path = self.work_dir.join("lang")
                .to_string_lossy().into_owned();
            let active_stem   = self.lang.active_stem.clone();
            let work_dir      = self.work_dir.clone();
            let remote_status = self.lang_remote_fetch_status.clone();
            let fetching_langs = self.lang_remote_fetch_rx.is_some();
            let downloading_lang = self.lang_download_rx.is_some();

            let lp_title     = self.lang.t("lang_page.title");
            let lp_active    = self.lang.t("lang_page.active_label");
            let lp_available = self.lang.t("lang_page.available_label");
            let lp_badge     = self.lang.t("lang_page.default_badge");
            let lp_folder    = self.lang.t("lang_page.folder_label");
            let lp_open      = self.lang.t("lang_page.open_folder_btn");
            let lp_close     = self.lang.t("lang_page.close_button");
            let lp_no_files  = self.lang.t("lang_page.no_files");
            let lp_reload    = {
                let s = self.lang.t("lang_page.reload_git_btn");
                if s == "lang_page.reload_git_btn" {
                    "Refresh Git".into()
                } else {
                    s
                }
            };
            let lp_local     = {
                let s = self.lang.t("lang_page.local_badge");
                if s == "lang_page.local_badge" { "[local]".into() } else { s }
            };
            let lp_repo      = {
                let s = self.lang.t("lang_page.repo_badge");
                if s == "lang_page.repo_badge" { "[git]".into() } else { s.replace("repo", "git").replace("Repo", "Git") }
            };

            // Nom d'affichage de la langue active
            let active_display = lang_infos.iter()
                .find(|i| i.stem == active_stem)
                .map(|i| i.display.clone())
                .unwrap_or_else(|| if active_stem.is_empty() { "—".into() } else { active_stem.clone() });

            let mut close      = false;
            let mut selected_local: Option<(String, std::path::PathBuf)> = None;
            let mut selected_remote: Option<RemoteLangInfo> = None;
            let mut reload_requested = false;

            egui::Window::new(lp_title)
                .collapsible(false)
                .resizable(false)
                .fixed_size([688.0, 548.0])
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    // ── Langue active ────────────────────────────────────────
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(&lp_active).weak());
                            ui.label(egui::RichText::new(&active_display).strong());
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                            if ui.small_button("✕").on_hover_text(&lp_close).clicked() {
                                close = true;
                            }
                        });
                    });
                    ui.add_space(6.0);
                    ui.add_space(6.0);

                    if let Some((ok, msg)) = &remote_status {
                        let color = if *ok {
                            egui::Color32::from_rgb(86, 201, 110)
                        } else {
                            egui::Color32::from_rgb(220, 90, 90)
                        };
                        ui.horizontal(|ui| {
                            ui.colored_label(color, "●");
                            ui.colored_label(color, msg);
                        });
                        ui.add_space(8.0);
                    }

                    ui.separator();
                    ui.add_space(8.0);

                    // ── Liste des fichiers disponibles ───────────────────────
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(&lp_available).weak().small());
                        if ui.small_button(&lp_reload).clicked() && !fetching_langs && !downloading_lang {
                            reload_requested = true;
                        }
                    });
                    ui.add_space(4.0);

                    let list_height = 234.0;
                    egui::ScrollArea::vertical()
                        .max_height(list_height)
                        .id_source("lang_list")
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            if lang_infos.is_empty() {
                                ui.label(egui::RichText::new(&lp_no_files).weak().italics());
                            } else {
                                for info in &lang_infos {
                                    let is_active = info.stem == active_stem;
                                    let mut parts = vec![info.display.clone()];
                                    if info.is_default {
                                        parts.push(lp_badge.clone());
                                    }
                                    if info.is_local {
                                        parts.push(lp_local.clone());
                                    }
                                    if info.is_remote {
                                        parts.push(lp_repo.clone());
                                    }
                                    parts.push(crate::lang::Lang::stem_compact_code(&info.stem));
                                    let label = parts.join("  ");
                                    let width = ui.available_width();
                                    let fill = if is_active {
                                        egui::Color32::from_rgb(0, 105, 148)
                                    } else {
                                        egui::Color32::from_rgb(66, 66, 66)
                                    };
                                    let stroke = if is_active {
                                        egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 120, 170))
                                    } else {
                                        egui::Stroke::new(1.0, egui::Color32::from_rgb(72, 72, 72))
                                    };
                                    let text = if is_active {
                                        egui::RichText::new(label).color(egui::Color32::WHITE)
                                    } else {
                                        egui::RichText::new(label).color(egui::Color32::from_gray(210))
                                    };
                                    let button = egui::Button::new(text)
                                        .min_size(egui::vec2(width, 28.0))
                                        .fill(fill)
                                        .stroke(stroke)
                                        .rounding(egui::Rounding::same(3.0));
                                    let clicked = ui.add(button).clicked();
                                    if clicked && !is_active {
                                        if let Some(path) = &info.path {
                                            selected_local = Some((info.stem.clone(), path.clone()));
                                        } else if let Some(url) = &info.remote_url {
                                            selected_remote = Some(RemoteLangInfo {
                                                stem: info.stem.clone(),
                                                download_url: url.clone(),
                                            });
                                        }
                                    }
                                    ui.add_space(4.0);
                                }
                            }
                        });

                    ui.add_space(12.0);
                    ui.allocate_space(egui::vec2(1.0, 48.0));
                    ui.separator();
                    ui.add_space(10.0);

                    // ── Dossier + bouton ouvrir ──────────────────────────────
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(&lp_folder).weak().small());
                        ui.label(egui::RichText::new(&lang_dir_path).small().monospace());
                        if ui.small_button(&lp_open).clicked() {
                            #[cfg(target_os = "windows")]
                            let _ = std::process::Command::new("explorer")
                                .arg(&work_dir.join("lang"))
                                .spawn();
                        }
                    });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(8.0);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(4.0);
                        if ui.button(&lp_close).clicked() { close = true; }
                    });
                    ui.add_space(4.0);
                });

            // Applique la sélection (après la closure, emprunts libérés)
            if reload_requested {
                self.remote_langs.clear();
                trigger_remote_lang_fetch(self);
            }
            if let Some((stem, path)) = selected_local {
                if let Ok(new_lang) = crate::lang::Lang::load_file(&path) {
                    crate::lang::Lang::save_choice(&self.work_dir, &stem);
                    self.lang = new_lang;
                }
            }
            if let Some(info) = selected_remote {
                if self.lang_download_rx.is_none() {
                    let work_dir = self.work_dir.clone();
                    let stem = info.stem.clone();
                    let (tx, rx) = std::sync::mpsc::channel();
                    std::thread::spawn(move || {
                        let result = crate::lang::Lang::download_to_lang_dir(&work_dir, &info)
                            .map(|path| (stem, path));
                        let _ = tx.send(result);
                    });
                    self.lang_download_rx = Some(rx);
                    self.lang_remote_fetch_status = Some((true, "T\u{E9}l\u{E9}chargement de la langue…".into()));
                }
            }
            if close { self.show_lang_settings = false; }
        }

        // ── Sidebar gauche ───────────────────────────────────────────────────
        let nav_create       = self.lang.t("nav.create");
        let nav_profiles     = self.lang.t("nav.profiles");
        let nav_library      = self.lang.t("nav.library");
        let nav_card         = self.lang.t("nav.card_designer");
        let nav_export       = self.lang.t("nav.export");
        let lbl_active_prof  = self.lang.t("nav.active_profile");
        let count            = self.library.len();
        let lbl_saved = if count == 1 {
            self.lang.t("sidebar.saved_single")
        } else {
            self.lang.t("sidebar.saved_plural").replace("{n}", &count.to_string())
        };
        egui::SidePanel::left("nav").resizable(false).exact_width(190.0).show(ctx, |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);

                nav_btn(ui, &mut self.tab, Tab::Creator,      &nav_create);
                nav_btn(ui, &mut self.tab, Tab::Profiles,     &nav_profiles);
                nav_btn(ui, &mut self.tab, Tab::Library,      &nav_library);
                nav_btn(ui, &mut self.tab, Tab::CardDesigner, &nav_card);
                nav_btn(ui, &mut self.tab, Tab::Export,       &nav_export);

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);

                // Indicateur bibliothèque
                if count > 0 {
                    ui.label(egui::RichText::new(&lbl_saved).small().weak());
                    ui.add_space(4.0);
                }

                // Profil actif
                ui.label(egui::RichText::new(&lbl_active_prof).small().weak());
                ui.add_space(2.0);
                let names: Vec<String> = self.profiles.iter().map(|p| p.name.clone()).collect();
                for (i, name) in names.iter().enumerate() {
                    if ui.selectable_label(self.selected_profile == i, name).clicked() {
                        self.selected_profile = i;
                        self.mark_qr_dirty();
                    }
                }
            });
        });

        // ── Panneau droit : aperçu QR ────────────────────────────────────────
        egui::SidePanel::right("preview").resizable(true).default_width(300.0).show(ctx, |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                ui::preview::show_preview(self, ui, ctx);
            });
        });

        // ── Zone centrale ────────────────────────────────────────────────────
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                match self.tab {
                    Tab::Creator      => ui::creator::show(self, ui),
                    Tab::Profiles     => ui::profiles::show(self, ui),
                    Tab::Library      => ui::library::show(self, ui),
                    Tab::CardDesigner => ui::card_designer::show(self, ui),
                    Tab::Export       => ui::export_ui::show(self, ui),
                }
            });
        });
    }
}

fn row(ui: &mut egui::Ui, label: impl AsRef<str>, value: impl AsRef<str>) {
    ui.label(egui::RichText::new(label.as_ref()).weak());
    ui.label(value.as_ref());
    ui.end_row();
}

fn nav_btn(ui: &mut egui::Ui, current: &mut Tab, target: Tab, label: impl AsRef<str>) {
    if ui.selectable_label(*current == target, label.as_ref()).clicked() {
        *current = target;
    }
}

/// Charge une police système avec une couverture étendue des symboles Unicode
/// (Dingbats, flèches, symboles divers) comme police de repli après la police
/// par défaut d'egui. Si aucune police n'est trouvée, le comportement est inchangé.
fn setup_fonts(ctx: &egui::Context) {
    // Candidats par ordre de priorité (Windows, macOS, Linux)
    let candidates: &[&str] = &[
        "C:/Windows/Fonts/seguisym.ttf",                          // Windows — Segoe UI Symbol
        "C:/Windows/Fonts/segoeui.ttf",                           // Windows — Segoe UI (fallback)
        "/System/Library/Fonts/Supplemental/Symbola.ttf",         // macOS
        "/System/Library/Fonts/Geneva.ttf",                       // macOS fallback
        "/usr/share/fonts/truetype/unifont/unifont.ttf",          // Linux — unifont (très complet)
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",        // Linux fallback
    ];

    let mut fonts = egui::FontDefinitions::default();
    let mut loaded = false;

    for path in candidates {
        if let Ok(data) = std::fs::read(path) {
            fonts.font_data.insert(
                "SymbolFallback".to_owned(),
                egui::FontData::from_owned(data),
            );
            // Ajout en fin de liste = utilisé uniquement si le glyphe est absent
            // des polices précédentes (Ubuntu-Light, Hack).
            fonts.families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push("SymbolFallback".to_owned());
            loaded = true;
            break;
        }
    }

    if loaded {
        ctx.set_fonts(fonts);
    }
}

fn trigger_remote_lang_fetch(app: &mut RustyQrApp) {
    if app.lang_remote_fetch_rx.is_some() {
        return;
    }
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(crate::lang::Lang::fetch_remote_index());
    });
    app.lang_remote_fetch_rx = Some(rx);
    app.lang_remote_fetch_status = Some((true, "V\u{E9}rification GitHub des langues…".into()));
}
