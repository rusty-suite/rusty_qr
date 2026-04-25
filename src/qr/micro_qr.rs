//! Micro QR Code encoder — M1 à M4, modes numérique, alphanumérique, octet.
//! Basé sur ISO/IEC 18004:2015.

use super::encoder::QrMatrix;

#[derive(Debug)]
pub struct MicroQrError(pub String);

impl std::fmt::Display for MicroQrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ─── GF(256) tables ──────────────────────────────────────────────────────────
// Primitive polynomial: x^8 + x^4 + x^3 + x^2 + 1 (0x11D)

static mut GF_EXP: [u8; 512] = [0u8; 512];
static mut GF_LOG: [u8; 256] = [0u8; 256];
static mut GF_INIT: bool = false;

fn init_gf() {
    unsafe {
        if GF_INIT { return; }
        let mut x: u16 = 1;
        for i in 0..255usize {
            GF_EXP[i] = x as u8;
            GF_LOG[x as usize] = i as u8;
            x <<= 1;
            if x & 0x100 != 0 { x ^= 0x11D; }
        }
        for i in 255..512usize { GF_EXP[i] = GF_EXP[i - 255]; }
        GF_INIT = true;
    }
}

fn gf_mul(a: u8, b: u8) -> u8 {
    if a == 0 || b == 0 { return 0; }
    unsafe { GF_EXP[(GF_LOG[a as usize] as usize + GF_LOG[b as usize] as usize) % 255] }
}

fn gf_exp(n: usize) -> u8 {
    unsafe { GF_EXP[n % 255] }
}

// ─── Reed-Solomon ────────────────────────────────────────────────────────────

fn rs_generator(degree: usize) -> Vec<u8> {
    let mut gen = vec![1u8];
    for i in 0..degree {
        let term = vec![1u8, gf_exp(i)];
        let mut res = vec![0u8; gen.len() + 1];
        for (j, &g) in gen.iter().enumerate() {
            res[j] ^= g;
            res[j + 1] ^= gf_mul(g, term[1]);
        }
        gen = res;
    }
    gen
}

fn rs_ec_codewords(data: &[u8], num_ec: usize) -> Vec<u8> {
    init_gf();
    let gen = rs_generator(num_ec);
    let mut buf = data.to_vec();
    buf.extend(vec![0u8; num_ec]);
    for i in 0..data.len() {
        let coef = buf[i];
        if coef != 0 {
            for j in 0..num_ec {
                buf[i + 1 + j] ^= gf_mul(gen[j + 1], coef);
            }
        }
    }
    buf[data.len()..].to_vec()
}

// ─── Capacity tables ─────────────────────────────────────────────────────────

// (version, ec_index) -> (data_codewords, ec_codewords, total_codewords)
// ec_index: 0=L,1=M,2=Q  (M1 has no EC level, uses 0)
// version: 1=M1, 2=M2, 3=M3, 4=M4
const CAPACITY: &[((u8, u8), (usize, usize, usize))] = &[
    ((1, 0), (3,  2,  5)),  // M1
    ((2, 0), (5,  5, 10)),  // M2-L
    ((2, 1), (4,  6, 10)),  // M2-M
    ((3, 0), (11, 6, 17)),  // M3-L
    ((3, 1), (9,  8, 17)),  // M3-M
    ((4, 0), (16, 8, 24)),  // M4-L
    ((4, 1), (14, 10,24)),  // M4-M
    ((4, 2), (10, 14,24)),  // M4-Q
];

// Byte capacity per version/ec
const BYTE_CAPACITY: &[(u8, u8, usize)] = &[
    // (version, ec_index, max_bytes)
    (1, 0,  0),  // M1: numeric only
    (2, 0,  0),  // M2: no byte mode
    (2, 1,  0),  // M2-M: no byte mode
    (3, 0,  9),  // M3-L: 9 bytes
    (3, 1,  7),  // M3-M: 7 bytes
    (4, 0, 15),  // M4-L: 15 bytes
    (4, 1, 11),  // M4-M: 11 bytes
    (4, 2,  9),  // M4-Q: 9 bytes
];

/// Find smallest version that fits data in byte mode (M3-L minimum).
fn select_version(data_len: usize) -> Result<(u8, u8), MicroQrError> {
    for &(ver, ec, cap) in BYTE_CAPACITY {
        if cap >= data_len { return Ok((ver, ec)); }
    }
    Err(MicroQrError(format!(
        "Données trop longues pour Micro QR ({} octets, max 15). Utilisez le QR standard.",
        data_len
    )))
}

fn cap(ver: u8, ec: u8) -> (usize, usize, usize) {
    CAPACITY.iter().find(|&&((v, e), _)| v == ver && e == ec)
        .map(|&(_, c)| c)
        .unwrap_or((0, 0, 0))
}

// ─── Encoding ────────────────────────────────────────────────────────────────

