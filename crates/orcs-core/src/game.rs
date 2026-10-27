//! Состояние партии и общие операции: раздача, добор, хроника, владелец решения.
//! Правила конкретных фаз — в соседних модулях `combat`, `effects`, `play`, `choices`, `turn`.

use std::collections::VecDeque;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::cards::{CardDb, CardDef, CardId, CardType, Effect, Slot};
use crate::rng::Pcg32;
use crate::state::{Boost, Choice, Phase, Player};

pub const MIN_PLAYERS: usize = 2;
pub const MAX_PLAYERS: usize = 4;
pub const STARTING_HAND: usize = 3;
pub const HAND_LIMIT: usize = 5;
pub const WIN_LEVEL: i32 = 10;
/// Максимальный уровень, который дают эффекты вне боя.
pub const NON_COMBAT_LEVEL_CAP: i32 = 9;
const JOURNAL_LIMIT: usize = 120;

/// Отказ правил. Текст показывается игроку как есть.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct Rejection(pub String);

pub type RuleResult = Result<(), Rejection>;

pub(crate) fn reject<T>(message: impl Into<String>) -> Result<T, Rejection> {
    Err(Rejection(message.into()))
}

/// Удаляет первое вхождение карты. Возвращает, была ли она там.
pub(crate) fn remove_card(cards: &mut Vec<CardId>, id: &CardId) -> bool {
    match cards.iter().position(|c| c == id) {
        Some(i) => {
            cards.remove(i);
            true
        }
        None => false,
    }
}

