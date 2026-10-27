//! Две колоды, районы, торговля и развитие. Все платежи проходят после проверок.
use crate::cards::{CardId, CardType, Effect};
use crate::game::{Game, RuleResult, reject};
use crate::state::{Boost, BoostSource, Choice, ChoiceKind, ChoiceOption, Phase};

impl Game {
    pub fn active_perks(&self, p: usize) -> impl Iterator<Item = &CardId> {
        self.players[p]
            .perks
            .iter()
            .filter(move |id| self.cards[*id].tier <= self.players[p].level)
    }

    pub fn has_perk(&self, p: usize, key: &str) -> bool {
        let id = format!("perk_{key}");
        self.active_perks(p).any(|perk| perk.as_str() == id)
    }

    pub fn hand_limit(&self, p: usize) -> usize {
        if self.has_perk(p, "pockets") { 6 } else { 5 }
    }

    pub fn district_is(&self, id: &str) -> bool {
        self.district.as_ref().is_some_and(|d| d.as_str() == id)
    }

    pub(crate) fn rotate_district(&mut self) {
        if let Some(old) = self.district.take() {
            self.districts.insert(0, old);
        }
        self.district = self.districts.pop();
        for p in &mut self.players {
            p.helper_paid_round = false;
        }
        if let Some(id) = &self.district {
            self.note(format!("Район: {}. {}", self.cards[id].name, self.cards[id].text));
        }
    }

    pub(crate) fn discard_card(&mut self, id: CardId) {
        if matches!(self.cards[&id].card_type, CardType::Enemy | CardType::Event) {
            self.trouble_discard.push(id);
        } else {
            self.discard.push(id);
        }
    }

    pub(crate) fn draw_trouble(&mut self) -> Option<CardId> {
        if self.trouble_deck.is_empty() && !self.trouble_discard.is_empty() {
            self.trouble_deck = std::mem::take(&mut self.trouble_discard);
            self.rng.shuffle(&mut self.trouble_deck);
            self.note("Сброс Разборок перемешан.");
        }
        self.trouble_deck.pop()
    }

    pub(crate) fn refill_market(&mut self) {
        while self.market.len() < 3 {
            let Some(id) = self.draw() else { break };
            self.market.push(id);
        }
    }

    pub fn shop_limit(&self) -> u32 {
        if self.has_perk(self.active, "auction") { 3 } else { 2 }
    }

    pub fn shop_check(&self) -> RuleResult {
        if !self.action_window_open() {
            return reject("Магазин доступен в подготовку и после исследования, вне выбора.");
        }
        if self.players[self.active].shop_ops >= self.shop_limit() {
            return reject("Операции магазина на этот ход закончились.");
        }
        Ok(())
    }

    pub fn purchase_price(&self, id: &CardId, barter: bool) -> u32 {
        let first = self.players[self.active].purchases == 0;
        let discount = u32::from(first && self.has_perk(self.active, "haggle"))
            + u32::from(first && self.district_is("district_market"))
            + u32::from(barter);
        self.cards[id].price.saturating_sub(discount).max(1)
    }

    pub fn buy_check(&self, id: &CardId, payment: Option<&CardId>) -> RuleResult {
        self.shop_check()?;
        if !self.market.contains(id) {
            return reject("Выберите карту витрины.");
        }
        if let Some(paid) = payment
            && (!self.players[self.active].holds(paid) || self.cards[paid].price == 0)
        {
            return reject("Для бартера выберите Барахло с руки.");
        }
        if self.players[self.active].coins < self.purchase_price(id, payment.is_some()) {
            return reject("Не хватает крышаков.");
        }
        Ok(())
    }

