//! Розыгрыш карт с руки. Интерфейс и перетаскивание пользуются той же проверкой, что и движок.

use std::sync::Arc;

use crate::cards::{CardId, CardType, Effect};
use crate::game::{Game, NON_COMBAT_LEVEL_CAP, RuleResult, reject, remove_card};
use crate::state::{Boost, BoostSource, Choice, ChoiceKind, ChoiceOption, Phase};

impl Game {
    /// Можно ли сейчас сыграть карту `id` владельцем текущего решения.
    pub fn play_check(&self, id: &CardId) -> RuleResult {
        if !self.is_running() {
            return reject("Партия не идёт.");
        }
        if !self.choices.is_empty() {
            return reject("Сначала завершите текущий выбор.");
        }
        let who = self.owner();
        if !self.players[who].holds(id) {
            return reject("Выберите карту из своей руки.");
        }
        let a = self.active;
        let card = &self.cards[id];
        if self.phase == Phase::Combat {
            if card.card_type == CardType::Consumable {
                if who != a {
                    return reject("Расходники применяет только герой этого боя.");
                }
                if !matches!(card.effect, Effect::Boost | Effect::Tape | Effect::Luck | Effect::Rusk) {
                    return reject("Этот расходник используется после исследования.");
                }
                if card.effect == Effect::Tape && self.players[a].weapon.is_none() {
                    return reject("Для изоленты нужно оружие.");
                }
                return Ok(());
            }
            if card.card_type != CardType::Chaos {
                return reject("В бой можно играть расходники и боевой хаос.");
            }
            if self.chaos_used.contains(&who) {
                return reject("Вы уже сыграли свою карту хаоса в этом бою.");
            }
            return match card.effect {
                Effect::EnemyUp | Effect::Noise | Effect::EnemyDown => Ok(()),
                Effect::WeaponDown => {
                    let positive = self.players[a]
                        .weapon
                        .as_ref()
                        .is_some_and(|w| self.cards[w].bonus + self.weapon_mod > 0);
                    if who == a || !positive {
                        reject("Нужно оружие соперника с положительным бонусом.")
                    } else {
                        Ok(())
                    }
                }
                Effect::Battery if who == a || !self.technical(a) => reject("Нужна техническая экипировка соперника."),
                Effect::Battery => Ok(()),
                Effect::HeroUp if who != a => reject("Только в свой бой."),
                Effect::HeroUp => Ok(()),
                Effect::Friend if who == a || self.helper.is_some() => reject("Нужен чужой бой без другого помощника."),
                Effect::Friend => Ok(()),
                Effect::Cancel if self.boosts.iter().any(|b| b.amount > 0) => Ok(()),
                Effect::Cancel => reject("Нет положительного временного бонуса для отмены."),
                _ => reject("Эта карта применяется после исследования."),
            };
        }
        if self.players[a].skip_now {
            return reject("Энергетик: подготовка и действия в этот ход пропущены.");
        }
        if self.phase == Phase::Explore && card.card_type != CardType::Equipment {
            return reject(
                "В подготовку можно надевать снаряжение, обмениваться и заменять карту. Остальное — после исследования.",
            );
        }
        match card.card_type {
            CardType::Equipment | CardType::Event => Ok(()),
            CardType::Enemy if self.fought => reject("В этот ход уже был бой."),
            CardType::Enemy => Ok(()),
            CardType::Consumable if matches!(card.effect, Effect::Energy | Effect::Cycle) => Ok(()),
            CardType::Consumable => reject("Этот расходник используется во время боя."),
            CardType::Chaos if matches!(card.effect, Effect::Ambush | Effect::Stash) => Ok(()),
            CardType::Chaos => reject("Эта карта хаоса используется во время боя."),
            CardType::Character | CardType::Perk | CardType::District | CardType::Shop => {
                reject("Эту карту сейчас нельзя сыграть.")
            }
        }
    }

