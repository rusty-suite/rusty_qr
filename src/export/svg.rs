use crate::qr::encoder::QrMatrix;
use crate::style::profile::StyleProfile;

pub fn export(matrix: &QrMatrix, profile: &StyleProfile, path: &str) -> Result<(), String> {
    let svg = build_svg(matrix, profile);
    std::fs::write(path, svg).map_err(|e| format!("SVG: {e}"))
}

pub fn build_svg(matrix: &QrMatrix, profile: &StyleProfile) -> String {
    if matrix.is_empty() { return String::new(); }

    let n = matrix.len();
    let px = profile.module_px as usize;
    let qz = profile.quiet_zone as usize;
    let total = (n + qz * 2) * px;

    let fg = rgb_hex(profile.fg);
    let bg = rgb_hex(profile.bg);

    let mut svg = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" version="1.1"
     width="{total}" height="{total}" viewBox="0 0 {total} {total}">
  <rect width="{total}" height="{total}" fill="{bg}"/>
"#
    );

    for (row_i, row) in matrix.iter().enumerate() {
        for (col_i, &dark) in row.iter().enumerate() {
            if !dark { continue; }
            let x = (col_i + qz) * px;
            let y = (row_i + qz) * px;
            svg.push_str(&format!(
                r#"  <rect x="{x}" y="{y}" width="{px}" height="{px}" fill="{fg}"/>
"#
            ));
        }
    }

    svg.push_str("</svg>\n");
    svg
}

fn rgb_hex(c: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", c[0], c[1], c[2])
}
