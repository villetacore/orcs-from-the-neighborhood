//! Игровой стол: участники слева, общий стол по центру, карта крупным планом справа, рука снизу.
//! При смене владельца скрытой руки весь стол закрывает ширма.

use bevy_egui::egui::{self, Align, Frame, Layout, Margin, Ui};
use orcs_core::{CardId, Phase};

use super::card::{self, CardFace, HAND_CARD};
use super::model::{self, Action};
use super::widgets::{action_button, button, small_button, toast, wrapped, wrapped_colored};
use super::{Screen, View, gallery, theme};
use crate::session::GameCommand;

const ROSTER_WIDTH: f32 = 250.0;
const INSPECTOR_WIDTH: f32 = 300.0;
const LOG_LINES: usize = 4;

pub fn show(root: &mut Ui, view: &mut View) {
    if model::curtain_needed(view.game, view.state.private_owner) {
        view.state.selected = None;
        curtain(root, view);
        return;
    }
    header(root, view);
    if let Some(winner) = view.game.winner {
        egui::Panel::bottom("winner")
            .frame(Frame::NONE.inner_margin(Margin::symmetric(22, 12)))
            .show(root, |ui| {
                theme::panel(theme::WIN_BANNER).show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.label(theme::heading(
                        format!("ЛЕГЕНДА РАЙОНА — {}! УРОВЕНЬ 10", view.game.player_name(winner)),
                        30.0,
                        theme::GOLD_LIGHT,
                    ));
                });
            });
    } else {
        egui::Panel::bottom("hand")
            .exact_size(312.0)
            .resizable(false)
            .frame(Frame::NONE.inner_margin(Margin {
                left: 22,
                right: 22,
                top: 4,
                bottom: 18,
            }))
            .show(root, |ui| hand_area(ui, view));
    }
    egui::Panel::left("roster")
        .exact_size(ROSTER_WIDTH + 22.0)
        .resizable(false)
        .frame(Frame::NONE.inner_margin(Margin {
            left: 22,
            right: 0,
            top: 4,
            bottom: 6,
        }))
        .show(root, |ui| roster(ui, view));
    egui::Panel::right("inspector")
        .exact_size(INSPECTOR_WIDTH + 22.0)
        .resizable(false)
        .frame(Frame::NONE.inner_margin(Margin {
            left: 0,
            right: 22,
            top: 4,
            bottom: 6,
        }))
        .show(root, |ui| inspector(ui, view));
    egui::CentralPanel::default()
        .frame(Frame::NONE.inner_margin(Margin {
            left: 14,
            right: 14,
            top: 4,
            bottom: 6,
        }))
        .show(root, |ui| center_table(ui, view));
    shop_window(root.ctx(), view);
    discard_window(root.ctx(), view);
    gallery::card_popup(root.ctx(), view);
}

fn header(root: &mut Ui, view: &mut View) {
    egui::Panel::top("table-header")
        .frame(Frame::NONE.inner_margin(Margin::symmetric(22, 14)))
        .show(root, |ui| {
            ui.horizontal(|ui| {
                ui.label(theme::heading("ОРКИ С РАЙОНА", 31.0, theme::GOLD));
                ui.label(theme::text(
                    format!("    ХОД {:02}", view.game.turn),
                    16.0,
                    theme::TEXT_DIM,
                ));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if button(ui, "Магазин / район", true).clicked() {
                        view.state.shop_open = !view.state.shop_open;
                    }
                    if button(ui, "Меню", true).clicked() {
                        view.go(Screen::Menu);
                    }
                    if button(ui, "Колода", true).clicked() {
                        view.go(Screen::Gallery);
                    }
                    if button(ui, "Правила", true).clicked() {
                        view.go(Screen::Rules);
                    }
                });
            });
        });
}

fn curtain(root: &mut Ui, view: &mut View) {
    let owner = view.game.owner();
    egui::CentralPanel::default()
        .frame(Frame::NONE.fill(theme::CURTAIN))
        .show(root, |ui| {
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                ui.add_space((ui.available_height() / 2.0 - 170.0).max(0.0));
                ui.set_max_width(640.0);
                ui.spacing_mut().item_spacing.y = 24.0;
                ui.label(theme::text("✦   ПЕРЕДАЙ ЭКРАН", 22.0, theme::GOLD));
                ui.label(theme::heading(view.game.player_name(owner), 44.0, theme::GOLD_LIGHT));
                ui.label(theme::text(
                    "Остальные игроки отворачиваются.\nСледующий игрок открывает свою руку, когда будет готов.",
                    22.0,
                    theme::TEXT,
                ));
                if button(ui, theme::text("Я готов — открыть мою руку", 18.0, theme::TEXT), true).clicked()
                {
                    view.state.private_owner = Some(owner);
                }
            });
        });
}

