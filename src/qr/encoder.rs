use qrcode::QrCode;
use crate::qr::types::QrForm;

pub type QrMatrix = Vec<Vec<bool>>;

/// Encoding failure. Messages are translated at display time
/// (see [`EncoderError::message`]) so they follow language changes.
#[derive(Debug, Clone)]
pub enum EncoderError {
    /// Nothing to encode.
    EmptyData,
    /// Data exceeds the Micro QR capacity (length in bytes).
    MicroTooLong(usize),
    /// Error reported by the underlying encoder.
    Backend(String),
}

impl EncoderError {
    pub fn message(&self, lang: &crate::lang::Lang) -> String {
        match self {
            Self::EmptyData       => lang.t("errors.empty_data"),
            Self::MicroTooLong(n) => lang.tf("errors.micro_too_long", &[("n", &n.to_string())]),
            Self::Backend(e)      => lang.tf("errors.encoder", &[("error", e)]),
        }
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
        return Err(EncoderError::EmptyData);
    }
    let code = QrCode::with_error_correction_level(data.as_bytes(), form.ec_level.to_qrcode())
        .map_err(|e| EncoderError::Backend(e.to_string()))?;

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
        return Err(EncoderError::EmptyData);
    }
    super::micro_qr::encode(data.as_bytes())
        .map_err(|e| EncoderError::MicroTooLong(e.0))
}