    pub fn buy(&mut self, id: &CardId, payment: Option<&CardId>) -> RuleResult {
        self.buy_check(id, payment)?;
        let a = self.active;
        let price = self.purchase_price(id, payment.is_some());
        self.players[a].coins -= price;
        if let Some(paid) = payment {
            self.discard_from_hand(a, paid);
        }
        self.market.retain(|c| c != id);
        self.players[a].hand.push(id.clone());
        self.players[a].shop_ops += 1;
        self.players[a].purchases += 1;
        if payment.is_none() && self.has_perk(a, "cashback") && !self.players[a].cashback_used {
            self.players[a].coins += price.saturating_sub(1).min(1);
            self.players[a].cashback_used = true;
        }
        self.refill_market();
        self.note(format!(
            "{} покупает «{}» за {} кр.",
            self.player_name(a),
            self.cards[id].name,
            price
        ));
        Ok(())
    }

    pub fn sell_check(&self, id: &CardId) -> RuleResult {
        self.shop_check()?;
        let p = &self.players[self.active];
        if !(p.holds(id) || p.weapon.as_ref() == Some(id) || p.armor.as_ref() == Some(id))
            || self.cards.get(id).is_none_or(|c| c.price == 0)
        {
            return reject("Продать можно своё Барахло.");
        }
        Ok(())
    }

    pub fn sell(&mut self, id: &CardId) -> RuleResult {
        self.sell_check(id)?;
        let a = self.active;
        if self.players[a].holds(id) {
            self.discard_from_hand(a, id);
        } else {
            if self.players[a].weapon.as_ref() == Some(id) {
                self.players[a].weapon = None;
            }
            if self.players[a].armor.as_ref() == Some(id) {
                self.players[a].armor = None;
            }
            self.discard_card(id.clone());
        }
        self.players[a].coins += 1;
        self.players[a].sales += 1;
        self.players[a].shop_ops += 1;
        self.note(format!(
            "{} продаёт «{}» за 1 крышак.",
            self.player_name(a),
            self.cards[id].name
        ));
        Ok(())
    }

    pub fn refresh_market(&mut self, id: &CardId) -> RuleResult {
        self.shop_check()?;
        if !self.market.contains(id) {
            return reject("Выберите карту витрины.");
        }
        let a = self.active;
        if self.players[a].coins < 1 {
            return reject("Обновление стоит 1 крышак.");
        }
        self.players[a].coins -= 1;
        self.players[a].shop_ops += 1;
        self.market.retain(|c| c != id);
        // Убираем выбранную карту до добора, чтобы обновление не вернуло её сразу.
        self.refill_market();
        self.discard_card(id.clone());
        self.note("Витрина обновлена за 1 крышак.");
        Ok(())
    }

    pub fn scout_price(&self) -> u32 {
        if self.district_is("district_mall") { 1 } else { 2 }
    }

    pub fn scout(&mut self) -> RuleResult {
        self.shop_check()?;
        let a = self.active;
        let price = self.scout_price();
        if self.players[a].coins < price {
            return reject("Не хватает денег на разведку.");
        }
        let Some(id) = self.draw_trouble() else {
            return reject("В Разборках не осталось карт.");
        };
        self.trouble_deck.push(id.clone());
        self.players[a].coins -= price;
        self.players[a].shop_ops += 1;
        self.choices.push_back(Choice {
            player: a,
            kind: ChoiceKind::Scout,
            title: format!(
                "Разведка: {}. {} Оставить сверху или положить вниз?",
                self.cards[&id].name, self.cards[&id].text
            ),
            options: vec![ChoiceOption::Ok, ChoiceOption::Bottom],
        });
        // Содержимое не попадает в публичную хронику.
        self.note(format!("{} покупает слух за {} кр.", self.player_name(a), price));
        Ok(())
    }

    pub fn work_check(&self) -> RuleResult {
        if !self.is_running() || self.phase != Phase::Explore || !self.choices.is_empty() {
            return reject("Подработка заменяет исследование в начале хода.");
        }
        Ok(())
    }

    pub fn work(&mut self) -> RuleResult {
        self.work_check()?;
        let a = self.active;
        let pay = 2 + u32::from(self.has_perk(a, "worker")) + u32::from(self.district_is("district_swamp"));
        self.players[a].coins += pay;
        if self.has_perk(a, "failure") {
            self.take(a, 1);
        }
        self.fought = true; // В этот ход нет вызова врага с руки.
        self.phase = Phase::Actions;
        self.note(format!(
            "{} подрабатывает: +{} кр., без исследования и боя.",
            self.player_name(a),
            pay
        ));
        Ok(())
    }

