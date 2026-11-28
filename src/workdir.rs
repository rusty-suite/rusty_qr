//! Dossier de travail de Rusty QR (bibliothèque, profils, langues, réglages).
//!
//! Les programmes de la suite sont portables : par défaut, les données vivent dans
//! `rusty_qr-data` à côté de l'exécutable (`rusty_core::paths`, avec repli sur le dossier
//! de configuration de l'utilisateur si celui de l'exécutable n'est pas inscriptible).
//! Pour ne jamais « perdre » la bibliothèque ou les profils d'une version précédente,
//! l'ancien dossier (`%APPDATA%\rusty-suite\rusty_qr` ou `~/rusty_qr`) reste utilisé
//! s'il en contient déjà.

use std::path::PathBuf;
use std::sync::OnceLock;

/// Identifiant de l'application (dossier portable, journal, mises à jour).
pub const APP_ID: &str = "rusty_qr";

fn legacy_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(appdata) = std::env::var("APPDATA") {
        dirs.push(PathBuf::from(appdata).join("rusty-suite").join(APP_ID));
    }
    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(APP_ID));
    }
    dirs
}

/// Dossier de travail, déterminé une fois au démarrage.
pub fn work_dir() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        legacy_dirs()
            .into_iter()
            .find(|d| d.join("library.json").is_file() || d.join("profiles.json").is_file())
            .unwrap_or_else(|| rusty_core::paths::data_dir(APP_ID))
    })
    .clone()
}
