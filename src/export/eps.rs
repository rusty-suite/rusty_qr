use crate::qr::encoder::QrMatrix;
use crate::style::profile::StyleProfile;

pub fn export(matrix: &QrMatrix, profile: &StyleProfile, path: &str) -> Result<(), String> {
    let eps = build_eps(matrix, profile);
    std::fs::write(path, eps).map_err(|e| format!("EPS: {e}"))
}

fn build_eps(matrix: &QrMatrix, profile: &StyleProfile) -> String {
    if matrix.is_empty() { return String::new(); }

    let n = matrix.len() as i32;
    let px = profile.module_px as i32;
    let qz = profile.quiet_zone as i32;
    let total = (n + qz * 2) * px;

    let fg = (
        profile.fg[0] as f32 / 255.0,
        profile.fg[1] as f32 / 255.0,
        profile.fg[2] as f32 / 255.0,
    );
    let bg = (
        profile.bg[0] as f32 / 255.0,
        profile.bg[1] as f32 / 255.0,
        profile.bg[2] as f32 / 255.0,
    );

    let mut eps = format!(
        "%!PS-Adobe-3.0 EPSF-3.0\n\
         %%BoundingBox: 0 0 {total} {total}\n\
         %%Title: QR Code\n\
         %%Creator: RustyQR\n\
         %%EndComments\n\
         {:.4} {:.4} {:.4} setrgbcolor\n\
         0 0 {total} {total} rectfill\n\
         {:.4} {:.4} {:.4} setrgbcolor\n",
        bg.0, bg.1, bg.2,
        fg.0, fg.1, fg.2,
    );

    // EPS y-axis is bottom-to-top, so flip
    for (row_i, row) in matrix.iter().enumerate() {
        for (col_i, &dark) in row.iter().enumerate() {
            if !dark { continue; }
            let x = (col_i as i32 + qz) * px;
            // Flip Y: EPS origin is bottom-left
            let y = total - (row_i as i32 + qz + 1) * px;
            eps.push_str(&format!("{x} {y} {px} {px} rectfill\n"));
        }
    }

    eps.push_str("%%EOF\n");
    eps
}
