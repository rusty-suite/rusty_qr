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

/// Dossier portable déjà présent à côté de l'exécutable : il l'emporte toujours sur
/// l'ancien dossier (choix explicite de l'utilisateur).
fn existing_portable_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?.join(format!("{APP_ID}-data"));
    dir.is_dir().then_some(dir)
}

/// Dossier de travail, déterminé une fois au démarrage : dossier portable existant,
/// sinon ancien dossier s'il contient des données, sinon nouveau dossier portable.
pub fn work_dir() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        if let Some(dir) = existing_portable_dir() {
            return dir;
        }
        legacy_dirs()
            .into_iter()
            .find(|d| d.join("library.json").is_file() || d.join("profiles.json").is_file())
            .unwrap_or_else(|| rusty_core::paths::data_dir(APP_ID))
    })
    .clone()
}
