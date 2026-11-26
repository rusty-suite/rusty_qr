use std::collections::HashMap;
use std::path::{Path, PathBuf};

const GITHUB_DEFAULT_LANG: &str =
    "https://raw.githubusercontent.com/rusty-suite/rusty_qr/main/lang/EN_en.default.toml";
const GITHUB_LANG_API: &str =
    "https://api.github.com/repos/rusty-suite/rusty_qr/contents/lang";

/// Stem of the universal fallback language (always compiled in).
const DEFAULT_STEM: &str = "EN_en.default";

/// Every language shipped with the binary. They are written to the local
/// `lang/` folder on first run and also serve as in-memory fallback, so a
/// stale or incomplete local file never shows raw keys to the user.
pub const BUNDLED_LANGS: &[(&str, &str)] = &[
    ("EN_en.default.toml", include_str!("../lang/EN_en.default.toml")),
    ("FR_fr.toml",         include_str!("../lang/FR_fr.toml")),
    ("CH_fr.toml",         include_str!("../lang/CH_fr.toml")),
    ("DE_de.toml",         include_str!("../lang/DE_de.toml")),
    ("CH_de.toml",         include_str!("../lang/CH_de.toml")),
    ("IT_it.toml",         include_str!("../lang/IT_it.toml")),
    ("CH_it.toml",         include_str!("../lang/CH_it.toml")),
];

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
    pub path: Option<PathBuf>,
    pub remote_url: Option<String>,
    pub is_local: bool,
    pub is_remote: bool,
}

#[derive(Clone)]
pub struct RemoteLangInfo {
    pub stem: String,
    pub download_url: String,
}

// ─── Implémentation ───────────────────────────────────────────────────────────

impl Default for Lang {
    /// The bundled default language (English).
    fn default() -> Self {
        Self { strings: bundled_strings(DEFAULT_STEM), active_stem: DEFAULT_STEM.to_string() }
    }
}

impl Lang {
    /// Returns the translation of `key`.
    ///
    /// Lookup order: active language → bundled English. When a key is
    /// unknown everywhere (e.g. a free-form message coming from a template
    /// or a library error), the key itself is returned unchanged.
    pub fn t(&self, key: &str) -> String {
        self.strings
            .get(key)
            .or_else(|| default_strings().get(key))
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }

