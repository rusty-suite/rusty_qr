pub mod docx;
pub mod eps;
pub mod pdf;
pub mod raster;
pub mod svg;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum ExportFormat {
    Png,
    Jpg,
    Svg,
    Pdf,
    Eps,
    Docx,
}

impl ExportFormat {
    pub const ALL: &'static [ExportFormat] = &[
        Self::Png, Self::Jpg, Self::Svg, Self::Pdf, Self::Eps, Self::Docx,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Png  => "PNG",
            Self::Jpg  => "JPEG",
            Self::Svg  => "SVG",
            Self::Pdf  => "PDF",
            Self::Eps  => "EPS",
            Self::Docx => "DOCX (Word)",
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Png  => "png",
            Self::Jpg  => "jpg",
            Self::Svg  => "svg",
            Self::Pdf  => "pdf",
            Self::Eps  => "eps",
            Self::Docx => "docx",
        }
    }

    /// Translation key of the file-dialog filter name (section `export_format`).
    pub fn filter_key(&self) -> &'static str {
        match self {
            Self::Png  => "export_format.png",
            Self::Jpg  => "export_format.jpg",
            Self::Svg  => "export_format.svg",
            Self::Pdf  => "export_format.pdf",
            Self::Eps  => "export_format.eps",
            Self::Docx => "export_format.docx",
        }
    }
}

use crate::qr::encoder::QrMatrix;
use crate::qr::types::EcLevel;
use crate::style::profile::StyleProfile;

pub fn export(
    matrix:  &QrMatrix,
    profile: &StyleProfile,
    ec:      EcLevel,
    format:  ExportFormat,
    path:    &str,
) -> Result<(), String> {
    match format {
        ExportFormat::Png  => raster::export_png(matrix, profile, path),
        ExportFormat::Jpg  => raster::export_jpg(matrix, profile, path),
        ExportFormat::Svg  => svg::export(matrix, profile, path),
        ExportFormat::Pdf  => pdf::export(matrix, profile, ec, path),
        ExportFormat::Eps  => eps::export(matrix, profile, path),
        ExportFormat::Docx => docx::export(matrix, profile, path),
    }
}
