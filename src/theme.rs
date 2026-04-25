use egui::Color32;

pub const GREEN:      Color32 = Color32::from_rgb(80,  210, 80);
pub const YELLOW:     Color32 = Color32::from_rgb(220, 160, 60);
pub const RED:        Color32 = Color32::from_rgb(220, 80,  80);
#[allow(dead_code)]
pub const GREEN2:     Color32 = Color32::from_rgb(80,  200, 80);
#[allow(dead_code)]
pub const RED_BTN:    Color32 = Color32::from_rgb(220, 70,  70);
#[allow(dead_code)]
pub const GRAY:       Color32 = Color32::from_rgb(140, 140, 140);
#[allow(dead_code)]
pub const BODY_SIZE:  f32 = 14.0;
pub const TITLE_SIZE: f32 = 18.0;
#[allow(dead_code)]
pub const MONO_SIZE:  f32 = 13.0;

pub fn title(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).size(TITLE_SIZE).strong());
}

pub fn hint(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).small().weak());
}

pub fn status_ok(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).color(GREEN));
}

pub fn status_err(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).color(RED));
}

pub fn status_warn(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).color(YELLOW));
}