    /// Like [`Lang::t`], then replaces each `{name}` placeholder with its value.
    pub fn tf(&self, key: &str, args: &[(&str, &str)]) -> String {
        let mut s = self.t(key);
        for (name, value) in args {
            s = s.replace(&format!("{{{name}}}"), value);
        }
        s
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
        ensure_bundled_languages(&lang_dir);

        // 1. Préférence enregistrée
        if let Some(stem) = saved_choice(work_dir) {
            let path = lang_dir.join(format!("{stem}.toml"));
            if path.exists() {
                return match parse_file(&path, stem.clone()) {
                    Ok(lang) => (lang, None),
                    Err(e)   => load_error(&e),
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
                    Err(e) => load_error(&e),
                }
            }
            None => (Self::default(), None),
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
    pub fn list_available(work_dir: &Path, remote_langs: &[RemoteLangInfo]) -> Vec<LangInfo> {
        let lang_dir = work_dir.join("lang");
        let mut infos: Vec<LangInfo> = toml_files_in(&lang_dir)
            .into_iter()
            .map(|path| {
                let stem       = file_stem(&path);
                let is_default = stem.ends_with(".default");
                let display    = read_lang_name(&path).unwrap_or_else(|| friendly_name(&stem));
                LangInfo {
                    stem,
                    display,
                    is_default,
                    path: Some(path),
                    remote_url: None,
                    is_local: true,
                    is_remote: false,
                }
            })
            .collect();

        for remote in remote_langs {
            if let Some(existing) = infos.iter_mut().find(|info| info.stem.eq_ignore_ascii_case(&remote.stem)) {
                existing.remote_url = Some(remote.download_url.clone());
                existing.is_remote = true;
            } else {
                let stem = remote.stem.clone();
                infos.push(LangInfo {
                    display: friendly_name(&stem),
                    is_default: stem.ends_with(".default"),
                    path: None,
                    remote_url: Some(remote.download_url.clone()),
                    is_local: false,
                    is_remote: true,
                    stem,
                });
            }
        }

        infos.sort_by(|a, b| {
            a.is_default.cmp(&b.is_default).then(a.stem.cmp(&b.stem))
        });
        infos
    }

    pub fn download_to_lang_dir(work_dir: &Path, info: &RemoteLangInfo) -> Result<PathBuf, String> {
        let lang_dir = work_dir.join("lang");
        let _ = std::fs::create_dir_all(&lang_dir);
        let body = ureq::get(&info.download_url)
            .timeout(std::time::Duration::from_secs(15))
            .call()
            .map_err(|e| e.to_string())?
            .into_string()
            .map_err(|e| e.to_string())?;
        let dest = lang_dir.join(format!("{}.toml", info.stem));
        std::fs::write(&dest, body).map_err(|e| e.to_string())?;
        Ok(dest)
    }

    pub fn fetch_remote_index() -> Result<Vec<RemoteLangInfo>, String> {
        let resp = ureq::get(GITHUB_LANG_API)
            .set("User-Agent", "rusty_qr")
            .timeout(std::time::Duration::from_secs(8))
            .call()
            .map_err(|e| e.to_string())?;
        let json: serde_json::Value = resp.into_json().map_err(|e| e.to_string())?;
        let arr = json.as_array().ok_or_else(|| "invalid language index".to_string())?;

        let mut out: Vec<RemoteLangInfo> = arr.iter().filter_map(|item| {
            let kind = item.get("type")?.as_str()?;
            if kind != "file" {
                return None;
            }
            let name = item.get("name")?.as_str()?;
            if !name.ends_with(".toml") {
                return None;
            }
            let stem = Path::new(name).file_stem()?.to_str()?.to_string();
            let download_url = item.get("download_url")?.as_str()?.to_string();
            Some(RemoteLangInfo { stem, download_url })
        }).collect();

        out.sort_by(|a, b| a.stem.cmp(&b.stem));
        Ok(out)
    }

    pub fn stem_compact_code(stem: &str) -> String {
        let base = stem.split('.').next().unwrap_or(stem);
        let mut it = base.splitn(2, '_');
        let country = it.next().unwrap_or("").to_uppercase();
        let lang = it.next().unwrap_or("").to_lowercase();
        if lang.is_empty() {
            return base.to_lowercase();
        }
        if country.is_empty() || country.eq_ignore_ascii_case(&lang) {
            lang
        } else {
            format!("{lang}_{country}")
        }
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
                    match parse_toml_str(&body, DEFAULT_STEM.into()) {
                        Ok(lang) => (lang, None),
                        Err(e)   => load_error(&e),
                    }
                }
                Err(e) => load_error(&e.to_string()),
            },
            // Offline: the bundled English language is always available.
            Err(_) => (Self::default(), None),
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
    // Complete a stale local file with the keys of the bundled version of
    // the same language (local values always win).
    for (k, v) in bundled_strings(&stem) {
        strings.entry(k).or_insert(v);
    }
    Ok(Lang { strings, active_stem: stem })
}

/// Default language + error message shown in the start-up warning dialog.
fn load_error(detail: &str) -> (Lang, Option<String>) {
    let lang = Lang::default();
    let msg = lang.tf("errors.lang_load", &[("error", detail)]);
    (lang, Some(msg))
}

/// Flattened strings of a bundled language, empty if `stem` is not bundled.
fn bundled_strings(stem: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let file = format!("{stem}.toml");
    if let Some((_, content)) = BUNDLED_LANGS.iter().find(|(name, _)| *name == file) {
        if let Ok(table) = toml::from_str::<toml::Table>(content) {
            flatten_table("", &table, &mut out);
        }
    }
    out
}

/// Bundled English strings, parsed once.
fn default_strings() -> &'static HashMap<String, String> {
    static CACHE: std::sync::OnceLock<HashMap<String, String>> = std::sync::OnceLock::new();
    CACHE.get_or_init(|| bundled_strings(DEFAULT_STEM))
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

fn ensure_bundled_languages(lang_dir: &Path) {
    for &(name, content) in BUNDLED_LANGS {
        let dest = lang_dir.join(name);
        if !dest.exists() {
            let _ = std::fs::write(dest, content);
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn keys(stem: &str) -> BTreeSet<String> {
        bundled_strings(stem).into_keys().collect()
    }

    /// Every bundled language must define exactly the keys of the English file.
    #[test]
    fn all_languages_have_the_same_keys() {
        let reference = keys(DEFAULT_STEM);
        assert!(!reference.is_empty());
        for (file, _) in BUNDLED_LANGS {
            let stem = file.trim_end_matches(".toml");
            let k = keys(stem);
            let missing: Vec<_> = reference.difference(&k).collect();
            let extra: Vec<_>   = k.difference(&reference).collect();
            assert!(missing.is_empty() && extra.is_empty(),
                "{file}: missing {missing:?}, extra {extra:?}");
        }
    }

    /// Every translation key written in the source code must exist in English.
    #[test]
    fn keys_used_in_code_exist() {
        let reference = keys(DEFAULT_STEM);
        let sections: BTreeSet<&str> = reference.iter().filter_map(|k| k.split('.').next()).collect();
        let mut missing = Vec::new();
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut stack = vec![src];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).unwrap().flatten() {
                let path = entry.path();
                if path.is_dir() { stack.push(path); continue; }
                if path.extension().and_then(|e| e.to_str()) != Some("rs") { continue; }
                let text = std::fs::read_to_string(&path).unwrap();
                for lit in text.split('"').skip(1).step_by(2) {
                    let is_key = lit.contains('.')
                        && lit.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '.')
                        && sections.contains(lit.split('.').next().unwrap())
                        // file names such as "library.json" are not keys
                        && !matches!(lit.rsplit('.').next(), Some("json" | "toml" | "txt" | "svg" | "png"));
                    if is_key && !reference.contains(lit) {
                        missing.push(format!("{}: {lit}", path.display()));
                    }
                }
            }
        }
        assert!(missing.is_empty(), "keys missing from EN: {missing:#?}");
    }

    /// Built-in SVG themes are looked up by id at runtime.
    #[test]
    fn builtin_templates_are_translated() {
        let reference = keys(DEFAULT_STEM);
        for t in crate::template::BUILTIN {
            for suffix in ["name", "desc"] {
                let k = format!("template.{}.{suffix}", t.id);
                assert!(reference.contains(&k), "missing {k}");
            }
        }
    }

    /// No language file may still carry a bilingual / foreign window title.
    #[test]
    fn language_titles_are_localized() {
        for (file, _) in BUNDLED_LANGS {
            let stem = file.trim_end_matches(".toml");
            let title = bundled_strings(stem).remove("lang_page.title").unwrap();
            assert!(!title.contains('/'), "{file}: {title}");
        }
    }

    #[test]
    fn missing_key_falls_back_to_english_then_key() {
        let mut lang = Lang::default();
        lang.strings.clear();
        assert_eq!(lang.t("lang_page.title"), "Language");
        assert_eq!(lang.t("no.such.key"), "no.such.key");
        assert_eq!(lang.tf("preview.profile", &[("name", "X")]), "Profile: X");
    }
}
