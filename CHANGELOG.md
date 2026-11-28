# Changelog

Toutes les modifications notables de ce projet sont documentées ici.

Le format suit [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/)
et le projet respecte le [Versionnage sémantique](https://semver.org/lang/fr/) :
`MAJEUR.MINEUR.CORRECTIF`, tags Git `vX.Y.Z` identiques à la version de `Cargo.toml`.

## [Non publié]

## [0.4.0] - 2026-11-28

### Ajouté
- Nouvelle interface conforme à la charte Rusty Suite : menu latéral réductible avec
  icônes, barre d'outils (profil actif), barre d'état (réseau, langues, mise à jour),
  notifications, logo animé, navigation clavier (Alt+1 à Alt+5, F6, F1).
- Mise à jour automatique avec contrôle SHA-256 et retour à la version précédente si la
  nouvelle ne démarre pas.
- Instance unique et journal de l'application avec rotation.

### Modifié
- Thème sombre unique de la charte (le choix clair / système est retiré).
- Programme portable : données dans `rusty_qr-data` à côté de l'exécutable ; un ancien
  dossier contenant une bibliothèque ou des profils reste utilisé.
- Langue choisie depuis la fenêtre Langue commune (l'ancien choix est repris).
- Textes et boutons : emoji décoratifs retirés des menus, titres et messages, tirets
  cadratins remplacés ; les boutons à icône utilisent la police d'icônes de la charte.
- Un dossier `rusty_qr-data` déjà présent à côté de l'exécutable l'emporte sur l'ancien
  dossier.
- Fenêtre centrée de 1180 x 560 pt (écran 1080p à 150 %).

### Supprimé
- Anciennes fenêtres Langue et À propos, et leurs textes.

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

[Non publié]: https://github.com/rusty-suite/rusty_qr/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/rusty-suite/rusty_qr/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/rusty-suite/rusty_qr/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/rusty-suite/rusty_qr/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/rusty-suite/rusty_qr/compare/Win_%26_Linux...v0.2.0
[0.1.0]: https://github.com/rusty-suite/rusty_qr/releases/tag/Win_%26_Linux
