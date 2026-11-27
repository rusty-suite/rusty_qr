# Changelog

Toutes les modifications notables de ce projet sont documentées ici.

Le format suit [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/)
et le projet respecte le [Versionnage sémantique](https://semver.org/lang/fr/) :
`MAJEUR.MINEUR.CORRECTIF`, tags Git `vX.Y.Z` identiques à la version de `Cargo.toml`.

## [Non publié]

## [0.3.0] - 2026-11-28

### Modifié
- Licence : GNU GPL v3 ou ultérieure (`GPL-3.0-or-later`) à la place de PolyForm Noncommercial 1.0.0. Le programme devient un logiciel libre, utilisable par tous.

## [0.2.1] - 2026-11-27

### Modifié
- README en anglais (`README.md`), version française dans `README.fr.md`.
- Licence PolyForm Noncommercial 1.0.0 déclarée dans `Cargo.toml`.

### Corrigé
- Le README citait un bouton « Charger un fichier SVG » au lieu de « 📁 Fichier SVG ».

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
- Le bouton « Ouvrir » du dossier des langues fonctionne aussi sous Linux et macOS.

### Supprimé
- Ancien export PDF des cartes non utilisé (remplacé par l'export rastérisé).

## [0.1.0] - 2026-04-26

Première pré-release publique (tag historique `Win_&_Linux`).

[Non publié]: https://github.com/rusty-suite/rusty_qr/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/rusty-suite/rusty_qr/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/rusty-suite/rusty_qr/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/rusty-suite/rusty_qr/compare/Win_%26_Linux...v0.2.0
[0.1.0]: https://github.com/rusty-suite/rusty_qr/releases/tag/Win_%26_Linux
