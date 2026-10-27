//! Ход: исследование, замена, обмен, лимит руки и передача хода.

use crate::cards::CardId;
use crate::game::{Game, RuleResult, reject};
use crate::state::{Choice, ChoiceKind, ChoiceOption, Phase};

impl Game {
    pub fn explore(&mut self) -> RuleResult {
        if !self.is_running() || self.phase != Phase::Explore || !self.choices.is_empty() {
            return reject("Сейчас нельзя исследовать.");
        }
        self.phase = Phase::Actions;
        self.open_card();
        Ok(())
    }

    /// Общие условия для обмена и замены: подготовка или действия, без открытых решений.
    pub(crate) fn action_window_open(&self) -> bool {
        self.is_running()
            && self.phase.is_action_window()
            && self.choices.is_empty()
            && !self.players[self.active].skip_now
    }

    pub fn recycle_check(&self, id: &CardId) -> RuleResult {
        if !self.action_window_open() || self.players[self.active].recycled {
            return reject("Замена доступна один раз за ход: в подготовку или после исследования.");
        }
        if !self.players[self.active].holds(id) {
            return reject("Выберите карту с руки.");
        }
        Ok(())
    }

    pub fn recycle(&mut self, id: &CardId) -> RuleResult {
        self.recycle_check(id)?;
        let a = self.active;
        self.discard_from_hand(a, id);
        if self.ability(a, crate::cards::Effect::Elf) {
            let first = self.draw();
            let second = self.draw();
            let options: Vec<_> = first.into_iter().chain(second).map(ChoiceOption::Card).collect();
            if !options.is_empty() {
                self.choices.push_back(Choice {
                    player: a,
                    kind: ChoiceKind::Stash,
                    title: "Эльвира: выбери карту Барахла".into(),
                    options,
                });
            }
        } else {
            self.take(a, 1);
        }
        if self.has_perk(a, "recycler") {
            self.take(a, 1);
        }
        self.players[a].recycled = true;
        self.note(format!("{} заменяет одну карту.", self.player_name(a)));
        Ok(())
    }

    fn trade_partners(&self) -> Vec<ChoiceOption> {
        (0..self.players.len())
            .filter(|&i| i != self.active && !self.players[i].hand.is_empty())
            .map(ChoiceOption::Player)
            .collect()
    }

    pub fn trade_check(&self, id: &CardId) -> RuleResult {
        if !self.action_window_open() {
            return reject("Обмен доступен в подготовку или после исследования.");
        }
        if self.players[self.active].traded >= self.trade_limit(self.active) {
            return reject("Лимит обменов на этот ход исчерпан.");
        }
        if !self.players[self.active].holds(id) {
            return reject("Выберите предлагаемую карту с руки.");
        }
        if self.trade_partners().is_empty() {
            return reject("Ни у кого нет карт для обмена.");
        }
        Ok(())
    }

    /// Предложение обмена: получатель сам выбирает карту или отказывается.
    pub fn trade(&mut self, id: &CardId) -> RuleResult {
        self.trade_check(id)?;
        let title = format!("Кому предложить «{}»?", self.cards[id].name);
        self.choices.push_back(Choice {
            player: self.active,
            kind: ChoiceKind::TradeTarget { offered: id.clone() },
            title,
            options: self.trade_partners(),
        });
        Ok(())
    }

    pub fn discard_excess_check(&self, id: &CardId) -> RuleResult {
        let hand = &self.players[self.active].hand;
        if self.phase != Phase::Actions
            || !self.choices.is_empty()
            || hand.len() <= self.hand_limit(self.active)
            || !hand.contains(id)
        {
            return reject(format!(
                "Свободный сброс доступен при превышении лимита {}.",
                self.hand_limit(self.active)
            ));
        }
        Ok(())
    }

    pub fn discard_excess(&mut self, id: &CardId) -> RuleResult {
        self.discard_excess_check(id)?;
        self.discard_from_hand(self.active, id);
        Ok(())
    }

    pub fn end_turn_check(&self) -> RuleResult {
        if self.winner.is_some() || self.phase != Phase::Actions || !self.choices.is_empty() {
            return reject("Сначала завершите исследование и все эффекты.");
        }
        if self.players[self.active].hand.len() > self.hand_limit(self.active) {
            return reject(format!(
                "Сбросьте лишние карты до лимита {}.",
                self.hand_limit(self.active)
            ));
        }
        Ok(())
    }

    pub fn end_turn(&mut self) -> RuleResult {
        self.end_turn_check()?;
        self.active = (self.active + 1) % self.players.len();
        self.turn += 1;
        if self.active == self.first_player {
            self.rotate_district();
        }

        self.phase = Phase::Explore;
        self.fought = false;
        self.last_card = None;
        for p in &mut self.players {
            p.witch_used = false;
        }
        let next = &mut self.players[self.active];
        next.water = 0;
        next.recycled = false;
        next.traded = 0;
        next.shop_ops = 0;
        next.purchases = 0;
        next.sales = 0;
        next.cashback_used = false;
        next.safety_used = false;
        next.final_ready = next.level >= 9;
        next.skip_now = next.skip_next;
        next.skip_next = false;
        self.note(format!("Ход {}. {}.", self.turn, self.player_name(self.active)));
        Ok(())
    }
}
