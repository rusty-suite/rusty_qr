// Embed the app icon into the Windows .exe at compile time.
// On non-Windows targets this is a no-op.

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let ico_path = std::path::PathBuf::from(&out_dir).join("rusty_qr.ico");
        write_ico(&ico_path);
        winresource::WindowsResource::new()
            .set_icon(ico_path.to_str().unwrap())
            .compile()
            .expect("winresource: failed to embed icon");
    }
}

// ─── ICO writer ──────────────────────────────────────────────────────────────

fn write_ico(path: &std::path::Path) {
    // Embed 4 sizes for crisp rendering at every Windows DPI level
    let sizes: &[u32] = &[16, 32, 48, 256];
    let pngs: Vec<Vec<u8>> = sizes
        .iter()
        .map(|&s| encode_png(s, &generate_logo(s)))
        .collect();

    // ICO header  (6 bytes)  +  directory entries  (16 bytes × N)
    let dir_offset = 6u32 + 16 * sizes.len() as u32;
    let mut ico: Vec<u8> = Vec::new();

    // ICONDIR
    ico.extend_from_slice(&[0x00, 0x00]);              // reserved
    ico.extend_from_slice(&[0x01, 0x00]);              // type = 1 (icon)
    ico.extend_from_slice(&(sizes.len() as u16).to_le_bytes());

    // ICONDIRENTRY for each size
    let mut img_offset = dir_offset;
    for (i, &size) in sizes.iter().enumerate() {
        let w = if size >= 256 { 0u8 } else { size as u8 }; // 0 encodes 256
        let h = w;
        let png_len = pngs[i].len() as u32;
        ico.push(w);
        ico.push(h);
        ico.push(0); // color count (0 = truecolor)
        ico.push(0); // reserved
        ico.extend_from_slice(&[0x01, 0x00]); // planes
        ico.extend_from_slice(&[0x20, 0x00]); // bits per pixel = 32
        ico.extend_from_slice(&png_len.to_le_bytes());
        ico.extend_from_slice(&img_offset.to_le_bytes());
        img_offset += png_len;
    }

    // PNG payloads
    for png in &pngs {
        ico.extend_from_slice(png);
    }

    std::fs::write(path, ico).expect("build.rs: cannot write icon.ico");
}

fn encode_png(size: u32, rgba: &[u8]) -> Vec<u8> {
    use image::codecs::png::PngEncoder;
    use image::{ColorType, ImageEncoder};
    let mut buf = Vec::new();
    PngEncoder::new(&mut buf)
        .write_image(rgba, size, size, ColorType::Rgba8)
        .expect("build.rs: PNG encode failed");
    buf
}

// ─── Logo pixel generator (mirrors src/logo.rs) ──────────────────────────────

const BG:    [u8; 4] = [26, 26, 46, 255]; // #1A1A2E
const GREEN: [u8; 4] = [80, 210, 80, 255]; // #50D250

fn put(buf: &mut [u8], stride: u32, x: u32, y: u32, w: u32, h: u32, c: [u8; 4]) {
    let rows = buf.len() as u32 / stride / 4;
    for dy in 0..h {
        for dx in 0..w {
            let px = x + dx;
            let py = y + dy;
            if px < stride && py < rows {
                let i = (py * stride + px) as usize * 4;
                if i + 3 < buf.len() {
                    buf[i..i + 4].copy_from_slice(&c);
                }
            }
        }
    }
}

fn finder(buf: &mut [u8], stride: u32, ox: u32, oy: u32, m: u32) {
    put(buf, stride, ox,         oy,         7 * m, 7 * m, GREEN);
    put(buf, stride, ox + m,     oy + m,     5 * m, 5 * m, BG);
    put(buf, stride, ox + 2 * m, oy + 2 * m, 3 * m, 3 * m, GREEN);
}

fn generate_logo(size: u32) -> Vec<u8> {
    let mut buf = vec![0u8; (size * size * 4) as usize];
    for px in buf.chunks_mut(4) {
        px.copy_from_slice(&BG);
    }

    let m   = (size / 22).max(1);
    let pad = m;

    finder(&mut buf, size, pad,              pad,              m); // top-left
    finder(&mut buf, size, size - 8 * m,     pad,              m); // top-right
    finder(&mut buf, size, pad,              size - 8 * m,     m); // bottom-left

    // Timing patterns
    let ts = 8 * m;
    let te = size - 8 * m;
    let mut t = ts;
    while t < te {
        if (t / m) % 2 == 0 {
            put(&mut buf, size, t,     6 * m, m, m, GREEN);
            put(&mut buf, size, 6 * m, t,     m, m, GREEN);
        }
        t += m;
    }

    // Decorative data modules (bottom-right)
    const DATA: &[(u32, u32)] = &[
        (0,0),(2,0),(4,0),(6,0),
        (1,1),(3,1),(5,1),
        (0,2),(2,2),(4,2),(6,2),
        (1,3),(5,3),
        (0,4),(2,4),(4,4),(6,4),
        (3,5),(5,5),
        (0,6),(1,6),(4,6),(6,6),
    ];
    let dox = te + m;
    let doy = te + m;
    for &(dx, dy) in DATA {
        let x = dox + dx * m;
        let y = doy + dy * m;
        if x + m <= size && y + m <= size {
            put(&mut buf, size, x, y, m, m, GREEN);
        }
    }

    // Alignment pattern (centred bottom)
    let ax = size / 2 - m;
    let ay = size - 4 * m;
    put(&mut buf, size, ax,         ay,         5 * m, 5 * m, GREEN);
    put(&mut buf, size, ax + m,     ay + m,     3 * m, 3 * m, BG);
    put(&mut buf, size, ax + 2 * m, ay + 2 * m, m,     m,     GREEN);

    buf
}
