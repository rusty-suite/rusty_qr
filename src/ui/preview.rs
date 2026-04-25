use egui::{Context, Ui};

use crate::app::RustyQrApp;
use crate::style::renderer;
use crate::theme;

pub fn show_preview(app: &mut RustyQrApp, ui: &mut Ui, ctx: &Context) {
    ui.add_space(8.0);
    theme::title(ui, "Aperçu");
    ui.separator();
    ui.add_space(4.0);

    // Profile name indicator
    let profile_name = app.current_profile().name.clone();
    ui.label(egui::RichText::new(format!("Profil: {}", profile_name)).small().weak());
    ui.add_space(4.0);

    // Regenerate texture if dirty
    if app.preview_dirty {
        if let Some(matrix) = &app.qr_matrix {
            let profile = app.current_profile().clone();
            let img = renderer::render(matrix, &profile);
            let color_img = renderer::to_egui_image(&img);
            app.preview_texture = Some(ctx.load_texture(
                "qr_preview",
                color_img,
                egui::TextureOptions::NEAREST,
            ));
        } else {
            app.preview_texture = None;
        }
        app.preview_dirty = false;
    }

    // Display texture or placeholder
    let available = ui.available_size();
    let display_size = available.x.min(available.y).min(380.0);

    if let Some(tex) = &app.preview_texture {
        let size = egui::vec2(display_size, display_size);
        ui.add(egui::Image::new(tex).fit_to_exact_size(size));
    } else if let Some(err) = &app.qr_error.clone() {
        theme::status_err(ui, &format!("✗ {err}"));
    } else {
        let rect = ui.allocate_exact_size(
            egui::vec2(display_size, display_size),
            egui::Sense::hover(),
        ).0;
        ui.painter().rect_filled(rect, 0.0, egui::Color32::from_gray(30));
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "QR apparaîtra ici",
            egui::FontId::proportional(14.0),
            egui::Color32::from_gray(120),
        );
    }

    ui.add_space(8.0);

    // Quick generate button
    if ui.button("⟳ Générer").clicked() {
        app.regenerate_qr();
    }
}
