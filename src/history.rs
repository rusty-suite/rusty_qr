//! Bibliothèque — configurations QR sauvegardées, rechargeables et modifiables.

use serde::{Deserialize, Serialize};
use crate::card::CardConfig;
use crate::qr::types::QrForm;

// ─── Template state ───────────────────────────────────────────────────────────

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct SavedField {
    pub var:     String,
    pub value:   String,
    #[serde(default = "bool_true")]
    pub visible: bool,
}
fn bool_true() -> bool { true }

#[derive(Clone, Serialize, Deserialize)]
pub struct SavedColor {
    pub var:   String,
    pub value: [u8; 3],
}

/// État complet du concepteur de cartes, persisté avec chaque entrée de bibliothèque.
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct SavedTemplateState {
    /// Identifiant du thème intégré ("classic", "dark", …) ou `None`
    pub builtin_id: Option<String>,
    /// Contenu SVG si thème personnalisé ou distant (non-intégré)
    pub custom_svg: Option<String>,
    /// Valeurs des champs texte {{F0}}…{{F4}}
    pub fields: Vec<SavedField>,
    /// Valeurs des couleurs {{C0}}…{{C4}}
    pub colors: Vec<SavedColor>,
    /// Configuration de la carte (gabarit, couleurs, champs de texte)
    pub card: Option<CardConfig>,
}

// ─── Library entry ────────────────────────────────────────────────────────────

#[derive(Clone, Serialize, Deserialize)]
pub struct LibraryEntry {
    pub id:   u64,
    pub name: String,
    pub date: String,   // "YYYY-MM-DD HH:MM"
    pub form: QrForm,
    #[serde(default)]
    pub template: SavedTemplateState,
}

// ─── Persistence ─────────────────────────────────────────────────────────────

fn library_path() -> std::path::PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let dir = base.join("rusty_qr");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("library.json")
}

fn now_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let min  = (secs / 60) % 60;
    let hour = (secs / 3600) % 24;
    let days = secs / 86400;
    let (y, mo, d) = days_to_ymd(days);
    format!("{y:04}-{mo:02}-{d:02}  {hour:02}:{min:02}")
}

fn days_to_ymd(mut d: u64) -> (u64, u64, u64) {
    let mut y = 1970u64;
    loop {
        let dy = if is_leap(y) { 366 } else { 365 };
        if d < dy { break; }
        d -= dy; y += 1;
    }
    let months = if is_leap(y) {
        [31u64,29,31,30,31,30,31,31,30,31,30,31]
    } else {
        [31u64,28,31,30,31,30,31,31,30,31,30,31]
    };
    let mut mo = 1u64;
    for &dm in &months {
        if d < dm { break; }
        d -= dm; mo += 1;
    }
    (y, mo, d + 1)
}

fn is_leap(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn next_id(entries: &[LibraryEntry]) -> u64 {
    entries.iter().map(|e| e.id).max().unwrap_or(0) + 1
}

pub fn load_library() -> Vec<LibraryEntry> {
    std::fs::read_to_string(library_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_library(entries: &[LibraryEntry]) {
    if let Ok(json) = serde_json::to_string_pretty(entries) {
        let _ = std::fs::write(library_path(), json);
    }
}

pub fn add_entry(
    entries:  &mut Vec<LibraryEntry>,
    name:     String,
    form:     QrForm,
    template: SavedTemplateState,
) {
    entries.insert(0, LibraryEntry {
        id: next_id(entries),
        name,
        date: now_string(),
        form,
        template,
    });
    save_library(entries);
}

pub fn remove_entry(entries: &mut Vec<LibraryEntry>, id: u64) {
    entries.retain(|e| e.id != id);
    save_library(entries);
}
