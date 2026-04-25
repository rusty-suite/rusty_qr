//! DOCX export — génère un fichier Word minimal contenant le QR code en image.
//! Format DOCX = archive ZIP avec XML et ressources media.

use std::io::Write;
use zip::ZipWriter;
use zip::write::FileOptions;

use crate::qr::encoder::QrMatrix;
use crate::style::{profile::StyleProfile, renderer};

pub fn export(matrix: &QrMatrix, profile: &StyleProfile, path: &str) -> Result<(), String> {
    let img = renderer::render(matrix, profile);

    // Encode image to PNG bytes
    let mut png_bytes: Vec<u8> = Vec::new();
    {
        use image::ImageEncoder;
        let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
        encoder
            .write_image(img.as_raw(), img.width(), img.height(), image::ColorType::Rgba8)
            .map_err(|e| format!("PNG encode: {e}"))?;
    }

    let w_px = img.width();
    let h_px = img.height();
    // EMU (English Metric Units): 1 cm = 360000 EMU, 1 inch = 914400 EMU
    // Assume 96 dpi: 1 px = 914400/96 EMU = 9525 EMU
    let emu_per_px = 9525u32;
    let cx = w_px * emu_per_px;
    let cy = h_px * emu_per_px;

    let file = std::fs::File::create(path).map_err(|e| format!("DOCX create: {e}"))?;
    let mut zip = ZipWriter::new(file);
    let opts: FileOptions<()> = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // [Content_Types].xml
    zip.start_file("[Content_Types].xml", opts).map_err(|e| e.to_string())?;
    zip.write_all(CONTENT_TYPES.as_bytes()).map_err(|e| e.to_string())?;

    // _rels/.rels
    zip.start_file("_rels/.rels", opts).map_err(|e| e.to_string())?;
    zip.write_all(ROOT_RELS.as_bytes()).map_err(|e| e.to_string())?;

    // word/_rels/document.xml.rels
    zip.start_file("word/_rels/document.xml.rels", opts).map_err(|e| e.to_string())?;
    zip.write_all(DOC_RELS.as_bytes()).map_err(|e| e.to_string())?;

    // word/media/image1.png
    zip.start_file("word/media/image1.png", opts).map_err(|e| e.to_string())?;
    zip.write_all(&png_bytes).map_err(|e| e.to_string())?;

    // word/document.xml
    let doc_xml = build_document_xml(cx, cy);
    zip.start_file("word/document.xml", opts).map_err(|e| e.to_string())?;
    zip.write_all(doc_xml.as_bytes()).map_err(|e| e.to_string())?;

    zip.finish().map_err(|e| format!("DOCX finish: {e}"))?;
    Ok(())
}

fn build_document_xml(cx: u32, cy: u32) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:wpc="http://schemas.microsoft.com/office/word/2010/wordprocessingCanvas"
  xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
  xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
  xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
  xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture"
  xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<w:body>
  <w:p>
    <w:r>
      <w:drawing>
        <wp:inline>
          <wp:extent cx="{cx}" cy="{cy}"/>
          <wp:docPr id="1" name="QR Code"/>
          <a:graphic>
            <a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture">
              <pic:pic>
                <pic:nvPicPr>
                  <pic:cNvPr id="0" name="QR Code"/>
                  <pic:cNvPicPr/>
                </pic:nvPicPr>
                <pic:blipFill>
                  <a:blip r:embed="rId1"/>
                  <a:stretch><a:fillRect/></a:stretch>
                </pic:blipFill>
                <pic:spPr>
                  <a:xfrm><a:off x="0" y="0"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm>
                  <a:prstGeom prst="rect"><a:avLst/></a:prstGeom>
                </pic:spPr>
              </pic:pic>
            </a:graphicData>
          </a:graphic>
        </wp:inline>
      </w:drawing>
    </w:r>
  </w:p>
  <w:sectPr/>
</w:body>
</w:document>"#
    )
}

const CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Default Extension="png" ContentType="image/png"/>
  <Override PartName="/word/document.xml"
    ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#;

const ROOT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument"
    Target="word/document.xml"/>
</Relationships>"#;

const DOC_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image"
    Target="media/image1.png"/>
</Relationships>"#;