/// Encode data bytes into a bit string.
fn encode_byte_mode(data: &[u8], ver: u8) -> Vec<u8> {
    let mut bits: Vec<u8> = Vec::new();

    // Mode indicator: M3=10 bits (bit count depends on version)
    // M3: mode indicator = 11 (2 bits)
    // M4: mode indicator = 100 (3 bits)
    match ver {
        3 => push_bits(&mut bits, 0b11, 2),
        4 => push_bits(&mut bits, 0b100, 3),
        _ => {}
    }

    // Character count indicator
    let cc_bits = match ver { 3 => 4, 4 => 5, _ => 4 };
    push_bits(&mut bits, data.len() as u64, cc_bits);

    // Data
    for &byte in data {
        push_bits(&mut bits, byte as u64, 8);
    }

    // Terminator (3 bits for M3, 5 bits for M4)
    let term = match ver { 3 => 3, 4 => 5, _ => 3 };
    for _ in 0..term { bits.push(0); }

    bits
}

fn push_bits(bits: &mut Vec<u8>, value: u64, count: usize) {
    for i in (0..count).rev() {
        bits.push(((value >> i) & 1) as u8);
    }
}

fn bits_to_codewords(bits: &[u8], total_codewords: usize) -> Vec<u8> {
    let mut cw: Vec<u8> = Vec::with_capacity(total_codewords);
    let padded: Vec<u8> = bits.iter().cloned()
        .chain(std::iter::repeat(0))
        .take(total_codewords * 8)
        .collect();
    for chunk in padded.chunks(8) {
        let mut b = 0u8;
        for (i, &bit) in chunk.iter().enumerate() {
            b |= bit << (7 - i);
        }
        cw.push(b);
    }
    // Pad remaining with alternating 0xEC / 0x11
    while cw.len() < total_codewords {
        cw.push(if cw.len() % 2 == 0 { 0xEC } else { 0x11 });
    }
    cw
}

// ─── Matrix placement ────────────────────────────────────────────────────────

fn symbol_size(ver: u8) -> usize {
    (2 * ver as usize) + 9
}

struct Matrix {
    size: usize,
    data: Vec<Vec<Option<bool>>>, // None = not yet placed
}

impl Matrix {
    fn new(size: usize) -> Self {
        Self { size, data: vec![vec![None; size]; size] }
    }

    fn set(&mut self, row: usize, col: usize, val: bool) {
        self.data[row][col] = Some(val);
    }

    fn is_free(&self, row: usize, col: usize) -> bool {
        self.data[row][col].is_none()
    }

    fn get(&self, row: usize, col: usize) -> bool {
        self.data[row][col].unwrap_or(false)
    }
}

fn place_finder(m: &mut Matrix) {
    // 7×7 finder pattern (top-left)
    let pat = [
        [true, true, true, true, true, true, true],
        [true, false,false,false,false,false,true],
        [true, false,true, true, true, false,true],
        [true, false,true, true, true, false,true],
        [true, false,true, true, true, false,true],
        [true, false,false,false,false,false,true],
        [true, true, true, true, true, true, true],
    ];
    for (r, row) in pat.iter().enumerate() {
        for (c, &v) in row.iter().enumerate() {
            m.set(r, c, v);
        }
    }
    // Separator: row 7 cols 0..=7 and col 7 rows 0..=7
    for i in 0..8 { m.set(7, i, false); m.set(i, 7, false); }
}

fn place_timing(m: &mut Matrix) {
    // Horizontal timing: row 0, cols 8..size
    for c in 8..m.size { m.set(0, c, c % 2 == 0); }
    // Vertical timing: col 0, rows 8..size
    for r in 8..m.size { m.set(r, 0, r % 2 == 0); }
}

fn place_format_info(m: &mut Matrix, ver: u8, mask: u8, ec: u8) {
    let fmt_data = format_data(ver, mask, ec);
    let fmt_bits = bch_format(fmt_data);
    let masked = fmt_bits ^ 0b100010001000101u16;

    // Row 8, cols 1..=8 (bits 0..7 of format info → LSB first)
    for i in 0..8usize {
        let bit = (masked >> i) & 1 == 1;
        m.set(8, i + 1, bit);
    }
    // Col 8, rows 1..=8 (bits 8..14 of format info)
    for i in 0..7usize {
        let bit = (masked >> (8 + i)) & 1 == 1;
        m.set(i + 1, 8, bit);
    }
    // Dark module at (8,8)
    m.set(8, 8, true);
}

/// 5-bit format data for Micro QR.
/// Bits 4-3: version symbol (M1=0,M2=1,M3=2,M4=3)
/// Bits 2-0: combined mask+ec per version
fn format_data(ver: u8, mask: u8, ec: u8) -> u16 {
    let sym = (ver - 1) as u16; // 0-3
    let combo = match ver {
        1 => mask as u16,                    // M1: bits 2-0 = mask(0-3)
        2 | 3 => (ec as u16 * 4) + mask as u16, // 8 combos
        4 => match ec {
            0 => mask as u16,           // L: mask 0-3 → 0-3
            1 => 4 + (mask & 1) as u16, // M: mask 0-1 → 4-5
            _ => 6 + (mask & 1) as u16, // Q: mask 0-1 → 6-7
        },
        _ => 0,
    };
    (sym << 3) | combo
}

