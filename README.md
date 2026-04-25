# RustyQR

Générateur de codes QR multi-formats avec profils de style, bibliothèque et export vectoriel.

## Fonctionnalités

- **11 types de QR** : URL, Texte, Wi-Fi, SMS, Téléphone, Email, vCard, MeCard, Géolocalisation, GS1, 2D-Doc
- **Micro QR M1–M4** implémenté nativement (sans dépendance externe)
- **Profils de style** : couleurs personnalisées, logo superposé avec positionnement libre
- **Bibliothèque** : sauvegarde et rechargement de configurations nommées
- **Concepteur de cartes** : 4 gabarits (Carte de visite, Étiquette, Badge, Flyer A6)
- **Thèmes SVG** : 4 thèmes intégrés + chargement de fichiers locaux + catalogue GitHub
- **Exports** : PNG, JPEG, SVG, PDF, EPS, DOCX

---

## Guide — Créer un thème SVG personnalisé

Les thèmes SVG permettent de personnaliser l'apparence des cartes exportées depuis le
**Concepteur de cartes**. Un thème est un fichier `.svg` contenant des marqueurs
`{{VARIABLE}}` que RustyQR remplace à l'export.

### Structure minimale d'un thème

```xml
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg"
     xmlns:xlink="http://www.w3.org/1999/xlink"
     width="{{W}}" height="{{H}}" viewBox="0 0 {{W}} {{H}}">

  <!-- Votre fond personnalisé -->
  <rect width="{{W}}" height="{{H}}" fill="{{BG}}" rx="8"/>

  <!-- Code QR (bloc généré automatiquement) -->
  {{QR_IMAGE}}

  <!-- Éléments d'accent selon le gabarit (barre, bandeau…) -->
  {{ACCENT_BLOCK}}

  <!-- Champs texte selon le gabarit -->
  {{TEXT_BLOCK}}

</svg>
```

### Variables disponibles

#### Dimensions et couleurs

| Variable | Contenu | Exemple |
|----------|---------|---------|
| `{{W}}` | Largeur du canvas en pixels | `321` |
| `{{H}}` | Hauteur du canvas en pixels | `204` |
| `{{BG}}` | Couleur de fond (#RRGGBB) | `#FFFFFF` |
| `{{FG}}` | Couleur de texte (#RRGGBB) | `#1E1E1E` |
| `{{AC}}` | Couleur d'accent (#RRGGBB) | `#3264C8` |

#### Position du code QR

| Variable | Contenu |
|----------|---------|
| `{{QR_X}}` | Position X du QR en pixels |
| `{{QR_Y}}` | Position Y du QR en pixels |
| `{{QR_SZ}}` | Taille (largeur = hauteur) du QR en pixels |

#### Blocs pré-générés (approche recommandée)

| Variable | Contenu |
|----------|---------|
| `{{QR_IMAGE}}` | Élément `<image>` SVG complet avec le QR encodé en base64 |
| `{{ACCENT_BLOCK}}` | Décoration d'accent selon le gabarit (barre verticale pour Carte de visite, bandeau pour Badge, vide pour les autres) |
| `{{TEXT_BLOCK}}` | Ensemble des éléments `<text>` positionnés selon le gabarit |

#### Contenu des champs texte (approche avancée)

Si vous souhaitez positionner le texte vous-même, utilisez :

| Variable | Contenu |
|----------|---------|
| `{{F0}}` | Premier champ (Nom, Titre…) |
| `{{F1}}` | Deuxième champ |
| `{{F2}}` | Troisième champ |
| `{{F3}}` | Quatrième champ |
| `{{F4}}` | Cinquième champ |

> Les valeurs sont échappées pour XML (`&amp;`, `&lt;`, etc.).

### Exemple avancé — fond dégradé avec halo

```xml
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg"
     xmlns:xlink="http://www.w3.org/1999/xlink"
     width="{{W}}" height="{{H}}" viewBox="0 0 {{W}} {{H}}">

  <defs>
    <linearGradient id="bg_grad" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%"   stop-color="{{BG}}"/>
      <stop offset="100%" stop-color="{{AC}}" stop-opacity="0.3"/>
    </linearGradient>
    <radialGradient id="qr_glow" cx="50%" cy="50%" r="60%">
      <stop offset="0%"   stop-color="{{AC}}" stop-opacity="0.15"/>
      <stop offset="100%" stop-color="{{AC}}" stop-opacity="0"/>
    </radialGradient>
  </defs>

  <rect width="{{W}}" height="{{H}}" fill="url(#bg_grad)" rx="10"/>

  <rect x="{{QR_X}}" y="{{QR_Y}}" width="{{QR_SZ}}" height="{{QR_SZ}}"
        fill="url(#qr_glow)" rx="8"/>

  {{QR_IMAGE}}

  <rect x="0" y="{{H}}" width="{{W}}" height="6"
        transform="translate(0,-6)" fill="{{AC}}" opacity="0.7"/>

  {{ACCENT_BLOCK}}
  {{TEXT_BLOCK}}

</svg>
```

### Tester votre thème localement

1. Créez votre fichier `mon_theme.svg` avec les variables ci-dessus.
2. Dans RustyQR → **Concepteur de cartes**, cliquez sur **📁 Charger un fichier SVG**.
3. Sélectionnez votre fichier — il devient immédiatement actif pour l'export SVG.

### Partager votre thème via GitHub

Pour que votre thème apparaisse dans la liste **🌐 GitHub** de RustyQR :

1. Forkez le dépôt `rusty-suite/rusty_qr`.
2. Placez votre fichier SVG dans le dossier `templates/`.
3. Ajoutez une entrée dans `templates/index.json` :

```json
{
  "id": "mon_theme",
  "name": "Mon Thème",
  "description": "Description courte de votre thème",
  "file": "mon_theme.svg"
}
```

4. Ouvrez une Pull Request — après fusion, votre thème sera accessible à tous les utilisateurs.

### Gabarits et dimensions de canvas

| Gabarit | Dimensions physiques | Pixels (96 dpi) |
|---------|---------------------|-----------------|
| Carte de visite | 85 × 54 mm | ~321 × 204 px |
| Étiquette QR | 60 × 60 mm | ~227 × 227 px |
| Badge | 90 × 60 mm | ~340 × 227 px |
| Flyer (A6) | 148 × 105 mm | ~560 × 397 px |

Votre thème SVG s'adapte automatiquement grâce aux variables `{{W}}` et `{{H}}`.

---

## Licence

PolyForm Noncommercial — © rusty-suite
