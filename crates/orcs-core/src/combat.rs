//! Бой: сила сторон, очередь вмешательства, помощь и итог.

use std::sync::Arc;

use crate::cards::{CardId, Effect, Penalty, Slot, Tag};
use crate::game::{Game, RuleResult, WIN_LEVEL, reject};
use crate::state::{Boost, BoostSource, Choice, ChoiceKind, ChoiceOption, Phase, StrengthPart};

/// Минимальная сила героя против дракона.
pub const DRAGON_THRESHOLD: i32 = 10;
pub const HELP_BONUS: i32 = 2;

fn part(label: impl Into<String>, amount: i32) -> StrengthPart {
    StrengthPart {
        label: label.into(),
        amount,
    }
}

impl Game {
    fn encounter_tag(&self) -> Option<Tag> {
        self.encounter.as_ref().and_then(|id| self.cards[id].tag)
    }

    /// Имя компонента стола по ID: подписи берутся из данных и не устаревают при переименовании.
    fn card_name(&self, id: &str) -> String {
        self.cards
            .get(&CardId::from(id))
            .map_or_else(|| id.to_owned(), |c| c.name.clone())
    }

    /// Из чего складывается сила героя — показывается на столе.
    pub fn strength_breakdown(&self) -> Vec<StrengthPart> {
        let a = self.active;
        let p = &self.players[a];
        let mut parts = vec![part("Уровень", p.level)];
        if p.water != 0 {
            parts.push(part("Холодная вода", p.water));
        }
        for slot in Slot::ALL {
            let Some(id) = p.gear(slot) else { continue };
            let card = &self.cards[id];
            let mut bonus = card.bonus;
            if slot == Slot::Weapon {
                bonus = (bonus + self.weapon_mod).max(0);
            }
            parts.push(part(slot.title(), bonus));
            if card.effect == Effect::Undead && self.encounter_tag() == Some(Tag::Undead) {
                parts.push(part("Против нежити", 2));
            }
        }
        if self.ability(a, Effect::Mechanic) && self.technical(a) {
            parts.push(part("Механик", 1));
        }
        if self.ability(a, Effect::MutantHunter) && self.encounter_tag() == Some(Tag::Mutant) {
            parts.push(part("Сталкер", 1));
        }
        if self.battle_penalty != 0 {
            parts.push(part("Штраф встречи", self.battle_penalty));
        }
        for boost in &self.boosts {
            let label = match &boost.source {
                BoostSource::HelpContract => "Помощь".to_owned(),
                BoostSource::Paid => "Платный аргумент".to_owned(),
                BoostSource::Card(id) => self.cards[id].name.clone(),
            };
            parts.push(part(label, boost.amount));
        }
        if self.has_perk(a, "solo") && self.helper.is_none() {
            parts.push(part(self.card_name("perk_solo"), 1));
        }
        if self.has_perk(a, "specialist") && self.encounter_tag() == Some(Tag::Construct) {
            parts.push(part(self.card_name("perk_specialist"), 2));
        }
        if self.has_perk(a, "veteran")
            && self
                .encounter
                .as_ref()
                .is_some_and(|id| self.cards[id].effect == Effect::Boss)
        {
            parts.push(part(self.card_name("perk_veteran"), 2));
        }
        if self.has_perk(a, "authority") && self.helper.is_some() {
            parts.push(part(self.card_name("perk_authority"), 1));
        }
        if self.district_is("district_depot") && self.technical(a) {
            parts.push(part(self.card_name("district_depot"), 1));
        }
        parts
    }

    pub fn hero_strength(&self) -> i32 {
        self.strength_breakdown().iter().map(|p| p.amount).sum::<i32>().max(0)
    }

    pub fn strength_text(&self) -> String {
        self.strength_breakdown()
            .iter()
            .map(|p| format!("{} {:+}", p.label, p.amount))
            .collect::<Vec<_>>()
            .join(" · ")
    }

    pub fn enemy_strength(&self) -> i32 {
        let Some(id) = &self.encounter else { return 0 };
        let card = &self.cards[id];
        let hero = &self.players[self.active];
        let mut total = card.strength + self.enemy_mod;
        if card.effect == Effect::Boss {
            total += self.active_perks(self.active).count() as i32;
        }
        if card.tag == Some(Tag::Spirit) && self.district_is("district_forest") {
            total += 1;
        }
        match card.effect {
            Effect::Unarmored if hero.armor.is_none() => total += 1,
            Effect::Armor if hero.armor.is_some() => total += 1,
            Effect::Technical if hero.weapon.as_ref().is_some_and(|w| self.cards[w].technical) => total += 1,
            _ => {}
        }
        total.max(0)
    }

    /// При равенстве побеждает герой; дракону нужна сила не меньше 10.
    pub fn can_win(&self) -> bool {
        let Some(id) = &self.encounter else { return false };
        let hero = self.hero_strength();
        hero >= self.enemy_strength() && (self.cards[id].effect != Effect::Dragon || hero >= DRAGON_THRESHOLD)
    }

