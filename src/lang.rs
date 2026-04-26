use std::collections::HashMap;
use std::path::{Path, PathBuf};

const GITHUB_DEFAULT_LANG: &str =
    "https://raw.githubusercontent.com/rusty-suite/rusty_qr/main/lang/EN_en.default.toml";

// ─── Types publics ────────────────────────────────────────────────────────────

pub struct Lang {
    strings: HashMap<String, String>,
    /// Stem du fichier chargé, ex. "CH_fr" ou "EN_en.default". Vide si défaut.
    pub active_stem: String,
}

pub struct LangInfo {
    /// Stem du fichier sans extension, ex. "CH_fr" ou "EN_en.default"
    pub stem: String,
    /// Nom affiché à l'utilisateur, lu depuis `app.lang_name` ou dérivé
    pub display: String,
    /// Vrai si le fichier porte le suffixe `.default`
    pub is_default: bool,
    pub path: PathBuf,
}

// ─── Implémentation ───────────────────────────────────────────────────────────

impl Default for Lang {
    fn default() -> Self {
        Self { strings: HashMap::new(), active_stem: String::new() }
    }
}

impl Lang {
    /// Retourne la traduction de `key`, ou `key` lui-même si absent.
    pub fn t(&self, key: &str) -> String {
        self.strings.get(key).cloned().unwrap_or_else(|| key.to_string())
    }

    /// Charge la langue depuis `{work_dir}/lang/`.
    ///
    /// Ordre de priorité :
    ///   1. Préférence sauvegardée dans `lang_chosen.txt`
    ///   2. Correspondance avec la locale système
    ///   3. Fichier `.default.toml`
    ///   4. Téléchargement GitHub si le dossier est vide
    ///
    /// Retourne `(Lang, Option<message_erreur>)`.
    pub fn load(work_dir: &Path) -> (Self, Option<String>) {
        let lang_dir = work_dir.join("lang");
        let _ = std::fs::create_dir_all(&lang_dir);

        // 1. Préférence enregistrée
        if let Some(stem) = saved_choice(work_dir) {
            let path = lang_dir.join(format!("{stem}.toml"));
            if path.exists() {
                return match parse_file(&path, stem.clone()) {
                    Ok(lang) => (lang, None),
                    Err(e)   => (Self::default(), Some(format!("Erreur langue '{stem}' : {e}"))),
                };
            }
        }

        // 2. Liste des fichiers disponibles
        let files = toml_files_in(&lang_dir);
        if files.is_empty() {
            return Self::download_default(&lang_dir);
        }

        // 3. Auto-détection locale
        let locale               = detect_locale();
        let (lang_code, country) = parse_locale(&locale);
        let wanted               = format!("{}_{}", country.to_uppercase(), lang_code.to_lowercase());

        let chosen = files.iter()
            .find(|p| file_stem(p).eq_ignore_ascii_case(&wanted))
            .or_else(|| files.iter().find(|p| {
                let s = file_stem(p);
                !s.ends_with(".default")
                    && s.split('_').nth(1)
                        .and_then(|x| x.split('.').next())
                        .map(|x| x.eq_ignore_ascii_case(&lang_code))
                        .unwrap_or(false)
            }))
            .or_else(|| files.iter().find(|p| file_stem(p).ends_with(".default")))
            .or_else(|| files.first());

        match chosen {
            Some(p) => {
                let stem = file_stem(p);
                match parse_file(p, stem) {
                    Ok(l)  => (l, None),
                    Err(e) => (Self::default(), Some(format!("Erreur chargement langue : {e}"))),
                }
            }
            None => (Self::default(), Some("Aucun fichier de langue trouvé.".into())),
        }
    }

    /// Charge un fichier de langue spécifique par son chemin.
    pub fn load_file(path: &Path) -> Result<Self, String> {
        parse_file(path, file_stem(path))
    }

    /// Sauvegarde le stem choisi dans `{work_dir}/lang_chosen.txt`.
    pub fn save_choice(work_dir: &Path, stem: &str) {
        let _ = std::fs::write(work_dir.join("lang_chosen.txt"), stem);
    }

