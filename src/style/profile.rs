use serde::{Deserialize, Serialize};

fn default_module_px() -> u32 { 10 }
fn default_quiet_zone() -> u32 { 4 }
fn default_logo_ratio() -> f32 { 0.0 }
fn default_logo_pos()   -> f32 { 0.5 }  // center
fn default_logo_pad()   -> u32 { 4 }

#[derive(Clone, Serialize, Deserialize)]
pub struct StyleProfile {
    pub name: String,
    pub fg: [u8; 3],
    pub bg: [u8; 3],

    #[serde(default = "default_module_px")]
    pub module_px: u32,
    #[serde(default = "default_quiet_zone")]
    pub quiet_zone: u32,

    // ── Logo / image incrustée ──────────────────────────────────────────────
    #[serde(default)]
    pub logo_path: String,
    /// Taille relative (0.0 = aucun, max 0.30 recommandé)
    #[serde(default = "default_logo_ratio")]
    pub logo_ratio: f32,
    /// Position horizontale : 0.0 = gauche, 0.5 = centre, 1.0 = droite
    #[serde(default = "default_logo_pos")]
    pub logo_pos_x: f32,
    /// Position verticale   : 0.0 = haut,   0.5 = centre, 1.0 = bas
    #[serde(default = "default_logo_pos")]
    pub logo_pos_y: f32,
    /// Marge blanche autour du logo (px)
    #[serde(default = "default_logo_pad")]
    pub logo_padding: u32,
}

impl Default for StyleProfile {
    fn default() -> Self {
        Self {
            name: "Défaut".into(),
            fg: [0, 0, 0],
            bg: [255, 255, 255],
            module_px: 10,
            quiet_zone: 4,
            logo_path: String::new(),
            logo_ratio: 0.0,
            logo_pos_x: 0.5,
            logo_pos_y: 0.5,
            logo_padding: 4,
        }
    }
}

impl StyleProfile {
    pub fn named(name: &str) -> Self {
        Self { name: name.into(), ..Default::default() }
    }
    pub fn fg_rgba(&self) -> [u8; 4] { [self.fg[0], self.fg[1], self.fg[2], 255] }
    pub fn bg_rgba(&self) -> [u8; 4] { [self.bg[0], self.bg[1], self.bg[2], 255] }
    pub fn has_logo(&self) -> bool    { !self.logo_path.is_empty() && self.logo_ratio > 0.001 }
}

// ─── Persistence ─────────────────────────────────────────────────────────────

fn profiles_path() -> std::path::PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let dir = base.join("rusty_qr");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("profiles.json")
}

pub fn load_profiles() -> Vec<StyleProfile> {
    let path = profiles_path();
    if let Ok(data) = std::fs::read_to_string(&path) {
        if let Ok(p) = serde_json::from_str::<Vec<StyleProfile>>(&data) {
            if !p.is_empty() { return p; }
        }
    }
    vec![
        StyleProfile::named("Défaut (noir/blanc)"),
        StyleProfile {
            name: "Bleu foncé".into(),
            fg: [20, 40, 120], bg: [240, 245, 255],
            ..Default::default()
        },
        StyleProfile {
            name: "Vert tech".into(),
            fg: [30, 100, 50], bg: [230, 255, 235],
            ..Default::default()
        },
    ]
}

pub fn save_profiles(profiles: &[StyleProfile]) {
    if let Ok(json) = serde_json::to_string_pretty(profiles) {
        let _ = std::fs::write(profiles_path(), json);
    }
}
