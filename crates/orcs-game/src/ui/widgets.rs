//! Общие мелкие элементы интерфейса.

use bevy_egui::egui::{self, Color32, Pos2, Response, RichText, Stroke, Ui};

use super::model::Action;
use super::theme;

/// Тёмный стол с еле заметной диагональной сеткой.
pub fn paint_background(ui: &Ui) {
    let rect = ui.max_rect();
    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, theme::BACKGROUND);
    let diagonal = Stroke::new(1.0, Color32::from_rgba_unmultiplied(102, 122, 94, 9));
    let horizontal = Stroke::new(1.0, Color32::from_rgba_unmultiplied(102, 122, 94, 6));
    let mut x = rect.left();
    while x < rect.right() {
        painter.line_segment(
            [Pos2::new(x, rect.top()), Pos2::new(x + 260.0, rect.bottom())],
            diagonal,
        );
        x += 80.0;
    }
    let mut y = rect.top();
    while y < rect.bottom() {
        painter.line_segment([Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)], horizontal);
        y += 80.0;
    }
}

pub fn button(ui: &mut Ui, label: impl Into<RichText>, enabled: bool) -> Response {
    ui.add_enabled(enabled, egui::Button::new(label.into()))
        .on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn small_button(ui: &mut Ui, label: &str, size: f32) -> Response {
    ui.add(egui::Button::new(theme::text(label, size, theme::TEXT)))
        .on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Кнопка модели: недоступная объясняет причину во всплывающей подсказке.
pub fn action_button(ui: &mut Ui, action: &Action) -> bool {
    let response = button(ui, action.label.as_str(), action.enabled);
    let response = if action.enabled {
        response.on_hover_text(&action.hint)
    } else {
        response.on_disabled_hover_text(&action.hint)
    };
    response.clicked()
}

pub fn wrapped(ui: &mut Ui, text: impl Into<String>, size: f32) {
    ui.add(egui::Label::new(theme::text(text, size, theme::TEXT)).wrap());
}

pub fn wrapped_colored(ui: &mut Ui, text: impl Into<String>, size: f32, color: Color32) {
    ui.add(egui::Label::new(theme::text(text, size, color)).wrap());
}

pub fn toast(ui: &mut Ui, toast: &Option<String>) {
    if let Some(message) = toast {
        wrapped_colored(ui, message, 14.0, theme::WARNING);
    }
}