/// Чистый движок правил: без зависимостей от интерфейса, у каждого скрытого решения есть владелец.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Game {
    #[serde(skip, default = "CardDb::shared")]
    pub cards: Arc<CardDb>,
    pub players: Vec<Player>,
    /// Верх колоды — конец вектора.
    pub deck: Vec<CardId>,
    /// Верх сброса — конец вектора.
    pub discard: Vec<CardId>,
    pub trouble_deck: Vec<CardId>,
    pub trouble_discard: Vec<CardId>,
    pub market: Vec<CardId>,
    pub districts: Vec<CardId>,
    pub district: Option<CardId>,
    pub first_player: usize,
    pub help_for_coins: bool,
    pub active: usize,
    pub phase: Phase,
    pub turn: u32,
    pub encounter: Option<CardId>,
    pub last_card: Option<CardId>,
    /// Кто сейчас вмешивается в бой.
    pub reaction: usize,
    /// Сколько участников уже завершили вмешательство.
    pub reaction_step: usize,
    pub chaos_used: Vec<usize>,
    pub boosts: Vec<Boost>,
    pub enemy_mod: i32,
    pub weapon_mod: i32,
    pub battle_penalty: i32,
    pub lucky: bool,
    pub rusk: bool,
    pub helper: Option<usize>,
    pub help_requested: bool,
    pub fought: bool,
    pub choices: VecDeque<Choice>,
    pub journal: Vec<String>,
    pub winner: Option<usize>,
    pub rng: Pcg32,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    pub fn new() -> Self {
        Self::with_cards(CardDb::shared())
    }

    pub fn with_cards(cards: Arc<CardDb>) -> Self {
        Self {
            cards,
            players: Vec::new(),
            deck: Vec::new(),
            discard: Vec::new(),
            trouble_deck: Vec::new(),
            trouble_discard: Vec::new(),
            market: Vec::new(),
            districts: Vec::new(),
            district: None,
            first_player: 0,
            help_for_coins: false,
            active: 0,
            phase: Phase::Explore,
            turn: 1,
            encounter: None,
            last_card: None,
            reaction: 0,
            reaction_step: 0,
            chaos_used: Vec::new(),
            boosts: Vec::new(),
            enemy_mod: 0,
            weapon_mod: 0,
            battle_penalty: 0,
            lucky: false,
            rusk: false,
            helper: None,
            help_requested: false,
            fought: false,
            choices: VecDeque::new(),
            journal: Vec::new(),
            winner: None,
            rng: Pcg32::seeded(0),
        }
    }

    /// Новая партия. `seed == 0` — случайный сид.
    pub fn start<I, S>(&mut self, heroes: I, seed: u64) -> RuleResult
    where
        I: IntoIterator<Item = S>,
        S: Into<CardId>,
    {
        let heroes: Vec<CardId> = heroes.into_iter().map(Into::into).collect();
        if !(MIN_PLAYERS..=MAX_PLAYERS).contains(&heroes.len()) {
            return reject("За столом должно быть от 2 до 4 персонажей.");
        }
        for (i, hero) in heroes.iter().enumerate() {
            if self.cards.get(hero).map(|c| c.card_type) != Some(CardType::Character) {
                return reject(format!("«{hero}» — не персонаж."));
            }
            if heroes[..i].contains(hero) {
                return reject("Персонажи за столом не должны повторяться.");
            }
        }
        let cards = Arc::clone(&self.cards);
        *self = Self::with_cards(cards);
        self.rng = if seed == 0 {
            Pcg32::from_entropy()
        } else {
            Pcg32::seeded(seed)
        };
        self.players = heroes.into_iter().map(Player::new).collect();
        self.deck = self
            .cards
            .iter()
            .filter(|c| {
                matches!(
                    c.card_type,
                    CardType::Equipment | CardType::Consumable | CardType::Chaos
                )
            })
            .map(|c| c.id.clone())
            .collect();
        self.rng.shuffle(&mut self.deck);
        self.trouble_deck = self
            .cards
            .iter()
            .filter(|c| matches!(c.card_type, CardType::Enemy | CardType::Event) && c.effect != Effect::Boss)
            .map(|c| c.id.clone())
            .collect();
        self.rng.shuffle(&mut self.trouble_deck);
        self.districts = self
            .cards
            .iter()
            .filter(|c| c.card_type == CardType::District)
            .map(|c| c.id.clone())
            .collect();
        self.rng.shuffle(&mut self.districts);
        self.rotate_district();
        for p in 0..self.players.len() {
            self.take(p, STARTING_HAND);
        }
        self.active = self.rng.range_inclusive(0, self.players.len() - 1);
        self.first_player = self.active;
        self.refill_market();
        self.note(format!(
            "Партия началась. Первый ход: {}.",
            self.player_name(self.active)
        ));
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        !self.players.is_empty() && self.winner.is_none()
    }

    pub fn card(&self, id: &CardId) -> &CardDef {
        &self.cards[id]
    }

    pub fn player_name(&self, p: usize) -> &str {
        &self.cards[&self.players[p].hero].name
    }

    /// Верхняя карта колоды; пустая колода сначала замешивает сброс.
    pub fn draw(&mut self) -> Option<CardId> {
        if self.deck.is_empty() && !self.discard.is_empty() {
            self.deck = std::mem::take(&mut self.discard);
            self.rng.shuffle(&mut self.deck);
            self.note("Сброс перемешан в новую колоду.");
        }
        self.deck.pop()
    }

    pub fn take(&mut self, p: usize, count: usize) {
        for _ in 0..count {
            if let Some(id) = self.draw() {
                self.players[p].hand.push(id);
            }
        }
    }

    pub fn note(&mut self, message: impl Into<String>) {
        self.journal.push(message.into());
        if self.journal.len() > JOURNAL_LIMIT {
            self.journal.remove(0);
        }
    }

    /// Чьё решение сейчас ожидается — тот и видит руку.
    pub fn owner(&self) -> usize {
        if let Some(q) = self.choices.front() {
            return q.player;
        }
        if self.phase == Phase::Combat {
            return self.reaction;
        }
        self.active
    }

    pub fn ability(&self, p: usize, effect: Effect) -> bool {
        let player = &self.players[p];
        !player.silenced && self.cards[&player.hero].effect == effect
    }

    pub fn technical(&self, p: usize) -> bool {
        Slot::ALL
            .iter()
            .filter_map(|&slot| self.players[p].gear(slot))
            .any(|id| self.cards[id].technical)
    }

    pub fn trade_limit(&self, p: usize) -> u32 {
        if self.ability(p, Effect::Merchant) { 2 } else { 1 }
    }

    /// Все карты, которые сейчас где-то лежат: колода, сброс, руки, слоты, бой и выборы «из колоды».
    pub fn cards_in_play(&self) -> Vec<CardId> {
        let mut held: Vec<CardId> = self
            .deck
            .iter()
            .chain(&self.discard)
            .chain(&self.trouble_deck)
            .chain(&self.trouble_discard)
            .chain(&self.market)
            .cloned()
            .collect();
        for p in &self.players {
            held.extend(p.hand.iter().cloned());
            held.extend(p.weapon.iter().cloned());
            held.extend(p.armor.iter().cloned());
        }
        held.extend(
            self.encounter
                .iter()
                .filter(|id| self.cards[*id].effect != Effect::Boss)
                .cloned(),
        );
        for q in &self.choices {
            if matches!(
                q.kind,
                crate::state::ChoiceKind::Portal | crate::state::ChoiceKind::Stash
            ) {
                held.extend(q.options.iter().filter_map(|o| match o {
                    crate::state::ChoiceOption::Card(id) => Some(id.clone()),
                    _ => None,
                }));
            }
        }
        held
    }
}