fn roster(ui: &mut Ui, view: &mut View) {
    let game = view.game;
    egui::Panel::bottom("roster-footer")
        .frame(Frame::NONE.inner_margin(Margin::symmetric(0, 6)))
        .show(ui, |ui| {
            if button(ui, "Закрыть руку ширмой", true).clicked() {
                view.state.private_owner = None;
            }
        });
    ui.label(theme::text("ЗА СТОЛОМ", 14.0, theme::TEXT_DIM));
    egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        ui.set_width(ROSTER_WIDTH - 12.0);
        for (i, p) in game.players.iter().enumerate() {
            let active = i == game.active;
            theme::panel(if active { theme::PANEL_ACTIVE } else { theme::PANEL })
                .inner_margin(Margin::same(12))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.spacing_mut().item_spacing.y = 5.0;
                    let marker = if active { "▸ " } else { "" };
                    ui.label(theme::heading(
                        format!("{marker}{}", game.player_name(i)),
                        19.0,
                        theme::TEXT,
                    ));
                    ui.label(theme::text(model::roster_line(game, i), 14.0, theme::GOLD));
                    ui.add(
                        egui::ProgressBar::new(p.level as f32 / orcs_core::WIN_LEVEL as f32)
                            .desired_height(5.0)
                            .fill(theme::RUST),
                    );
                    for (gear, mark) in [(&p.weapon, "⚔"), (&p.armor, "◈")] {
                        if let Some(id) = gear
                            && small_button(ui, &format!("{mark} {}", game.card(id).name), 12.0).clicked()
                        {
                            view.state.selected = Some(id.clone());
                        }
                    }
                    for perk in &p.perks {
                        let active = game.card(perk).tier <= p.level;
                        if small_button(
                            ui,
                            &format!("{} {}", if active { "✦" } else { "○" }, game.card(perk).name),
                            11.0,
                        )
                        .clicked()
                        {
                            view.state.selected = Some(perk.clone());
                        }
                    }
                    if p.silenced {
                        ui.label(theme::text("Способность заблокирована", 12.0, theme::WARNING));
                    }
                    if p.water < 0 {
                        ui.label(theme::text("Нет горячей воды: −1 силы", 12.0, theme::WARNING));
                    }
                    if p.skip_now {
                        ui.label(theme::text("Энергетик: без действий", 12.0, theme::WARNING));
                    }
                    if small_button(ui, "Способность / портрет", 11.0).clicked() {
                        view.state.selected = Some(p.hero.clone());
                    }
                });
            ui.add_space(4.0);
        }
    });
}

fn center_table(ui: &mut Ui, view: &mut View) {
    let dragging = card::dragged_card(ui.ctx()).is_some();
    let hovering = dragging && ui.rect_contains_pointer(ui.available_rect_before_wrap());
    let mut frame = theme::panel(theme::TABLE);
    if hovering {
        frame = frame.stroke(egui::Stroke::new(2.0, theme::CARD_SELECTED));
    }
    let response = frame
        .show(ui, |ui| {
            ui.set_min_size(ui.available_size());
            table_contents(ui, view);
        })
        .response
        .on_hover_text("Перетащите сюда карту с руки, чтобы разыграть или экипировать её.");
    if let Some(id) = response.dnd_release_payload::<CardId>() {
        view.send(GameCommand::Play((*id).clone()));
    }
}