    pub fn challenge_check(&self) -> RuleResult {
        self.work_check()?;
        if self.players[self.active].level < 9 || !self.players[self.active].final_ready {
            return reject("Хозяина района можно вызвать с начала своего следующего хода после достижения уровня 9.");
        }
        Ok(())
    }

    pub fn challenge(&mut self) -> RuleResult {
        self.challenge_check()?;
        self.begin_battle(CardId::from("boss"));
        Ok(())
    }

    pub(crate) fn queue_perks(&mut self) {
        // Сначала полностью заканчиваем штрафы и добор, потом предлагаем развитие.
        if !self.choices.is_empty() || self.phase == Phase::Combat {
            return;
        }
        for p in 0..self.players.len() {
            for tier in [3, 5, 7] {
                if self.players[p].level < tier || self.players[p].unlocked_tiers.contains(&tier) {
                    continue;
                }
                let options = self
                    .cards
                    .iter()
                    .filter(|c| {
                        c.card_type == CardType::Perk
                            && c.tier == tier
                            && !self.players.iter().any(|owner| owner.perks.contains(&c.id))
                    })
                    .map(|c| ChoiceOption::Card(c.id.clone()))
                    .collect::<Vec<_>>();
                self.players[p].unlocked_tiers.push(tier);
                if !options.is_empty() {
                    self.choices.push_back(Choice {
                        player: p,
                        kind: ChoiceKind::Perk { tier },
                        title: format!("Ступень {tier}: выбери перк. Карта остаётся у тебя до конца партии."),
                        options,
                    });
                    return;
                }
            }
        }
    }

    pub fn help_payment_text(&self) -> String {
        if self.help_for_coins {
            format!(
                "Плата сразу: {} кр.; добыча остаётся герою.",
                if self.has_perk(self.active, "contacts") { 1 } else { 2 }
            )
        } else {
            "Плата: первая карта добычи при победе.".into()
        }
    }

    pub fn request_paid_help(&mut self) -> RuleResult {
        self.help_check()?;
        let price = if self.has_perk(self.active, "contacts") { 1 } else { 2 };
        if self.players[self.active].coins < price {
            return reject("Не хватает денег для помощника.");
        }
        self.help_for_coins = true;
        self.queue_help_target();
        Ok(())
    }

    pub(crate) fn pay_helper_perk(&mut self, p: usize) {
        if self.has_perk(p, "helper") && !self.players[p].helper_paid_round {
            self.players[p].coins += 1;
            self.players[p].helper_paid_round = true;
        }
    }

    pub fn paid_boost_check(&self) -> RuleResult {
        let a = self.active;
        if !self.is_running() || self.phase != Phase::Combat || !self.choices.is_empty() || self.owner() != a {
            return reject("Платный аргумент доступен в своём окне боя.");
        }
        let price = if self.ability(a, Effect::Djinn) {
            1
        } else if self.has_perk(a, "last_word") {
            3
        } else {
            return reject("Нужна способность джинна или перк «Последний аргумент».");
        };
        if self.players[a].paid_boost_used {
            return reject("Платный аргумент уже использован в этом бою.");
        }
        if self.players[a].coins < price {
            return reject("Не хватает крышаков.");
        }
        Ok(())
    }

    pub fn paid_boost(&mut self) -> RuleResult {
        self.paid_boost_check()?;
        let a = self.active;
        let price = if self.ability(a, Effect::Djinn) { 1 } else { 3 };
        self.players[a].coins -= price;
        self.players[a].paid_boost_used = true;
        self.boosts.push(Boost {
            amount: 2,
            source: BoostSource::Paid,
        });
        self.note(format!("{} платит {} кр. за +2 силы.", self.player_name(a), price));
        Ok(())
    }
}
