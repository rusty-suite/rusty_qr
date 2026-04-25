mod app;
mod card;
mod export;
mod history;
mod logo;
mod qr;
mod style;
mod theme;
mod ui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([900.0, 600.0])
            .with_title("RustyQR — Générateur de codes QR")
            .with_icon(logo::icon_data()),
        ..Default::default()
    };
    eframe::run_native(
        "RustyQR",
        options,
        Box::new(|cc| Ok(Box::new(app::RustyQrApp::new(cc)))),
    )
}