    pub fn begin_battle(&mut self, id: CardId) {
        let a = self.active;
        self.encounter = Some(id.clone());
        self.fought = true;
        self.phase = Phase::Combat;
        self.reaction = a;
        self.reaction_step = 0;
        self.chaos_used.clear();
        self.boosts.clear();
        self.enemy_mod = 0;
        self.weapon_mod = 0;
        self.battle_penalty = -self.players[a].next_penalty;
        self.players[a].next_penalty = 0;
        self.lucky = false;
        self.rusk = false;
        self.helper = None;
        self.help_requested = false;
        self.help_for_coins = false;
        self.players[a].paid_boost_used = false;
        self.players[a].reserve_used = false;
        if self.cards[&id].effect == Effect::Pipe {
            let cards = Arc::clone(&self.cards);
            let mut options: Vec<ChoiceOption> = self.players[a]
                .hand
                .iter()
                .filter(|held| cards[*held].card_type == crate::cards::CardType::Consumable)
                .cloned()
                .map(ChoiceOption::Card)
                .collect();
            options.push(ChoiceOption::Penalty);
            self.choices.push_back(Choice {
                player: a,
                kind: ChoiceKind::Pipe,
                title: format!("{}: сбрось расходник или получи −1 силы", cards[&id].name),
                options,
            });
        }
    }

    /// Все соперники высказались — последнее слово за героем.
    pub fn final_response(&self) -> bool {
        self.phase == Phase::Combat && self.reaction_step == self.players.len()
    }

    pub fn reaction_text(&self) -> String {
        if self.final_response() {
            return "Финальный ответ героя. После подтверждения бой завершится; нового круга нет.".into();
        }
        let remaining = self.players.len().saturating_sub(self.reaction_step + 1);
        format!("После вас вмешаются ещё {remaining}. Затем герой получит финальный ответ.")
    }

    pub fn help_check(&self) -> RuleResult {
        if self.phase != Phase::Combat || self.winner.is_some() || !self.choices.is_empty() {
            return reject("Сначала завершите текущий выбор.");
        }
        if self.reaction_step != 0 {
            return reject("Помощь предлагают до первого круга вмешательства.");
        }
        if self.helper.is_some() {
            return reject("В этом бою уже есть помощник.");
        }
        if self.help_requested {
            return reject("Предложение помощи уже было сделано в этом бою.");
        }
        if !self.boosts.is_empty() || self.chaos_used.contains(&self.active) || self.lucky || self.rusk {
            return reject("Предложите помощь до розыгрыша боевых карт.");
        }
        if self.help_candidates().is_empty() {
            return reject("У соперников нет карт для оплаты помощи.");
        }
        Ok(())
    }

    fn help_candidates(&self) -> Vec<ChoiceOption> {
        (0..self.players.len())
            .filter(|&i| i != self.active && !self.players[i].hand.is_empty())
            .map(ChoiceOption::Player)
            .collect()
    }

    pub fn request_help(&mut self) -> RuleResult {
        self.help_check()?;
        self.help_for_coins = false;
        self.queue_help_target();
        Ok(())
    }

    pub(crate) fn queue_help_target(&mut self) {
        let mut options = self.help_candidates();
        options.push(ChoiceOption::Cancel);
        self.choices.push_back(Choice {
            player: self.active,
            kind: ChoiceKind::HelpTarget,
            title: format!("Кому предложить помощь? {}", self.help_payment_text()),
            options,
        });
    }

    pub(crate) fn offer_help_to(&mut self, p: usize) {
        self.help_requested = true;
        let mut options = ChoiceOption::cards(&self.players[p].hand);
        options.push(ChoiceOption::Decline);
        let title = format!(
            "{} просит помощи: сбросьте карту. {} Можно отказаться.",
            self.player_name(self.active),
            self.help_payment_text()
        );
        self.choices.push_front(Choice {
            player: p,
            kind: ChoiceKind::HelpAccept,
            title,
            options,
        });
    }

    pub(crate) fn accept_help(&mut self, p: usize, payment: &CardId) {
        crate::game::remove_card(&mut self.players[p].hand, payment);
        self.discard_card(payment.clone());
        self.helper = Some(p);
        if self.help_for_coins {
            let price = if self.has_perk(self.active, "contacts") { 1 } else { 2 };
            self.players[self.active].coins -= price;
            self.players[p].coins += price;
        }
        self.pay_helper_perk(p);
        self.boosts.push(Boost {
            amount: if self.ability(p, Effect::Oni) { 3 } else { HELP_BONUS },
            source: BoostSource::HelpContract,
        });
        self.note(format!(
            "{} помогает, сбрасывая «{}». Оплата по согласованному договору.",
            self.player_name(p),
            self.cards[payment].name
        ));
    }

