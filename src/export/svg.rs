use crate::qr::encoder::QrMatrix;
use crate::style::profile::StyleProfile;

pub fn export(matrix: &QrMatrix, profile: &StyleProfile, path: &str) -> Result<(), String> {
    let svg = build_svg(matrix, profile);
    std::fs::write(path, svg).map_err(|e| format!("SVG: {e}"))
}

pub fn build_svg(matrix: &QrMatrix, profile: &StyleProfile) -> String {
    if matrix.is_empty() { return String::new(); }

    let n  = matrix.len();
    let px = profile.module_px as usize;
    let qz = profile.quiet_zone as usize;
    let total = (n + qz * 2) * px;

    let fg = rgb_hex(profile.fg);
    let bg = rgb_hex(profile.bg);

    // Pre-allocate conservatively (merging cuts rect count by ~60 %)
    let mut svg = String::with_capacity(512 + n * px * 40);
    svg.push_str(&format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <svg xmlns=\"http://www.w3.org/2000/svg\" version=\"1.1\"\n\
              width=\"{total}\" height=\"{total}\" viewBox=\"0 0 {total} {total}\">\n\
           <rect width=\"{total}\" height=\"{total}\" fill=\"{bg}\"/>\n"
    ));

    // Merge consecutive dark modules per row into a single wider rect.
    // This reduces element count from O(dark_modules) to O(dark_runs) —
    // typically 40–60 % fewer elements for real QR patterns.
    for (row_i, row) in matrix.iter().enumerate() {
        let y = (row_i + qz) * px;
        let mut col = 0;
        while col < n {
            if row[col] {
                let start = col;
                while col < n && row[col] { col += 1; }
                let x = (start + qz) * px;
                let w = (col - start) * px;
                svg.push_str(&format!(
                    "  <rect x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{px}\" fill=\"{fg}\"/>\n"
                ));
            } else {
                col += 1;
            }
        }
    }

    svg.push_str("</svg>\n");
    svg
}

fn rgb_hex(c: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", c[0], c[1], c[2])
}
