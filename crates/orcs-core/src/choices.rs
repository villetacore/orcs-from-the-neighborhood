//! Применение отложенных решений из очереди `Game::choices`.

use crate::cards::Slot;
use crate::game::{Game, RuleResult, reject, remove_card};
use crate::state::{Choice, ChoiceKind, ChoiceOption};

impl Game {
    pub fn current_choice(&self) -> Option<&Choice> {
        self.choices.front()
    }

    /// Владелец первого решения выбирает один из предложенных вариантов.
    pub fn choose(&mut self, option: ChoiceOption) -> RuleResult {
        let Some(q) = self.choices.front() else {
            return reject("Нет активного выбора.");
        };
        if !q.options.contains(&option) {
            return reject("Недопустимый выбор.");
        }
        let q = self.choices.pop_front().expect("выбор проверен выше");
        let p = q.player;
        match (&q.kind, option) {
            (ChoiceKind::AfterLoss, _) => self.take(p, 1),
            (ChoiceKind::Perk { tier }, ChoiceOption::Card(id)) => {
                self.players[p].perks.push(id);
                self.note(format!("{} выбирает перк ступени {}.", self.player_name(p), tier));
            }
            (ChoiceKind::Scout, ChoiceOption::Bottom) => {
                if let Some(id) = self.trouble_deck.pop() {
                    self.trouble_deck.insert(0, id);
                }
            }
            (ChoiceKind::Water, ChoiceOption::Penalty) => self.players[p].water = -1,
            (ChoiceKind::Pipe, ChoiceOption::Penalty) => self.battle_penalty -= 1,
            (ChoiceKind::Discard | ChoiceKind::Water | ChoiceKind::Pipe, ChoiceOption::Card(id)) => {
                self.discard_from_hand(p, &id);
                self.note(format!(
                    "{} сбрасывает «{}».",
                    self.player_name(p),
                    self.cards[&id].name
                ));
            }
            (kind @ (ChoiceKind::Portal | ChoiceKind::Stash), ChoiceOption::Card(kept)) => {
                self.players[p].hand.push(kept.clone());
                for other in q.options.iter().filter_map(|o| match o {
                    ChoiceOption::Card(id) if *id != kept => Some(id.clone()),
                    _ => None,
                }) {
                    if *kind == ChoiceKind::Stash {
                        // Низ колоды — начало вектора.
                        self.deck.insert(0, other);
                    } else {
                        self.discard_card(other);
                    }
                }
            }
            (ChoiceKind::Silence, ChoiceOption::Player(target)) => {
                self.players[target].silenced = true;
                self.note(format!("{} временно лишается способности.", self.player_name(target)));
            }
            (ChoiceKind::Ambush, ChoiceOption::Player(target)) => self.lose_card(target),
            (ChoiceKind::Witch, ChoiceOption::Random) => self.random_loss(p),
            (ChoiceKind::Witch, ChoiceOption::Card(id)) => {
                self.players[p].witch_used = true;
                self.discard_from_hand(p, &id);
            }
            (ChoiceKind::Protect, ChoiceOption::Protect) => self.remove_gear(p, Slot::Armor),
            (ChoiceKind::Protect, _) => self.random_loss(p),
            (ChoiceKind::SecondLoss, _) => self.lose_card(p),
            (ChoiceKind::HelpTarget, ChoiceOption::Player(target)) => self.offer_help_to(target),
            (ChoiceKind::HelpAccept, ChoiceOption::Card(payment)) => self.accept_help(p, &payment),
            (ChoiceKind::HelpAccept, _) => self.note(format!("{} отказывается помогать.", self.player_name(p))),
            (ChoiceKind::TradeTarget { offered }, ChoiceOption::Player(target)) => {
                let mut options = ChoiceOption::cards(&self.players[target].hand);
                options.push(ChoiceOption::Decline);
                let title = format!(
                    "{} предлагает «{}». Выберите свою карту или откажитесь.",
                    self.player_name(p),
                    self.cards[offered].name
                );
                self.choices.push_front(Choice {
                    player: target,
                    kind: ChoiceKind::TradeAccept {
                        from: p,
                        offered: offered.clone(),
                    },
                    title,
                    options,
                });
            }
            (ChoiceKind::TradeAccept { from, offered }, ChoiceOption::Card(given)) => {
                let from = *from;
                remove_card(&mut self.players[p].hand, &given);
                remove_card(&mut self.players[from].hand, offered);
                self.players[p].hand.push(offered.clone());
                self.players[from].hand.push(given);
                self.players[from].traded += 1;
                self.note("Игроки обменялись картами.");
            }
            (ChoiceKind::TradeAccept { .. }, _) => self.note("Обмен отклонён."),
            // Reveal — только просмотр; HelpTarget/Cancel — предложение не отправлено.
            _ => {}
        }
        self.queue_perks();
        self.finish_victory();
        Ok(())
    }

    /// Человекочитаемая подпись варианта для интерфейса.
    pub fn option_label(&self, choice: &Choice, option: &ChoiceOption) -> String {
        match option {
            ChoiceOption::Card(id) => {
                let card = self.card(id);
                format!("{} — {}", card.name, card.text)
            }
            ChoiceOption::Player(p) => match choice.kind {
                ChoiceKind::Silence | ChoiceKind::Ambush | ChoiceKind::TradeTarget { .. } | ChoiceKind::HelpTarget => {
                    self.player_name(*p).to_owned()
                }
                _ => p.to_string(),
            },
            ChoiceOption::Bottom => "Положить вниз Разборок".into(),
            ChoiceOption::Penalty => "Принять −1 силы".into(),
            ChoiceOption::Protect => "Сбросить плащ, сохранить руку".into(),
            ChoiceOption::LoseRandom => "Потерять случайную карту".into(),
            ChoiceOption::Random => "Обычная случайная потеря".into(),
            ChoiceOption::Decline => "Отказаться".into(),
            ChoiceOption::Cancel => "Не предлагать помощь".into(),
            ChoiceOption::Ok => "Готово / продолжить".into(),
        }
    }
}
