//! Открытие карт исследования, события и потери карт.

use crate::cards::{CardId, CardType, Effect, Slot};
use crate::game::{Game, remove_card};
use crate::state::{Choice, ChoiceKind, ChoiceOption};

impl Game {
    /// Открывает верхнюю карту колоды для активного героя.
    pub(crate) fn open_card(&mut self) {
        let a = self.active;
        let Some(id) = self.draw_trouble() else {
            self.note("Колода пуста. Можно перейти к действиям.");
            return;
        };
        self.last_card = Some(id.clone());
        let card = self.cards[&id].clone();
        self.note(format!("{} открывает «{}».", self.player_name(a), card.name));
        match card.card_type {
            // Второй враг за ход уходит в руку.
            CardType::Enemy if self.fought => self.players[a].hand.push(id),
            CardType::Enemy => self.begin_battle(id),
            CardType::Event => {
                self.discard_card(id);
                self.event_effect(card.effect);
            }
            _ => self.players[a].hand.push(id),
        }
    }

    pub fn event_effect(&mut self, effect: Effect) {
        let a = self.active;
        let n = self.players.len();
        match effect {
            Effect::Water => {
                for offset in 0..n {
                    let p = (a + offset) % n;
                    let mut options = ChoiceOption::cards(&self.players[p].hand);
                    options.push(ChoiceOption::Penalty);
                    self.choices.push_back(Choice {
                        player: p,
                        kind: ChoiceKind::Water,
                        title: "Нет горячей воды: сбрось карту или −1 силы до своего хода".into(),
                        options,
                    });
                }
            }
            Effect::Portal => {
                let options: Vec<ChoiceOption> = (0..2).filter_map(|_| self.draw()).map(ChoiceOption::Card).collect();
                if !options.is_empty() {
                    self.choices.push_back(Choice {
                        player: a,
                        kind: ChoiceKind::Portal,
                        title: "Портал: какую карту оставить?".into(),
                        options,
                    });
                }
            }
            Effect::Silence => self.choices.push_back(Choice {
                player: a,
                kind: ChoiceKind::Silence,
                title: "Кто лишится способности до конца своего следующего боя?".into(),
                options: (0..n).map(ChoiceOption::Player).collect(),
            }),
            Effect::Lift => {
                if self.players[a].hand.is_empty() {
                    self.players[a].next_penalty += 1;
                } else {
                    self.lose_card(a);
                }
            }
            Effect::Raid => {
                self.take(a, 1);
                self.players[a].coins += 1;
            }
            Effect::Reveal => self.choices.push_back(Choice {
                player: (a + 1) % n,
                kind: ChoiceKind::Reveal { subject: a },
                title: "Сосед показывает вам руку. Посмотрите и закройте.".into(),
                options: vec![ChoiceOption::Ok],
            }),
            Effect::Draw => self.take(a, 1),
            Effect::Harvest => self.take(a, 2),
            Effect::Rust => self.remove_gear(a, Slot::Armor),
            Effect::Tax => self.players[a].coins = self.players[a].coins.saturating_sub(2),
            _ => {}
        }
    }

    pub fn remove_gear(&mut self, p: usize, slot: Slot) {
        if self.phase == crate::state::Phase::Combat
            && !self.can_win()
            && !self.players[p].safety_used
            && (self.ability(p, Effect::Dwarf) || self.has_perk(p, "safety"))
            && self.players[p].gear(slot).is_some()
        {
            self.players[p].safety_used = true;
            self.note("Техника безопасности сохраняет предмет.");
            return;
        }
        if let Some(id) = self.players[p].gear_mut(slot).take() {
            self.note(format!("{} теряет «{}».", self.player_name(p), self.cards[&id].name));
            self.discard_card(id);
        }
    }

    /// Потеря случайной карты с учётом плаща и способности Зины.
    pub fn lose_card(&mut self, p: usize) {
        let player = &self.players[p];
        if player.hand.is_empty() {
            return;
        }
        let cloak = player
            .armor
            .as_ref()
            .is_some_and(|id| self.cards[id].effect == Effect::Protect);
        if cloak {
            self.choices.push_back(Choice {
                player: p,
                kind: ChoiceKind::Protect,
                title: "Сбросить плащ вместо случайной карты?".into(),
                options: vec![ChoiceOption::Protect, ChoiceOption::LoseRandom],
            });
        } else if self.ability(p, Effect::Witch) && !player.witch_used {
            let mut options = ChoiceOption::cards(&player.hand);
            options.push(ChoiceOption::Random);
            self.choices.push_back(Choice {
                player: p,
                kind: ChoiceKind::Witch,
                title: "Зина: выбери другую жертву или обычную случайную потерю".into(),
                options,
            });
        } else {
            self.random_loss(p);
        }
    }

    pub fn random_loss(&mut self, p: usize) {
        let len = self.players[p].hand.len();
        if len == 0 {
            return;
        }
        let index = self.rng.range_inclusive(0, len - 1);
        let id = self.players[p].hand.remove(index);
        self.note(format!("{} теряет «{}».", self.player_name(p), self.cards[&id].name));
        self.discard_card(id);
    }

    /// Добровольный сброс выбранной карты с руки.
    pub(crate) fn discard_from_hand(&mut self, p: usize, id: &CardId) {
        if remove_card(&mut self.players[p].hand, id) {
            self.discard_card(id.clone());
        }
    }
}