fn table_contents(ui: &mut Ui, view: &mut View) {
    let game = view.game;
    ui.label(theme::text(model::phase_title(game), 16.0, theme::TEXT_DIM));
    if let Some(id) = &game.district {
        let district = game.card(id);
        if small_button(ui, &format!("{} · {}", district.name, district.text), 12.0).clicked() {
            view.state.selected = Some(id.clone());
        }
    }
    ui.label(theme::text(
        "Перетащи карту с руки на стол, чтобы разыграть её",
        12.0,
        theme::TEXT_MUTED,
    ));
    egui::Panel::bottom("chronicle")
        .frame(Frame::NONE.inner_margin(Margin::symmetric(0, 4)))
        .show(ui, |ui| {
            Frame::new()
                .fill(theme::LOG)
                .stroke(egui::Stroke::new(1.0, theme::PANEL_BORDER))
                .corner_radius(10)
                .inner_margin(Margin::symmetric(16, 8))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(theme::text("ХРОНИКА ДВОРА", 11.0, theme::TEXT_DIM));
                    let start = game.journal.len().saturating_sub(LOG_LINES);
                    for line in &game.journal[start..] {
                        wrapped(ui, line, 12.0);
                    }
                });
        });
    if let Some(choice) = game.current_choice() {
        let decision = model::decision_view(game, choice);
        wrapped(ui, &decision.title, 23.0);
        if !decision.revealed.is_empty() {
            wrapped_colored(ui, decision.revealed.join("\n"), 18.0, theme::AMBER);
        }
        egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            for (label, option) in decision.options {
                let button = egui::Button::new(theme::text(label, 15.0, theme::TEXT)).wrap();
                if ui.add_sized([ui.available_width(), 36.0], button).clicked() {
                    view.send(GameCommand::Choose(option));
                }
            }
        });
        return;
    }
    egui::ScrollArea::vertical()
        .id_salt("table-actions")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 16.0;
                pile(ui, view);
                let shown = game.encounter.as_ref().or(game.last_card.as_ref());
                if let Some(id) = shown {
                    let card = game.card(id);
                    if CardFace::new(card, view.art, egui::vec2(190.0, 300.0))
                        .show(ui)
                        .clicked()
                    {
                        view.state.selected = Some(id.clone());
                    }
                }
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 6.0;
                    status(ui, view);
                });
            });
        });
}

fn pile(ui: &mut Ui, view: &mut View) {
    let game = view.game;
    ui.vertical(|ui| {
        ui.set_width(132.0);
        for (title, count, color) in [
            ("РАЗБОРКИ", game.trouble_deck.len(), theme::RUST),
            ("БАРАХЛО", game.deck.len(), theme::GOLD),
        ] {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(132.0, 100.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 7.0, theme::PANEL_DEEP);
            if let Some(texture) = view.art.back() {
                ui.painter().image(
                    texture,
                    rect.shrink(3.0),
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.25), egui::pos2(1.0, 0.75)),
                    egui::Color32::WHITE,
                );
            }
            let bar = egui::Rect::from_min_size(rect.left_bottom() - egui::vec2(0.0, 26.0), egui::vec2(132.0, 26.0));
            ui.painter().rect_filled(bar, 0.0, theme::PANEL_DEEP);
            ui.painter().text(
                bar.center(),
                egui::Align2::CENTER_CENTER,
                format!("{title} · {count}"),
                egui::FontId::proportional(13.0),
                color,
            );
        }
        if button(
            ui,
            format!("Сбросы · {}", game.discard.len() + game.trouble_discard.len()),
            !game.discard.is_empty() || !game.trouble_discard.is_empty(),
        )
        .clicked()
        {
            view.state.discard_open = !view.state.discard_open;
        }
    });
}