/// BCH(15,5) error correction for format information.
/// Generator: x^10 + x^8 + x^5 + x^4 + x^2 + x + 1 = 0b10100110111
fn bch_format(data5: u16) -> u16 {
    let gen: u16 = 0b10100110111;
    let mut val = data5 << 10;
    for i in (0..=4).rev() {
        if (val >> (i + 10)) & 1 == 1 {
            val ^= gen << i;
        }
    }
    (data5 << 10) | (val & 0x3FF)
}

/// Place data + EC codewords using Micro QR zigzag pattern.
fn place_data(m: &mut Matrix, codewords: &[u8]) {
    let n = m.size;
    let mut bits: Vec<bool> = Vec::with_capacity(codewords.len() * 8);
    for &cw in codewords {
        for i in (0..8).rev() { bits.push((cw >> i) & 1 == 1); }
    }

    let mut bit_idx = 0;
    // Columns right-to-left in pairs, starting from rightmost
    let mut col = n as isize - 1;
    while col >= 1 {
        let cols = [col as usize, (col - 1) as usize];
        let going_up = ((n as isize - 1 - col) / 2) % 2 == 0;

        let rows: Vec<usize> = if going_up {
            (0..n).rev().collect()
        } else {
            (0..n).collect()
        };

        for row in rows {
            for &c in &cols {
                if m.is_free(row, c) {
                    if bit_idx < bits.len() {
                        m.set(row, c, bits[bit_idx]);
                        bit_idx += 1;
                    } else {
                        m.set(row, c, false);
                    }
                }
            }
        }
        col -= 2;
    }
}

/// Apply mask pattern to data modules only.
fn apply_mask(m: &mut Matrix, mask: u8) {
    let n = m.size;
    for r in 0..n {
        for c in 0..n {
            // Skip function modules (finder, separator, timing, format)
            let is_timing = r == 0 || c == 0;
            let in_finder = r < 8 && c < 8;
            let in_format = (r == 8 && c >= 1 && c <= 8) || (c == 8 && r >= 1 && r <= 8);
            if is_timing || in_finder || in_format { continue; }

            let flip = match mask {
                0 => (r + c) % 2 == 0,
                1 => r % 2 == 0,
                2 => c % 3 == 0,
                3 => (r + c) % 3 == 0,
                _ => false,
            };
            if flip {
                let cur = m.get(r, c);
                m.set(r, c, !cur);
            }
        }
    }
}

/// Evaluate penalty score for mask selection.
fn penalty(m: &Matrix) -> i32 {
    let n = m.size;
    // Rule 1: sum of dark modules in right column and bottom row
    let right_col: i32 = (0..n).map(|r| if m.get(r, n - 1) { 1 } else { 0 }).sum();
    let bottom_row: i32 = (0..n).map(|c| if m.get(n - 1, c) { 1 } else { 0 }).sum();
    let score = if right_col <= bottom_row {
        right_col * 16 + bottom_row
    } else {
        bottom_row * 16 + right_col
    };
    score
}

// ─── Public entry point ──────────────────────────────────────────────────────

pub fn encode(data: &[u8]) -> Result<QrMatrix, MicroQrError> {
    init_gf();

    let (ver, ec_idx) = select_version(data.len())?;
    let (data_cw_count, ec_cw_count, _total) = cap(ver, ec_idx);

    // Encode to bits → codewords
    let bits = encode_byte_mode(data, ver);
    let data_codewords = bits_to_codewords(&bits, data_cw_count);
    let ec_codewords = rs_ec_codewords(&data_codewords, ec_cw_count);
    let all_codewords: Vec<u8> = data_codewords.into_iter().chain(ec_codewords).collect();

    let size = symbol_size(ver);

    // Try 4 masks, pick best penalty
    let mut best_matrix: Option<Matrix> = None;
    let mut best_score = i32::MAX;
    let mut best_mask = 0u8;

    for mask in 0..4u8 {
        let mut m = Matrix::new(size);
        place_finder(&mut m);
        place_timing(&mut m);
        // Reserve format info area before placing data
        for i in 1..=8 { m.set(8, i, false); m.set(i, 8, false); }
        m.set(8, 8, true);
        place_data(&mut m, &all_codewords);
        apply_mask(&mut m, mask);
        let s = penalty(&m);
        if s < best_score {
            best_score = s;
            best_mask = mask;
            best_matrix = Some(m);
        }
    }

    let mut m = best_matrix.unwrap();
    // Overwrite format info with correct values
    place_format_info(&mut m, ver, best_mask, ec_idx);

    let matrix: QrMatrix = m.data.into_iter()
        .map(|row| row.into_iter().map(|cell| cell.unwrap_or(false)).collect())
        .collect();

    Ok(matrix)
}
