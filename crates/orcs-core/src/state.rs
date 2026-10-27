//! Типы состояния партии: игроки, фазы, временные бонусы и отложенные решения.

use serde::{Deserialize, Serialize};

use crate::cards::{CardId, Slot};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    /// Подготовка до исследования.
    Explore,
    /// Бой и очередь вмешательства.
    Combat,
    /// Действия после исследования.
    Actions,
}

impl Phase {
    /// Подготовка или действия: окна, где разрешены обмен и замена.
    pub fn is_action_window(self) -> bool {
        matches!(self, Phase::Explore | Phase::Actions)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Player {
    pub hero: CardId,
    pub level: i32,
    pub hand: Vec<CardId>,
    pub weapon: Option<CardId>,
    pub armor: Option<CardId>,
    /// «Нет горячей воды»: −1 силы до начала собственного хода.
    pub water: i32,
    /// Штраф к следующему бою («Проклятый лифт» без карт на руке).
    pub next_penalty: i32,
    pub silenced: bool,
    pub witch_used: bool,
    pub skip_next: bool,
    pub skip_now: bool,
    pub recycled: bool,
    pub traded: u32,
    pub coins: u32,
    pub perks: Vec<CardId>,
    pub unlocked_tiers: Vec<i32>,
    pub shop_ops: u32,
    pub purchases: u32,
    pub sales: u32,
    pub cashback_used: bool,
    pub helper_paid_round: bool,
    pub safety_used: bool,
    pub paid_boost_used: bool,
    pub reserve_used: bool,
    pub final_ready: bool,
}

impl Player {
    pub fn new(hero: CardId) -> Self {
        Self {
            hero,
            level: 1,
            hand: Vec::new(),
            weapon: None,
            armor: None,
            water: 0,
            next_penalty: 0,
            silenced: false,
            witch_used: false,
            skip_next: false,
            skip_now: false,
            recycled: false,
            traded: 0,
            coins: 4,
            perks: Vec::new(),
            unlocked_tiers: Vec::new(),
            shop_ops: 0,
            purchases: 0,
            sales: 0,
            cashback_used: false,
            helper_paid_round: false,
            safety_used: false,
            paid_boost_used: false,
            reserve_used: false,
            final_ready: false,
        }
    }

    pub fn gear(&self, slot: Slot) -> Option<&CardId> {
        match slot {
            Slot::Weapon => self.weapon.as_ref(),
            Slot::Armor => self.armor.as_ref(),
        }
    }

    pub fn gear_mut(&mut self, slot: Slot) -> &mut Option<CardId> {
        match slot {
            Slot::Weapon => &mut self.weapon,
            Slot::Armor => &mut self.armor,
        }
    }

    pub fn holds(&self, id: &CardId) -> bool {
        self.hand.contains(id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoostSource {
    Card(CardId),
    /// Договор помощи: +2 за сброшенную помощником карту.
    HelpContract,
    Paid,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Boost {
    pub amount: i32,
    pub source: BoostSource,
}

/// Вид отложенного решения. Данные варианта — то, что нужно для его применения.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ChoiceKind {
    Pipe,
    Water,
    Portal,
    Stash,
    Silence,
    Reveal { subject: usize },
    Discard,
    Protect,
    Witch,
    SecondLoss,
    Ambush,
    HelpTarget,
    HelpAccept,
    Perk { tier: i32 },
    Scout,
    AfterLoss,
    TradeTarget { offered: CardId },
    TradeAccept { from: usize, offered: CardId },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChoiceOption {
    Card(CardId),
    Player(usize),
    /// Принять −1 силы вместо сброса.
    Penalty,
    Bottom,
    /// Сбросить плащ вместо случайной карты.
    Protect,
    /// Отказаться от защиты плаща: потерять случайную карту.
    LoseRandom,
    /// Зина не вмешивается: обычная случайная потеря.
    Random,
    Decline,
    Cancel,
    Ok,
}

impl ChoiceOption {
    pub fn card(id: impl Into<CardId>) -> Self {
        ChoiceOption::Card(id.into())
    }

    pub fn cards(ids: &[CardId]) -> Vec<Self> {
        ids.iter().cloned().map(ChoiceOption::Card).collect()
    }
}

/// Решение, которое должен принять конкретный игрок до продолжения партии.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Choice {
    pub player: usize,
    #[serde(flatten)]
    pub kind: ChoiceKind,
    pub title: String,
    pub options: Vec<ChoiceOption>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StrengthPart {
    pub label: String,
    pub amount: i32,
}
