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

    pub fn filter_name(&self) -> &'static str {
        match self {
            Self::Png  => "PNG Image",
            Self::Jpg  => "JPEG Image",
            Self::Svg  => "SVG Vector",
            Self::Pdf  => "PDF Document",
            Self::Eps  => "EPS PostScript",
            Self::Docx => "Word Document",
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
