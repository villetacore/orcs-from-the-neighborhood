//! Главное меню: выбор героев, новая партия, продолжение, ссылки на колоду и правила.

use bevy_egui::egui::{self, Frame, Ui};
use orcs_core::{CardId, MAX_PLAYERS, MIN_PLAYERS};

use super::widgets::{button, toast, wrapped, wrapped_colored};
use super::{Screen, View, card, theme};
use crate::session::GameCommand;

pub fn show(root: &mut Ui, view: &mut View) {
    egui::Panel::right("menu-info")
        .exact_size(620.0)
        .resizable(false)
        .frame(Frame::NONE.inner_margin(egui::Margin {
            left: 18,
            right: 28,
            top: 28,
            bottom: 22,
        }))
        .show(root, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| info(ui, view));
        });
    egui::CentralPanel::default()
        .frame(Frame::NONE.inner_margin(egui::Margin {
            left: 22,
            right: 18,
            top: 22,
            bottom: 22,
        }))
        .show(root, |ui| poster(ui, view));
}

fn poster(ui: &mut Ui, view: &View) {
    theme::panel(theme::PANEL_DEEP).show(ui, |ui| {
        ui.set_min_size(ui.available_size());
        ui.label(theme::text(
            "БАЙКИ РЖАВОГО ДВОРА   /   КАРТОЧНАЯ RPG",
            15.0,
            theme::RUST,
        ));
        let bottom = 34.0;
        let size = egui::vec2(ui.available_width(), ui.available_height() - bottom);
        let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
        let ork = view.game.cards.get(&CardId::from("ork")).expect("в колоде есть орк");
        card::paint_art(ui.painter(), rect, ork, view.art.texture(&ork.id));
        ui.label(theme::text(
            "90 КАРТ    /    2–4 ИГРОКА    /    ОДИН СТОЛ",
            18.0,
            theme::TEXT,
        ));
    });
}

fn info(ui: &mut Ui, view: &mut View) {
    ui.spacing_mut().item_spacing.y = 14.0;
    ui.label(theme::heading("ОРКИ", 76.0, theme::GOLD));
    ui.label(theme::heading("С РАЙОНА", 34.0, theme::RUST));
    wrapped(
        ui,
        "Мир кончился. Квартплата — нет.\nТоргуйся с джиннами, прокачивай перки и подставляй корешей.\nДойди до уровня 9 и победи Хозяина района.",
        19.0,
    );
    ui.label(theme::text("КТО САДИТСЯ ЗА СТОЛ?", 17.0, theme::GOLD));
    for hero in view.game.cards.heroes() {
        let mut on = view.state.chosen.contains(&hero.id);
        let response = ui
            .checkbox(&mut on, theme::text(&hero.name, 17.0, theme::TEXT))
            .on_hover_text(&hero.text);
        if response.changed() {
            if on {
                view.state.chosen.push(hero.id.clone());
            } else {
                view.state.chosen.retain(|c| *c != hero.id);
            }
        }
    }
    wrapped_colored(
        ui,
        "Локальная игра с передачей экрана. Перед просмотром чужой руки появляется закрывающая стол ширма. Ботов и сетевого режима в этой версии нет.",
        15.0,
        theme::TEXT_MUTED,
    );
    let count = view.state.chosen.len();
    let ready = (MIN_PLAYERS..=MAX_PLAYERS).contains(&count);
    let start = button(ui, theme::heading("СЕСТЬ ЗА СТОЛ  →", 18.0, theme::TEXT), ready);
    if start.on_disabled_hover_text("Выберите от 2 до 4 персонажей.").clicked() {
        view.send(GameCommand::NewGame(view.state.chosen.clone()));
    }
    let save_exists = view.save_exists;
    if button(ui, "Продолжить сохранённую партию", save_exists)
        .on_disabled_hover_text("Сохранённой партии пока нет.")
        .clicked()
    {
        view.send(GameCommand::Continue);
    }
    ui.horizontal(|ui| {
        if button(ui, "Вся колода · 90", true).clicked() {
            view.go(Screen::Gallery);
        }
        if button(ui, "Как играть", true).clicked() {
            view.go(Screen::Rules);
        }
    });
    toast(ui, &view.state.toast);
}