fn status(ui: &mut Ui, view: &mut View) {
    let game = view.game;
    match game.phase {
        Phase::Combat => {
            let Some(combat) = model::combat_view(game) else { return };
            ui.label(theme::heading(&combat.score, 25.0, theme::AMBER));
            wrapped(ui, &combat.breakdown, 12.0);
            wrapped(ui, &combat.turn, 18.0);
            if button(ui, theme::heading(&combat.confirm, 16.0, theme::TEXT), true).clicked() {
                view.send(GameCommand::PassReaction);
            }

            wrapped(ui, &combat.reaction, 13.0);
            wrapped(ui, &combat.chaos, 12.0);
            if let Some(helper) = &combat.helper {
                wrapped(ui, helper, 13.0);
            }
            if let Some(help) = &combat.help
                && action_button(ui, help)
            {
                view.send(GameCommand::RequestHelp);
            }
            if combat.help.is_some()
                && button(
                    ui,
                    format!(
                        "Помощь · {} кр.",
                        if game.has_perk(game.active, "contacts") { 1 } else { 2 }
                    ),
                    game.help_check().is_ok()
                        && game.players[game.active].coins
                            >= if game.has_perk(game.active, "contacts") { 1 } else { 2 },
                )
                .clicked()
            {
                view.send(GameCommand::PaidHelp);
            }
            if (game.ability(game.active, orcs_core::Effect::Djinn) || game.has_perk(game.active, "last_word"))
                && button(
                    ui,
                    format!(
                        "+2 силы · {} кр.",
                        if game.ability(game.active, orcs_core::Effect::Djinn) {
                            1
                        } else {
                            3
                        }
                    ),
                    game.paid_boost_check().is_ok(),
                )
                .on_disabled_hover_text(game.paid_boost_check().err().map_or(String::new(), |e| e.0))
                .clicked()
            {
                view.send(GameCommand::PaidBoost);
            }
            if combat.dragon {
                wrapped_colored(ui, "Дракон: нужно не менее 10 силы героя.", 12.0, theme::WARNING);
            }
        }
        Phase::Explore => {
            ui.label(theme::heading("СОБЕРИСЬ В ДОРОГУ", 25.0, theme::GOLD));
            wrapped(
                ui,
                "Сначала надень снаряжение, предложи обмен или замени одну карту. Затем открой карту исследования.",
                18.0,
            );
            wrapped_colored(
                ui,
                "Замена и лимит обменов общие на весь ход. Враги, события и расходники с руки — после исследования.",
                14.0,
                theme::TEXT_MUTED,
            );
            if game.players[game.active].skip_now {
                wrapped_colored(
                    ui,
                    "Энергетик: подготовка пропущена, но исследование и бой доступны.",
                    14.0,
                    theme::WARNING,
                );
            }
            if button(ui, "ПОДРАБОТАТЬ · от 2 кр.", game.work_check().is_ok()).clicked() {
                view.send(GameCommand::Work);
            }
            if game.players[game.active].level >= 9
                && button(ui, "ВЫЗВАТЬ ХОЗЯИНА РАЙОНА", game.challenge_check().is_ok())
                    .on_disabled_hover_text("Вызов доступен со следующего своего хода после достижения уровня 9")
                    .clicked()
            {
                view.send(GameCommand::Challenge);
            }
            if button(ui, theme::heading("ОТКРЫТЬ РАЗБОРКУ", 16.0, theme::TEXT), true).clicked() {
                view.send(GameCommand::Explore);
            }
        }
        Phase::Actions => {
            ui.label(theme::heading("ПЕРЕДЫШКА", 24.0, theme::GOLD));
            wrapped(
                ui,
                format!(
                    "Снарядись, загляни в магазин или предложи обмен. Лимит руки — {}.",
                    game.hand_limit(game.active)
                ),
                18.0,
            );
            let end = Action {
                label: "ЗАВЕРШИТЬ ХОД →".into(),
                enabled: game.end_turn_check().is_ok(),
                hint: game
                    .end_turn_check()
                    .err()
                    .map_or_else(|| "Передать ход следующему".into(), |e| e.0),
            };
            if action_button(ui, &end) {
                view.send(GameCommand::EndTurn);
            }
        }
    }
}

fn inspector(ui: &mut Ui, view: &mut View) {
    let game = view.game;
    ui.label(theme::text("КАРТА КРУПНЫМ ПЛАНОМ", 12.0, theme::TEXT_DIM));
    let owner = game.owner();
    let id = view
        .state
        .selected
        .clone()
        .or_else(|| game.encounter.clone())
        .unwrap_or_else(|| game.players[owner].hero.clone());
    let Some(card) = game.cards.get(&id) else { return };
    egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        if CardFace::new(card, view.art, egui::vec2(INSPECTOR_WIDTH - 18.0, 390.0))
            .show(ui)
            .clicked()
        {
            view.state.popup = Some(id.clone());
        }
        wrapped_colored(ui, &card.flavor, 14.0, theme::GOLD);
        // Полный текст правила на случай, если на лице карты он сокращён многоточием.
        wrapped(ui, card.rules_text(), 13.0);
        if game.players[owner].holds(&id) {
            wrapped(ui, model::play_hint(game, &id), 13.0);
        }
        toast(ui, &view.state.toast);
    });
}

fn hand_area(ui: &mut Ui, view: &mut View) {
    let game = view.game;
    let who = game.owner();
    let selected = view.state.selected.clone().filter(|id| game.players[who].holds(id));
    let actions = model::hand_actions(game, selected.as_ref());
    ui.horizontal(|ui| {
        ui.label(theme::heading(model::hand_title(game), 17.0, theme::GOLD));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let id = || selected.clone().expect("кнопка активна только с выбранной картой");
            if action_button(ui, &actions.discard) {
                view.send(GameCommand::DiscardExcess(id()));
            }
            if action_button(ui, &actions.trade) {
                view.send(GameCommand::Trade(id()));
            }
            if action_button(ui, &actions.recycle) {
                view.send(GameCommand::Recycle(id()));
            }
            if action_button(ui, &actions.play) {
                view.send(GameCommand::Play(id()));
            }
        });
    });
    egui::ScrollArea::horizontal().auto_shrink(false).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;
            let hand = &game.players[who].hand;
            if hand.is_empty() {
                ui.label(theme::text(
                    "Пустые карманы. Район бывает щедрым — но не сегодня.",
                    17.0,
                    theme::TEXT_MUTED,
                ));
            }
            for id in hand {
                let card = game.card(id);
                let face = CardFace::new(card, view.art, HAND_CARD)
                    .compact(true)
                    .selected(selected.as_ref() == Some(id))
                    .draggable(model::draggable(game, id));
                if face.show(ui).clicked() {
                    view.state.selected = Some(id.clone());
                    view.state.toast = None;
                }
            }
        });
    });
}

