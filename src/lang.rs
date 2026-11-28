//! Textes de Rusty QR.
//!
//! Le chargement (packs intégrés, fichiers locaux, mise à jour depuis GitHub, repli sur
//! la version intégrée puis sur l'anglais) est fait par `rusty_core::lang` (charte Rusty
//! Suite, section 05). `Lang` est celui du kit : `lang.t(clé)` et `lang.tf(clé, variables)`.

pub use rusty_core::Lang;

/// Packs de langue intégrés au binaire (nom de fichier historique pour l'anglais).
pub const PACKS: rusty_core::lang::Packs = &[
    ("EN_en", include_str!("../lang/EN_en.default.toml")),
    ("FR_fr", include_str!("../lang/FR_fr.toml")),
    ("CH_fr", include_str!("../lang/CH_fr.toml")),
    ("DE_de", include_str!("../lang/DE_de.toml")),
    ("CH_de", include_str!("../lang/CH_de.toml")),
    ("IT_it", include_str!("../lang/IT_it.toml")),
    ("CH_it", include_str!("../lang/CH_it.toml")),
];

/// Charge la langue `chosen` (stem, vide = langue du système).
pub fn load(work_dir: &std::path::Path, chosen: &str) -> Lang {
    Lang::load(PACKS, work_dir, chosen)
}

/// Ancien choix de langue (`lang_chosen.txt`) converti en stem, pour reprendre le
/// choix des versions précédentes.
pub fn legacy_choice(work_dir: &std::path::Path) -> Option<String> {
    let name = std::fs::read_to_string(work_dir.join("lang_chosen.txt")).ok()?;
    let name = name.trim();
    let stem = name.strip_suffix(".toml").unwrap_or(name);
    let stem = stem.strip_suffix(".default").unwrap_or(stem);
    (!stem.is_empty()).then(|| stem.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::path::Path;

    fn english() -> Lang {
        load(&std::env::temp_dir().join("rusty_qr_lang_test"), "EN_en")
    }

    /// Toutes les langues ont exactement les clés de l'anglais.
    #[test]
    fn all_languages_have_the_same_keys() {
        assert_eq!(rusty_core::lang::check_packs(PACKS), Vec::<String>::new());
    }

    /// Chaque clé de traduction écrite dans le code existe en anglais.
    #[test]
    fn keys_used_in_code_exist() {
        let lang = english();
        let reference: BTreeSet<&String> = lang.keys().collect();
        let sections: BTreeSet<&str> = reference.iter().filter_map(|k| k.split('.').next()).collect();
        let mut missing = Vec::new();
        let mut stack = vec![Path::new(env!("CARGO_MANIFEST_DIR")).join("src")];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).unwrap().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                    continue;
                }
                let text = std::fs::read_to_string(&path).unwrap();
                for lit in text.split('"').skip(1).step_by(2) {
                    let is_key = lit.contains('.')
                        && lit.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '.')
                        && sections.contains(lit.split('.').next().unwrap())
                        // Les noms de fichiers comme « library.json » ne sont pas des clés.
                        && !matches!(lit.rsplit('.').next(), Some("json" | "toml" | "txt" | "svg" | "png"));
                    if is_key && !lang.has(lit) {
                        missing.push(format!("{}: {lit}", path.display()));
                    }
                }
            }
        }
        assert!(missing.is_empty(), "clés absentes de l'anglais : {missing:#?}");
    }

    /// Les thèmes SVG intégrés sont cherchés par identifiant à l'exécution.
    #[test]
    fn builtin_templates_are_translated() {
        let lang = english();
        for t in crate::template::BUILTIN {
            for suffix in ["name", "desc"] {
                let k = format!("template.{}.{suffix}", t.id);
                assert!(lang.has(&k), "clé manquante {k}");
            }
        }
    }

    /// Charte, 05 : pas de tiret cadratin ; orthographe suisse en CH-de.
    #[test]
    fn texts_follow_the_charter() {
        let ch_de = PACKS.iter().find(|(s, _)| *s == "CH_de").unwrap().1;
        assert!(!ch_de.contains('ß'), "CH_de doit utiliser « ss »");
    }

    #[test]
    fn missing_key_falls_back_to_the_key() {
        let lang = english();
        assert_eq!(lang.t("no.such.key"), "no.such.key");
        assert_eq!(lang.tf("preview.profile", &[("name", "X")]), "Profile: X");
    }
}
