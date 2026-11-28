use egui::{Context, Ui};

use crate::app::RustyQrApp;
use crate::style::renderer;
use crate::theme;

pub fn show_preview(app: &mut RustyQrApp, ui: &mut Ui, ctx: &Context) {
    ui.add_space(8.0);
    theme::title(ui, &app.lang.t("preview.title"));
    ui.separator();
    ui.add_space(4.0);

    // Profile name indicator
    let profile_name = app.current_profile().name.clone();
    ui.label(egui::RichText::new(app.lang.tf("preview.profile", &[("name", &profile_name)])).small().weak());
    ui.add_space(4.0);

    // Regenerate texture if dirty
    if app.preview_dirty {
        if let Some(matrix) = &app.qr_matrix {
            let profile = app.current_profile().clone();
            let ec      = app.form.ec_level;
            let img = renderer::render_ec(matrix, &profile, ec);
            let color_img = renderer::to_egui_image(&img);
            app.preview_texture = Some(ctx.load_texture(
                "qr_preview",
                color_img,
                egui::TextureOptions::NEAREST,
            ));
            // Keep a copy so the card designer can embed the same render
            // without re-running the (potentially heavy) QR rasterizer.
            app.qr_rendered_image = Some(img);
        } else {
            app.preview_texture = None;
            app.qr_rendered_image = None;
        }
        app.preview_dirty = false;
    }

    // Display texture or placeholder
    let available = ui.available_size();
    let display_size = available.x.min(available.y).min(380.0);

    if let Some(tex) = &app.preview_texture {
        let size = egui::vec2(display_size, display_size);
        ui.add(egui::Image::new(tex).fit_to_exact_size(size));
    } else if let Some(err) = &app.qr_error {
        theme::status_err(ui, &format!("\u{2717} {}", err.message(&app.lang)));
    } else {
        let rect = ui.allocate_exact_size(
            egui::vec2(display_size, display_size),
            egui::Sense::hover(),
        ).0;
        ui.painter().rect_filled(rect, 0.0, rusty_ui::tokens::BG_INPUT);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            app.lang.t("preview.placeholder"),
            egui::FontId::proportional(14.0),
            rusty_ui::tokens::TEXT_MUTED,
        );
    }

    ui.add_space(8.0);

    // Quick generate button
    if ui.button(format!("\u{1F504} {}", app.lang.t("preview.generate"))).clicked() {
        app.regenerate_qr();
    }
}