    /// Текущий участник завершает вмешательство; в финальном ответе — подтверждает бой.
    pub fn pass_reaction(&mut self) -> RuleResult {
        if self.phase != Phase::Combat || !self.choices.is_empty() {
            return reject("Сначала завершите текущий выбор.");
        }
        if self.final_response() {
            self.note(format!("{} подтверждает результат боя.", self.player_name(self.active)));
            self.resolve_battle();
            return Ok(());
        }
        self.note(format!("{} завершает вмешательство.", self.player_name(self.reaction)));
        self.reaction_step += 1;
        if self.reaction_step == self.players.len() {
            self.reaction = self.active;
            self.note("Последнее слово возвращается герою.");
        } else {
            self.reaction = (self.active + self.reaction_step) % self.players.len();
        }
        Ok(())
    }

    pub fn resolve_battle(&mut self) {
        let Some(enemy_id) = self.encounter.clone() else { return };
        let cards = Arc::clone(&self.cards);
        let enemy = &cards[&enemy_id];
        let a = self.active;
        let won = self.can_win();
        self.note(format!(
            "Бой: {} против {}. {}",
            self.hero_strength(),
            self.enemy_strength(),
            if won { "Победа!" } else { "Поражение." }
        ));
        self.players[a].silenced = false;
        if won {
            let cap = if enemy.effect == Effect::Boss {
                WIN_LEVEL
            } else {
                WIN_LEVEL - 1
            };
            self.players[a].level = (self.players[a].level + enemy.level_reward).min(cap);
            self.players[a].coins += enemy.coins;
            for j in 0..enemy.loot_cards {
                let receiver = if j == 0 && !self.help_for_coins {
                    self.helper.unwrap_or(a)
                } else {
                    a
                };
                self.take(receiver, 1);
            }
            let club = self.players[a]
                .weapon
                .as_ref()
                .is_some_and(|w| cards[w].effect == Effect::VictoryDiscard);
            if club && !self.players[a].hand.is_empty() {
                self.choices.push_back(Choice {
                    player: a,
                    kind: ChoiceKind::Discard,
                    title: format!(
                        "{}: выбери карту для сброса",
                        self.players[a]
                            .weapon
                            .as_ref()
                            .map_or("Оружие", |w| cards[w].name.as_str())
                    ),
                    options: ChoiceOption::cards(&self.players[a].hand),
                });
            }
        } else if !self.lucky {
            let penalty = enemy.penalty.unwrap_or(Penalty::Card);
            if matches!(penalty, Penalty::Level | Penalty::LevelCard | Penalty::LevelCoins) && !self.rusk {
                self.players[a].level = (self.players[a].level - 1).max(1);
            }
            if matches!(penalty, Penalty::LevelCoins | Penalty::Coins) {
                let discount = if self.has_perk(a, "insurance") { 2 } else { 0 }
                    + if self.district_is("district_sanatorium") { 1 } else { 0 };
                self.players[a].coins = self.players[a].coins.saturating_sub(2u32.saturating_sub(discount));
            }
            if matches!(penalty, Penalty::Card | Penalty::TwoCards | Penalty::LevelCard) {
                self.lose_card(a);
            }
            if penalty == Penalty::TwoCards {
                // Вторая потеря — после первого решения, чтобы варианты не устарели.
                self.choices.push_back(Choice {
                    player: a,
                    kind: ChoiceKind::SecondLoss,
                    title: format!("{}: потеря второй карты", enemy.name),
                    options: vec![ChoiceOption::Ok],
                });
            }
            let slot = match penalty {
                Penalty::Armor => Some(Slot::Armor),
                Penalty::Weapon => Some(Slot::Weapon),
                _ => None,
            };
            if let Some(slot) = slot {
                if self.players[a].gear(slot).is_some() {
                    self.remove_gear(a, slot);
                } else {
                    self.lose_card(a);
                }
            }
        }
        if enemy.effect != Effect::Boss {
            self.discard_card(enemy_id.clone());
        }
        self.last_card = Some(enemy_id);
        self.encounter = None;
        self.phase = Phase::Actions;
        self.boosts.clear();
        self.weapon_mod = 0;
        self.battle_penalty = 0;
        if !won && self.has_perk(a, "stash_loss") {
            self.choices.push_back(Choice {
                player: a,
                kind: ChoiceKind::AfterLoss,
                title: "Заначка: забрать Барахло после штрафов".into(),
                options: vec![ChoiceOption::Ok],
            });
        }
        self.queue_perks();
        self.finish_victory();
    }

    /// Победа засчитывается, когда все последствия боя разрешены.
    pub(crate) fn finish_victory(&mut self) {
        let a = self.active;
        if self.choices.is_empty()
            && self.phase == Phase::Actions
            && self.players.get(a).is_some_and(|p| p.level >= WIN_LEVEL)
            && self.winner.is_none()
        {
            self.winner = Some(a);
            self.note(format!("{} становится легендой района!", self.player_name(a)));
        }
    }
}
