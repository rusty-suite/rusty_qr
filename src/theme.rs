//! Couleurs et styles de texte de Rusty QR, reliés aux tokens de la charte Rusty Suite.

use egui::Color32;

pub const GREEN:      Color32 = rusty_ui::tokens::SUCCESS;
pub const YELLOW:     Color32 = rusty_ui::tokens::WARNING;
pub const RED:        Color32 = rusty_ui::tokens::DANGER_TEXT;
#[allow(dead_code)]
pub const GREEN2:     Color32 = rusty_ui::tokens::SUCCESS;
#[allow(dead_code)]
pub const RED_BTN:    Color32 = rusty_ui::tokens::DANGER;
#[allow(dead_code)]
pub const GRAY:       Color32 = rusty_ui::tokens::TEXT_MUTED;
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