    pub fn play(&mut self, id: &CardId) -> RuleResult {
        self.play_check(id)?;
        let cards = Arc::clone(&self.cards);
        let card = &cards[id];
        let who = self.owner();
        let a = self.active;
        if self.phase == Phase::Combat {
            if card.card_type == CardType::Consumable {
                match card.effect {
                    Effect::Boost => self.boost(2, id),
                    Effect::Tape => self.boost(1, id),
                    Effect::Luck => self.lucky = true,
                    Effect::Rusk => self.rusk = true,
                    _ => {}
                }
            } else {
                match card.effect {
                    Effect::EnemyUp => self.enemy_mod += 2,
                    Effect::Noise => self.enemy_mod += 3,
                    Effect::EnemyDown => self.enemy_mod -= 2,
                    Effect::WeaponDown => self.weapon_mod -= 1,
                    Effect::Battery => self.boost(-2, id),
                    Effect::HeroUp => self.boost(1, id),
                    Effect::Friend => {
                        self.helper = Some(who);
                        self.boost(if self.ability(who, Effect::Oni) { 3 } else { 2 }, id);
                        self.help_for_coins = false;
                        self.pay_helper_perk(who);
                    }
                    Effect::Cancel => self.cancel_latest_bonus(),
                    _ => {}
                }
                self.chaos_used.push(who);
            }
        } else {
            match card.card_type {
                CardType::Equipment => {
                    let slot = card.slot.expect("у снаряжения есть слот");
                    self.remove_gear(a, slot);
                    remove_card(&mut self.players[a].hand, id);
                    *self.players[a].gear_mut(slot) = Some(id.clone());
                    self.note(format!("{} экипирует «{}».", self.player_name(a), card.name));
                    return Ok(());
                }
                CardType::Enemy => {
                    remove_card(&mut self.players[a].hand, id);
                    self.note(format!("Герой вызывает с руки «{}».", card.name));
                    self.begin_battle(id.clone());
                    return Ok(());
                }
                CardType::Event => {
                    self.discard_from_hand(a, id);
                    self.note(format!("{} разыгрывает «{}».", self.player_name(a), card.name));
                    self.event_effect(card.effect);
                    return Ok(());
                }
                CardType::Consumable if card.effect == Effect::Energy => {
                    self.players[a].level = (self.players[a].level + 1).min(NON_COMBAT_LEVEL_CAP);
                    self.players[a].skip_next = true;
                }
                CardType::Consumable => {
                    // Котлета не попадает в собственный добор: сначала убрать, потом брать.
                    remove_card(&mut self.players[a].hand, id);
                    self.take(a, 2);
                    self.discard_card(id.clone());
                    if !self.players[a].hand.is_empty() {
                        self.choices.push_back(Choice {
                            player: a,
                            kind: ChoiceKind::Discard,
                            title: "Котлета: возьми две, затем выбери одну карту для сброса".into(),
                            options: ChoiceOption::cards(&self.players[a].hand),
                        });
                    }
                    self.note(format!("Герой использует «{}».", card.name));
                    return Ok(());
                }
                CardType::Chaos if card.effect == Effect::Stash => {
                    remove_card(&mut self.players[a].hand, id);
                    let options: Vec<ChoiceOption> =
                        (0..2).filter_map(|_| self.draw()).map(ChoiceOption::Card).collect();
                    self.discard_card(id.clone());
                    if !options.is_empty() {
                        self.choices.push_back(Choice {
                            player: a,
                            kind: ChoiceKind::Stash,
                            title: "Тайник: возьми одну карту, вторую положи вниз колоды".into(),
                            options,
                        });
                    }
                    self.note(format!("{} открывает свой тайник.", self.player_name(a)));
                    return Ok(());
                }
                CardType::Chaos => {
                    let options = (0..self.players.len())
                        .filter(|&i| i != a)
                        .map(ChoiceOption::Player)
                        .collect();
                    self.choices.push_back(Choice {
                        player: a,
                        kind: ChoiceKind::Ambush,
                        title: "Кому устроить подставу?".into(),
                        options,
                    });
                }
                CardType::Character | CardType::Perk | CardType::District | CardType::Shop => {}
            }
        }
        self.discard_from_hand(who, id);
        if card.card_type == CardType::Consumable
            && self.phase == Phase::Combat
            && self.has_perk(a, "reserve")
            && !self.players[a].reserve_used
        {
            self.players[a].reserve_used = true;
            self.take(a, 1);
        }
        self.queue_perks();
        self.note(format!("{} играет «{}».", self.player_name(who), card.name));
        Ok(())
    }

    fn boost(&mut self, amount: i32, id: &CardId) {
        self.boosts.push(Boost {
            amount,
            source: BoostSource::Card(id.clone()),
        });
    }

    /// «Курсы успеха от суккубы»: снимает последний положительный временный бонус.
    fn cancel_latest_bonus(&mut self) {
        if let Some(i) = self.boosts.iter().rposition(|b| b.amount > 0) {
            let removed = self.boosts.remove(i);
            let helper_bonus = match &removed.source {
                BoostSource::HelpContract => true,
                BoostSource::Paid => false,
                BoostSource::Card(id) => self.cards[id].effect == Effect::Friend,
            };
            if helper_bonus {
                self.helper = None;
            }
        }
    }
}
