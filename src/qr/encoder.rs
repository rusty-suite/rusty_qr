use qrcode::QrCode;
use crate::qr::types::QrForm;

pub type QrMatrix = Vec<Vec<bool>>;

#[derive(Debug)]
pub struct EncoderError(pub String);

impl std::fmt::Display for EncoderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Encode QR content into a boolean matrix (true = dark module).
pub fn encode(form: &QrForm) -> Result<QrMatrix, EncoderError> {
    if form.use_micro_qr {
        return encode_micro(form);
    }
    encode_standard(form)
}

fn encode_standard(form: &QrForm) -> Result<QrMatrix, EncoderError> {
    let data = form.to_qr_string();
    if data.is_empty() {
        return Err(EncoderError("Données vides".into()));
    }
    let code = QrCode::with_error_correction_level(data.as_bytes(), form.ec_level.to_qrcode())
        .map_err(|e| EncoderError(format!("QR encode: {e}")))?;

    let width = code.width();
    // Extract modules directly as Color values (avoids image crate version conflict)
    let colors = code.to_colors();
    let matrix: QrMatrix = colors
        .chunks(width)
        .map(|row| row.iter().map(|c| *c == qrcode::Color::Dark).collect())
        .collect();

    Ok(matrix)
}

fn encode_micro(form: &QrForm) -> Result<QrMatrix, EncoderError> {
    let data = form.to_qr_string();
    if data.is_empty() {
        return Err(EncoderError("Données vides".into()));
    }
    super::micro_qr::encode(data.as_bytes())
        .map_err(|e| EncoderError(format!("Micro QR: {e}")))
}
