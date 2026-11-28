# RustyQR

🇬🇧 **English** · 🇫🇷 [Français](README.fr.md)

Multi-format QR code generator with style profiles, a library and vector export.

## Features

- **11 QR types**: URL, Text, Wi-Fi, SMS, Phone, Email, vCard, MeCard, Geolocation, GS1, 2D-Doc
- **Micro QR M1–M4** implemented natively (no external dependency)
- **Style profiles**: custom colours, overlaid logo with free positioning
- **Library**: save and reload named configurations
- **Card designer**: 4 layouts (Business card, QR label, Badge, A6 flyer)
- **SVG themes**: 4 built-in themes + local files + GitHub catalogue
- **Exports**: PNG, JPEG, SVG, PDF, EPS, DOCX

---

## Guide: Creating a custom SVG theme

SVG themes customise the look of the cards exported from the **Card designer**.
A theme is an `.svg` file containing `{{VARIABLE}}` markers that RustyQR replaces on export.

### Minimal theme structure

```xml
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg"
     xmlns:xlink="http://www.w3.org/1999/xlink"
     width="{{W}}" height="{{H}}" viewBox="0 0 {{W}} {{H}}">

  <!-- Your custom background -->
  <rect width="{{W}}" height="{{H}}" fill="{{BG}}" rx="8"/>

  <!-- QR code (block generated automatically) -->
  {{QR_IMAGE}}

  <!-- Accent elements depending on the layout (bar, banner…) -->
  {{ACCENT_BLOCK}}

  <!-- Text fields depending on the layout -->
  {{TEXT_BLOCK}}

</svg>
```

### Available variables

#### Dimensions and colours

| Variable | Content | Example |
|----------|---------|---------|
| `{{W}}` | Canvas width in pixels | `321` |
| `{{H}}` | Canvas height in pixels | `204` |
| `{{BG}}` | Background colour (#RRGGBB) | `#FFFFFF` |
| `{{FG}}` | Text colour (#RRGGBB) | `#1E1E1E` |
| `{{AC}}` | Accent colour (#RRGGBB) | `#3264C8` |

#### QR code position

| Variable | Content |
|----------|---------|
| `{{QR_X}}` | X position of the QR code in pixels |
| `{{QR_Y}}` | Y position of the QR code in pixels |
| `{{QR_SZ}}` | Size (width = height) of the QR code in pixels |

#### Pre-generated blocks (recommended approach)

| Variable | Content |
|----------|---------|
| `{{QR_IMAGE}}` | Complete SVG `<image>` element with the QR code encoded in base64 |
| `{{ACCENT_BLOCK}}` | Accent decoration depending on the layout (vertical bar for Business card, banner for Badge, empty for the others) |
| `{{TEXT_BLOCK}}` | All `<text>` elements positioned according to the layout |

#### Text field contents (advanced approach)

To position the text yourself, use:

| Variable | Content |
|----------|---------|
| `{{F0}}` | First field (Name, Title…) |
| `{{F1}}` | Second field |
| `{{F2}}` | Third field |
| `{{F3}}` | Fourth field |
| `{{F4}}` | Fifth field |

> Values are XML-escaped (`&amp;`, `&lt;`, etc.).

### Advanced example: gradient background with glow

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

### Testing your theme locally

1. Create your `my_theme.svg` file using the variables above.
2. In RustyQR → **Card designer**, click **📁 SVG file**.
3. Select your file: it becomes active immediately for SVG export.

### Layouts and canvas dimensions

| Layout | Physical size | Pixels (96 dpi) |
|--------|---------------|-----------------|
| Business card | 85 × 54 mm | ~321 × 204 px |
| QR label | 60 × 60 mm | ~227 × 227 px |
| Badge | 90 × 60 mm | ~340 × 227 px |
| Flyer (A6) | 148 × 105 mm | ~560 × 397 px |

Your SVG theme adapts automatically thanks to the `{{W}}` and `{{H}}` variables.

---

## License

Copyright © rusty-suite.

This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version (`GPL-3.0-or-later`).
See [LICENSE](LICENSE) for the full text.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY.
