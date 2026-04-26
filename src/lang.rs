use std::collections::HashMap;
use std::path::Path;

const GITHUB_DEFAULT_LANG: &str =
    "https://raw.githubusercontent.com/rusty-suite/rusty_qr/main/lang/EN_en.default.toml";

// ─── Struct principale ────────────────────────────────────────────────────────

pub struct Lang {
    strings: HashMap<String, String>,
}

impl Default for Lang {
    fn default() -> Self {
        Self { strings: HashMap::new() }
    }
}

impl Lang {
    /// Retourne la traduction associée à `key`, ou `key` lui-même si absent.
    pub fn t(&self, key: &str) -> String {
        self.strings.get(key).cloned().unwrap_or_else(|| key.to_string())
    }

    /// Charge le fichier de langue approprié depuis `{work_dir}/lang/`.
    ///
    /// Retourne `(Lang, Option<message_erreur>)`.
    /// En l'absence de fichiers, tente le téléchargement GitHub.
    pub fn load(work_dir: &Path) -> (Self, Option<String>) {
        let lang_dir = work_dir.join("lang");
        let _ = std::fs::create_dir_all(&lang_dir);

        // Collecte tous les fichiers .toml du dossier lang/
        let toml_files: Vec<_> = std::fs::read_dir(&lang_dir)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("toml"))
            .collect();

        if toml_files.is_empty() {
            return Self::download_default(&lang_dir);
        }

        // Détermine la locale système puis cherche le meilleur fichier
        let locale               = detect_locale();
        let (lang_code, country) = parse_locale(&locale);
        let wanted_stem          = format!("{}_{}", country.to_uppercase(), lang_code.to_lowercase());

        let chosen = toml_files.iter()
            // 1. Correspondance exacte : CH_fr
            .find(|p| file_stem(p).eq_ignore_ascii_case(&wanted_stem))
            // 2. Même langue, n'importe quel pays (non-default)
            .or_else(|| toml_files.iter().find(|p| {
                let stem = file_stem(p);
                !stem.ends_with(".default")
                    && stem.split('_').nth(1)
                        .and_then(|s| s.split('.').next())
                        .map(|s| s.eq_ignore_ascii_case(&lang_code))
                        .unwrap_or(false)
            }))
            // 3. Fichier default (*.default.toml)
            .or_else(|| toml_files.iter().find(|p| file_stem(p).ends_with(".default")))
            // 4. N'importe quel fichier disponible
            .or_else(|| toml_files.first());

        match chosen.and_then(|p| parse_file(p).ok()) {
            Some(lang) => (lang, None),
            None => (
                Self::default(),
                Some("Erreur : impossible de lire le fichier de langue.".into()),
            ),
        }
    }

    // ─── Téléchargement du fichier par défaut depuis GitHub ──────────────────

    fn download_default(lang_dir: &Path) -> (Self, Option<String>) {
        match ureq::get(GITHUB_DEFAULT_LANG)
            .timeout(std::time::Duration::from_secs(15))
            .call()
        {
            Ok(resp) => match resp.into_string() {
                Ok(body) => {
                    let _ = std::fs::write(lang_dir.join("EN_en.default.toml"), &body);
                    match parse_toml_str(&body) {
                        Ok(lang) => (lang, None),
                        Err(e)   => (Self::default(), Some(format!("Erreur parsing langue : {e}"))),
                    }
                }
                Err(_) => (
                    Self::default(),
                    Some("Erreur lors de la lecture de la réponse réseau.".into()),
                ),
            },
            Err(_) => (
                Self::default(),
                Some(
                    "Ce programme a besoin d'un accès internet\n\
                     pour télécharger ses ressources linguistiques."
                        .into(),
                ),
            ),
        }
    }
}

// ─── Parsing TOML ────────────────────────────────────────────────────────────

fn parse_file(path: &Path) -> Result<Lang, String> {
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    parse_toml_str(&content)
}

fn parse_toml_str(content: &str) -> Result<Lang, String> {
    let table: toml::Table = toml::from_str(content).map_err(|e| e.to_string())?;
    let mut strings = HashMap::new();
    flatten_table("", &table, &mut strings);
    Ok(Lang { strings })
}

fn flatten_table(prefix: &str, table: &toml::Table, out: &mut HashMap<String, String>) {
    for (k, v) in table {
        let key = if prefix.is_empty() { k.clone() } else { format!("{prefix}.{k}") };
        match v {
            toml::Value::Table(t) => flatten_table(&key, t, out),
            toml::Value::String(s) => { out.insert(key, s.clone()); }
            other => { out.insert(key, other.to_string()); }
        }
    }
}

// ─── Détection de la locale système ─────────────────────────────────────────

fn detect_locale() -> String {
    for var in ["LANG", "LANGUAGE", "LC_ALL", "LC_MESSAGES"] {
        if let Ok(val) = std::env::var(var) {
            let val = val.trim().to_string();
            if !val.is_empty() && val != "C" && val != "POSIX" {
                // "fr_FR.UTF-8" → "fr_FR"
                return val.split('.').next().unwrap_or(&val).to_string();
            }
        }
    }
    "en_US".to_string()
}

/// "fr_CH" ou "fr-CH"  →  (lang = "fr", country = "CH")
fn parse_locale(locale: &str) -> (String, String) {
    let norm = locale.replace('-', "_");
    let mut it = norm.splitn(2, '_');
    let lang    = it.next().unwrap_or("en").to_string();
    let country = it.next().unwrap_or("US").to_string();
    (lang, country)
}

// ─── Utilitaires ─────────────────────────────────────────────────────────────

fn file_stem(path: &Path) -> String {
    path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string()
}
