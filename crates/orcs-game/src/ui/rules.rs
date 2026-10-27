//! Экран правил. Текст — `rules/quickstart.md`, встроенный в бинарник.
//! Поддерживается подмножество Markdown: `#`, `##`, абзацы и пункты `- `.

use bevy_egui::egui::{self, Frame, Ui};

use super::widgets::{button, wrapped};
use super::{View, theme};

const QUICKSTART: &str = include_str!("../../../../rules/quickstart.md");

pub fn show(root: &mut Ui, view: &mut View) {
    egui::CentralPanel::default()
        .frame(Frame::NONE.inner_margin(22))
        .show(root, |ui| {
            let label = if view.game.players.is_empty() {
                "← В меню"
            } else {
                "← К столу"
            };
            if button(ui, label, true).clicked() {
                view.go_back();
            }
            egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                ui.set_max_width(1100.0);
                render(ui, QUICKSTART);
            });
        });
}

fn render(ui: &mut Ui, markdown: &str) {
    for line in markdown.lines() {
        if let Some(title) = line.strip_prefix("# ") {
            ui.label(theme::heading(title, 36.0, theme::GOLD));
        } else if let Some(section) = line.strip_prefix("## ") {
            ui.add_space(10.0);
            ui.label(theme::heading(section, 23.0, theme::GOLD_LIGHT));
        } else if let Some(item) = line.strip_prefix("- ") {
            wrapped(ui, format!("•  {item}"), 16.0);
        } else if line.trim().is_empty() {
            ui.add_space(4.0);
        } else {
            wrapped(ui, line, 16.0);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn quickstart_uses_new_title() {
        assert!(super::QUICKSTART.starts_with("# ОРКИ С РАЙОНА"));
        assert!(!super::QUICKSTART.contains("ПУСТОШ"));
    }
}
