#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

// =============================================================================
// Rusty QR — point d'entrée
//
// Avant l'interface : dossier de travail (portable, ou ancien dossier s'il contient
// une bibliothèque ou des profils), journal, réglages (l'ancien choix de langue est
// repris), contrôle de la dernière mise à jour (retour arrière automatique) et
// instance unique (un second lancement ramène la fenêtre ouverte).
// =============================================================================

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

use rusty_core::{instance, journal, settings, update};

fn main() -> eframe::Result<()> {
    let work_dir = workdir::work_dir();
    journal::init(workdir::APP_ID, app::VERSION, &work_dir);
    rusty_core::lang::install_local(lang::PACKS, &work_dir, app::VERSION);

    let mut settings: app::Settings = settings::load(&work_dir);
    if settings.common.language.is_empty() {
        if let Some(stem) = lang::legacy_choice(&work_dir) {
            settings.common.language = stem;
            let _ = settings::save(&work_dir, &settings);
        }
    }
    let rolled_back = match update::startup(&work_dir, app::VERSION) {
        update::Startup::Relaunched => return Ok(()),
        update::Startup::RolledBack(v) => Some(v),
        update::Startup::Normal => None,
    };
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args_rx = match instance::start(workdir::APP_ID, &args) {
        instance::Instance::Secondary => return Ok(()),
        instance::Instance::Primary(rx) => rx,
    };
    update::mark_healthy_later(work_dir.clone());

    let lang = lang::load(&work_dir, &settings.common.language);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1240.0, 600.0])
            .with_min_inner_size([900.0, 480.0])
            .with_title(lang.t("app.title"))
            .with_icon(logo::icon_data()),
        // Centrée : la fenêtre reste entière sur un écran 1080p à 150 % (charte, 02).
        centered: true,
        ..Default::default()
    };
    eframe::run_native(
        "RustyQR",
        options,
        Box::new(move |cc| Ok(Box::new(app::RustyQrApp::new(cc, lang, work_dir, settings, args_rx, rolled_back)))),
    )
}
