//! Génère le logo RustyQR en pixels RGBA — aucune ressource externe requise.
//!
//! Design : fond sombre #1A1A2E, trois coins QR (finder patterns) en vert
//! #50D250, motifs de synchronisation et modules de données décoratifs.

const BG:    [u8; 4] = [26,  26,  46,  255]; // #1A1A2E
const GREEN: [u8; 4] = [80,  210, 80,  255]; // #50D250

fn put(buf: &mut [u8], stride: u32, x: u32, y: u32, w: u32, h: u32, c: [u8; 4]) {
    for dy in 0..h {
        for dx in 0..w {
            let px = x + dx;
            let py = y + dy;
            if px < stride && py < (buf.len() as u32 / stride / 4) {
                let i = (py * stride + px) as usize * 4;
                if i + 3 < buf.len() {
                    buf[i..i + 4].copy_from_slice(&c);
                }
            }
        }
    }
}

/// Finder pattern QR (carré concentrique 7×m) en vert sur fond sombre.
fn finder(buf: &mut [u8], stride: u32, ox: u32, oy: u32, m: u32) {
    put(buf, stride, ox,       oy,       7 * m, 7 * m, GREEN);
    put(buf, stride, ox + m,   oy + m,   5 * m, 5 * m, BG);
    put(buf, stride, ox + 2*m, oy + 2*m, 3 * m, 3 * m, GREEN);
}

/// Génère le logo en RGBA brut pour une taille carrée donnée.
pub fn generate_rgba(size: u32) -> Vec<u8> {
    let mut buf = vec![0u8; (size * size * 4) as usize];

    // Fond
    for px in buf.chunks_mut(4) {
        px.copy_from_slice(&BG);
    }

    // Taille d'un module (au moins 1 px). On vise ~7 modules = ~30% de la taille.
    let m = (size / 22).max(1);
    let pad = m; // marge intérieure

    // ── Trois coins QR ──────────────────────────────────────────────────────
    finder(&mut buf, size, pad,                 pad,                 m); // haut-gauche
    finder(&mut buf, size, size - 8 * m,        pad,                 m); // haut-droit
    finder(&mut buf, size, pad,                 size - 8 * m,        m); // bas-gauche

    // ── Timing patterns (alternés entre les coins) ───────────────────────
    let timing_start = 8 * m;
    let timing_end   = size - 8 * m;
    let mut t = timing_start;
    while t < timing_end {
        let dark = (t / m) % 2 == 0;
        if dark {
            put(&mut buf, size, t,     6 * m, m, m, GREEN); // horizontal
            put(&mut buf, size, 6 * m, t,     m, m, GREEN); // vertical
        }
        t += m;
    }

    // ── Modules de données décoratifs (bas-droite) ───────────────────────
    // Motif fixe qui ressemble à des données encodées
    const DATA: &[(u32, u32)] = &[
        (0,0),(2,0),(4,0),(6,0),
        (1,1),(3,1),(5,1),
        (0,2),(2,2),(4,2),(6,2),
        (1,3),(5,3),
        (0,4),(2,4),(4,4),(6,4),
        (3,5),(5,5),
        (0,6),(1,6),(4,6),(6,6),
    ];
    let dox = timing_end + m;
    let doy = timing_end + m;
    for &(dx, dy) in DATA {
        let x = dox + dx * m;
        let y = doy + dy * m;
        if x + m <= size && y + m <= size {
            put(&mut buf, size, x, y, m, m, GREEN);
        }
    }

    // ── Pixel d'alignement (module sombre spécial) ───────────────────────
    // Petit carré vert centré entre les finders bas-gauche et bas-droit
    let align_x = size / 2 - m;
    let align_y = size - 4 * m;
    put(&mut buf, size, align_x,     align_y,     5 * m, 5 * m, GREEN);
    put(&mut buf, size, align_x + m, align_y + m, 3 * m, 3 * m, BG);
    put(&mut buf, size, align_x+2*m, align_y+2*m, m,     m,     GREEN);

    buf
}

/// Données pour l'icône de fenêtre (32×32).
pub fn icon_data() -> egui::IconData {
    let size = 32u32;
    egui::IconData {
        rgba: generate_rgba(size),
        width: size,
        height: size,
    }
}
