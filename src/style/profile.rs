use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct StyleProfile {
    pub name: String,
    pub fg: [u8; 3],       // foreground (dark modules)
    pub bg: [u8; 3],       // background (light modules)
    pub module_px: u32,    // pixels per module
    pub quiet_zone: u32,   // quiet zone in modules
    pub logo_path: String, // empty = none
    pub logo_ratio: f32,   // fraction of QR width for logo (0.0 = none, max 0.30)
    pub rounded: bool,     // rounded modules (visual effect)
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
            rounded: false,
        }
    }
}

impl StyleProfile {
    pub fn named(name: &str) -> Self {
        Self { name: name.into(), ..Default::default() }
    }

    pub fn fg_rgba(&self) -> [u8; 4] { [self.fg[0], self.fg[1], self.fg[2], 255] }
    pub fn bg_rgba(&self) -> [u8; 4] { [self.bg[0], self.bg[1], self.bg[2], 255] }
}

// ─── Persistence ─────────────────────────────────────────────────────────────

fn profiles_path() -> std::path::PathBuf {
    let base = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let dir = base.join("rusty_qr");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("profiles.json")
}

pub fn load_profiles() -> Vec<StyleProfile> {
    let path = profiles_path();
    if let Ok(data) = std::fs::read_to_string(&path) {
        if let Ok(profiles) = serde_json::from_str::<Vec<StyleProfile>>(&data) {
            if !profiles.is_empty() { return profiles; }
        }
    }
    // Defaults
    vec![
        StyleProfile::named("Défaut (noir/blanc)"),
        StyleProfile {
            name: "Bleu foncé".into(),
            fg: [20, 40, 120],
            bg: [240, 245, 255],
            module_px: 10,
            quiet_zone: 4,
            logo_path: String::new(),
            logo_ratio: 0.0,
            rounded: false,
        },
        StyleProfile {
            name: "Vert tech".into(),
            fg: [30, 100, 50],
            bg: [230, 255, 235],
            module_px: 10,
            quiet_zone: 4,
            logo_path: String::new(),
            logo_ratio: 0.0,
            rounded: false,
        },
    ]
}

pub fn save_profiles(profiles: &[StyleProfile]) {
    if let Ok(json) = serde_json::to_string_pretty(profiles) {
        let _ = std::fs::write(profiles_path(), json);
    }
}