fn discard_window(ctx: &egui::Context, view: &mut View) {
    if !view.state.discard_open {
        return;
    }
    let mut open = true;
    egui::Window::new("Открытый сброс (сверху — последняя сброшенная)")
        .id(egui::Id::new("discard-window"))
        .collapsible(false)
        .default_size([430.0, 300.0])
        .open(&mut open)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for id in view
                    .game
                    .discard
                    .iter()
                    .rev()
                    .chain(view.game.trouble_discard.iter().rev())
                {
                    ui.label(theme::text(&view.game.card(id).name, 15.0, theme::TEXT));
                }
            });
        });
    view.state.discard_open = open;
}

fn shop_window(ctx: &egui::Context, view: &mut View) {
    if !view.state.shop_open {
        return;
    }
    if !view.game.choices.is_empty() {
        view.state.shop_open = false;
        return;
    }
    let mut open = true;
    egui::Window::new("Магазин Скупыча · район")
        .open(&mut open)
        .default_size([690.0, 580.0])
        .resizable(true)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let game = view.game;
                let a = game.active;
                if let Some(shop) = game.cards.get(&CardId::from("shop")) {
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(ui.available_width(), 135.0), egui::Sense::hover());
                    card::paint_art(ui.painter(), rect, shop, view.art.texture(&shop.id));
                }
                if let Some(id) = &game.district {
                    let district = game.card(id);
                    if small_button(ui, &district.name, 20.0).clicked() {
                        view.state.selected = Some(id.clone());
                    }
                    wrapped(ui, &district.text, 16.0);
                    wrapped_colored(ui, &district.flavor, 13.0, theme::GOLD);
                }
                ui.separator();
                wrapped(
                    ui,
                    format!(
                        "Кошелёк: {} кр. · Операции: {}/{}",
                        game.players[a].coins,
                        game.players[a].shop_ops,
                        game.shop_limit()
                    ),
                    18.0,
                );
                if let Err(reason) = game.shop_check() {
                    wrapped_colored(ui, reason.0, 14.0, theme::WARNING);
                }
                let selected = view.state.selected.clone().filter(|id| game.players[a].holds(id));
                wrapped(
                    ui,
                    "Выбери карту руки для бартера. Продажа — 1 крышак. Все цены видны до оплаты.",
                    14.0,
                );
                for id in &game.market {
                    let c = game.card(id);
                    theme::panel(theme::PANEL_DEEP).show(ui, |ui| {
                        if small_button(ui, &c.name, 18.0).clicked() {
                            view.state.popup = Some(id.clone());
                        }
                        wrapped(ui, &c.text, 14.0);
                        wrapped_colored(ui, &c.flavor, 12.0, theme::GOLD);
                        ui.horizontal_wrapped(|ui| {
                            if button(
                                ui,
                                format!("Купить · {} кр.", game.purchase_price(id, false)),
                                game.buy_check(id, None).is_ok(),
                            )
                            .clicked()
                            {
                                view.send(GameCommand::Buy(id.clone(), None));
                            }
                            let barter_ok = selected
                                .as_ref()
                                .is_some_and(|pay| game.buy_check(id, Some(pay)).is_ok());
                            if button(ui, format!("Бартер + {} кр.", game.purchase_price(id, true)), barter_ok)
                                .clicked()
                            {
                                view.send(GameCommand::Buy(id.clone(), selected.clone()));
                            }
                            if button(
                                ui,
                                "Обновить · 1 кр.",
                                game.shop_check().is_ok() && game.players[a].coins > 0,
                            )
                            .clicked()
                            {
                                view.send(GameCommand::Refresh(id.clone()));
                            }
                        });
                    });
                }
                if let Some(id) = &view.state.selected
                    && button(
                        ui,
                        format!("Продать «{}» · +1 кр.", game.card(id).name),
                        game.sell_check(id).is_ok(),
                    )
                    .clicked()
                {
                    view.send(GameCommand::Sell(id.clone()));
                }
                if button(
                    ui,
                    format!("Купить слух · {} кр.", game.scout_price()),
                    game.shop_check().is_ok() && game.players[a].coins >= game.scout_price(),
                )
                .clicked()
                {
                    view.send(GameCommand::Scout);
                }
                toast(ui, &view.state.toast);
            });
        });
    view.state.shop_open = open;
}