    /// Liste tous les fichiers `.toml` disponibles dans `{work_dir}/lang/`.
    /// Résultat trié : non-default d'abord, puis default, par ordre alphabétique.
    pub fn list_available(work_dir: &Path) -> Vec<LangInfo> {
        let lang_dir = work_dir.join("lang");
        let mut infos: Vec<LangInfo> = toml_files_in(&lang_dir)
            .into_iter()
            .map(|path| {
                let stem       = file_stem(&path);
                let is_default = stem.ends_with(".default");
                let display    = read_lang_name(&path).unwrap_or_else(|| friendly_name(&stem));
                LangInfo { stem, display, is_default, path }
            })
            .collect();

        infos.sort_by(|a, b| {
            a.is_default.cmp(&b.is_default).then(a.stem.cmp(&b.stem))
        });
        infos
    }

    // ── Téléchargement du fichier de secours depuis GitHub ───────────────────

    fn download_default(lang_dir: &Path) -> (Self, Option<String>) {
        match ureq::get(GITHUB_DEFAULT_LANG)
            .timeout(std::time::Duration::from_secs(15))
            .call()
        {
            Ok(resp) => match resp.into_string() {
                Ok(body) => {
                    let dest = lang_dir.join("EN_en.default.toml");
                    let _ = std::fs::write(&dest, &body);
                    match parse_toml_str(&body, "EN_en.default".into()) {
                        Ok(lang) => (lang, None),
                        Err(e)   => (Self::default(), Some(format!("Erreur parsing langue : {e}"))),
                    }
                }
                Err(_) => (Self::default(), Some("Erreur lecture réponse réseau.".into())),
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

fn parse_file(path: &Path, stem: String) -> Result<Lang, String> {
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    parse_toml_str(&content, stem)
}

fn parse_toml_str(content: &str, stem: String) -> Result<Lang, String> {
    let table: toml::Table = toml::from_str(content).map_err(|e| e.to_string())?;
    let mut strings = HashMap::new();
    flatten_table("", &table, &mut strings);
    Ok(Lang { strings, active_stem: stem })
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

// ─── Utilitaires ─────────────────────────────────────────────────────────────

fn toml_files_in(dir: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("toml"))
        .collect()
}

fn saved_choice(work_dir: &Path) -> Option<String> {
    std::fs::read_to_string(work_dir.join("lang_chosen.txt"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn read_lang_name(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let table: toml::Table = toml::from_str(&content).ok()?;
    table.get("app")?.as_table()?.get("lang_name")?.as_str().map(|s| s.to_string())
}

/// Dérive un nom d'affichage depuis le stem si `app.lang_name` est absent.
/// "CH_fr" → "fr (CH)", "EN_en.default" → "en (EN) [default]"
fn friendly_name(stem: &str) -> String {
    let base    = stem.split('.').next().unwrap_or(stem);
    let mut it  = base.splitn(2, '_');
    let country = it.next().unwrap_or("");
    let lang    = it.next().unwrap_or(stem);
    if stem.ends_with(".default") {
        format!("{lang} ({country}) [default]")
    } else {
        format!("{lang} ({country})")
    }
}

fn file_stem(path: &Path) -> String {
    path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string()
}

fn detect_locale() -> String {
    for var in ["LANG", "LANGUAGE", "LC_ALL", "LC_MESSAGES"] {
        if let Ok(val) = std::env::var(var) {
            let val = val.trim().to_string();
            if !val.is_empty() && val != "C" && val != "POSIX" {
                return val.split('.').next().unwrap_or(&val).to_string();
            }
        }
    }
    "en_US".to_string()
}

fn parse_locale(locale: &str) -> (String, String) {
    let norm    = locale.replace('-', "_");
    let mut it  = norm.splitn(2, '_');
    let lang    = it.next().unwrap_or("en").to_string();
    let country = it.next().unwrap_or("US").to_string();
    (lang, country)
}
