//! Галерея всей колоды с фильтрами и просмотром карты крупным планом.

use bevy_egui::egui::{self, Frame, Ui};
use orcs_core::CardType;

use super::card::CardFace;
use super::widgets::button;
use super::{View, theme};

const GALLERY_CARD: egui::Vec2 = egui::vec2(240.0, 390.0);

pub fn show(root: &mut Ui, view: &mut View) {
    egui::Panel::top("gallery-header")
        .frame(Frame::NONE.inner_margin(22))
        .show(root, |ui| {
            ui.horizontal(|ui| {
                ui.label(theme::heading(
                    format!("КОЛОДА РАЙОНА  /  {} КАРТ", view.game.cards.len()),
                    30.0,
                    theme::GOLD,
                ));
                if button(ui, "← Назад", true).clicked() {
                    view.go_back();
                }
            });
            ui.horizontal_wrapped(|ui| {
                let filter = view.state.gallery_filter;
                if ui.selectable_label(filter.is_none(), "Все").clicked() {
                    view.state.gallery_filter = None;
                }
                for t in CardType::ALL {
                    if ui.selectable_label(filter == Some(t), t.title()).clicked() {
                        view.state.gallery_filter = Some(t);
                    }
                }
            });
        });
    egui::CentralPanel::default()
        .frame(Frame::NONE.inner_margin(egui::Margin::symmetric(22, 6)))
        .show(root, |ui| {
            egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(15.0, 18.0);
                ui.horizontal_wrapped(|ui| {
                    for c in view.game.cards.iter() {
                        if view.state.gallery_filter.is_some_and(|t| t != c.card_type) {
                            continue;
                        }
                        if CardFace::new(c, view.art, GALLERY_CARD).show(ui).clicked() {
                            view.state.popup = Some(c.id.clone());
                        }
                    }
                });
            });
        });
    card_popup(root.ctx(), view);
}

/// Окно с картой крупным планом.
pub fn card_popup(ctx: &egui::Context, view: &mut View) {
    let Some(id) = view.state.popup.clone() else { return };
    let Some(card) = view.game.cards.get(&id) else {
        view.state.popup = None;
        return;
    };
    let mut open = true;
    egui::Window::new(card.name.as_str())
        .id(egui::Id::new("card-popup"))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .open(&mut open)
        .show(ctx, |ui| {
            CardFace::new(card, view.art, egui::vec2(380.0, 620.0)).show(ui);
        });
    if !open {
        view.state.popup = None;
    }
}
