#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod card;
mod export;
mod history;
mod lang;
mod logo;
mod qr;
mod style;
mod template;
mod theme;
mod ui;
mod workdir;

fn main() -> eframe::Result<()> {
    // ── Répertoire de travail ────────────────────────────────────────────────
    let work_dir = workdir::work_dir();

    // ── Chargement de la langue (bloquant, avant la fenêtre) ─────────────────
    let (lang, lang_error) = lang::Lang::load(&work_dir);

    // ── Titre de la fenêtre depuis la langue ─────────────────────────────────
    let window_title = lang.t("app.title");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([900.0, 600.0])
            .with_title(&window_title)
            .with_icon(logo::icon_data()),
        follow_system_theme: true,
        ..Default::default()
    };

    eframe::run_native(
        "RustyQR",
        options,
        Box::new(move |cc| Ok(Box::new(app::RustyQrApp::new(cc, lang, work_dir, lang_error)))),
    )
}
