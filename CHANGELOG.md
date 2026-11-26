# Changelog

Toutes les modifications notables de ce projet sont documentées ici.

Le format suit [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/)
et le projet respecte le [Versionnage sémantique](https://semver.org/lang/fr/) :
`MAJEUR.MINEUR.CORRECTIF`, tags Git `vX.Y.Z` identiques à la version de `Cargo.toml`.

## [Non publié]

## [0.2.0] - 2026-11-26

### Ajouté
- Traduction complète de l'interface dans les 7 langues (EN, FR, DE, IT, CH-fr, CH-de, CH-it) :
  création, profils, bibliothèque, export, aperçu, concepteur de cartes et thèmes intégrés.
- Les 7 langues sont intégrées au binaire.
- Tests garantissant que toutes les langues ont les mêmes clés et que chaque clé utilisée existe.
- Binaire Linux x86_64 publié avec la release.

### Modifié
- Une clé absente d'un fichier de langue local retombe sur la version intégrée,
  puis sur l'anglais : plus aucun nom de clé brut affiché.
- La version affichée dans « À propos » provient de `Cargo.toml`.

### Corrigé
- Le panneau de choix de langue restait en français (titre et messages d'état).
- Orthographe suisse dans CH-de (« ss » au lieu de « ß »).

### Supprimé
- Ancien export PDF des cartes non utilisé (remplacé par l'export rastérisé).

## [0.1.0] - 2026-04-26

Première pré-release publique (tag historique `Win_&_Linux`).

[Non publié]: https://github.com/rusty-suite/rusty_qr/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/rusty-suite/rusty_qr/compare/Win_%26_Linux...v0.2.0
[0.1.0]: https://github.com/rusty-suite/rusty_qr/releases/tag/Win_%26_Linux
