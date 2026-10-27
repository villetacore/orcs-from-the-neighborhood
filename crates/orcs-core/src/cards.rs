//! Каталог карт. Единственный источник данных — `data/cards.json`, он встраивается в бинарник.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, OnceLock};

use serde::{Deserialize, Serialize};

/// Канонический JSON колоды. Пересобирается `tools/build_deck.py`.
pub const EMBEDDED_CARDS: &str = include_str!("../../../data/cards.json");

/// Состав каталога: 90 карт, из них 56 в двух основных колодах.
pub const DECK_COMPOSITION: [(CardType, usize); 9] = [
    (CardType::Character, 8),
    (CardType::Enemy, 19),
    (CardType::Equipment, 12),
    (CardType::Consumable, 6),
    (CardType::Event, 10),
    (CardType::Chaos, 10),
    (CardType::Perk, 18),
    (CardType::District, 6),
    (CardType::Shop, 1),
];

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CardId(String);

impl CardId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for CardId {
    fn from(id: &str) -> Self {
        Self::new(id)
    }
}

impl From<&CardId> for CardId {
    fn from(id: &CardId) -> Self {
        id.clone()
    }
}

impl PartialEq<&str> for CardId {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardType {
    Character,
    Enemy,
    Equipment,
    Consumable,
    Event,
    Chaos,
    Perk,
    District,
    Shop,
}

impl CardType {
    pub const ALL: [CardType; 9] = [
        CardType::Character,
        CardType::Enemy,
        CardType::Equipment,
        CardType::Consumable,
        CardType::Event,
        CardType::Chaos,
        CardType::Perk,
        CardType::District,
        CardType::Shop,
    ];

    /// Подпись категории на лице карты.
    pub fn title(self) -> &'static str {
        match self {
            CardType::Character => "ПЕРСОНАЖ",
            CardType::Enemy => "ВРАГ",
            CardType::Equipment => "СНАРЯЖЕНИЕ",
            CardType::Consumable => "РАСХОДНИК",
            CardType::Event => "СОБЫТИЕ",
            CardType::Chaos => "ПОДСТАВА",
            CardType::Perk => "ПЕРК",
            CardType::District => "РАЙОН",
            CardType::Shop => "МАГАЗИН",
        }
    }
}

/// Все игровые эффекты. Значение в JSON — snake_case имя варианта.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    // Способности героев.
    Mechanic,
    MutantHunter,
    Merchant,
    Witch,
    Elf,
    Dwarf,
    Djinn,
    Oni,
    Boss,
    // Свойства врагов.
    None,
    Unarmored,
    Pipe,
    Armor,
    Dragon,
    Technical,
    // Свойства снаряжения.
    Undead,
    VictoryDiscard,
    Protect,
    // Расходники.
    Boost,
    Cycle,
    Tape,
    Luck,
    Energy,
    Rusk,
    // События.
    Water,
    Portal,
    Silence,
    Lift,
    Raid,
    Reveal,
    Draw,
    Rust,
    Harvest,
    Tax,
    // Хаос.
    EnemyUp,
    WeaponDown,
    Ambush,
    Friend,
    Stash,
    Cancel,
    Battery,
    EnemyDown,
    HeroUp,
    Noise,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tag {
    Undead,
    Orc,
    Spirit,
    Mutant,
    Beast,
    Human,
    Construct,
}

/// Последствие поражения от врага.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Penalty {
    Card,
    Level,
    Armor,
    Weapon,
    TwoCards,
    LevelCard,
    LevelCoins,
    Coins,
}

impl Penalty {
    pub fn description(self) -> &'static str {
        match self {
            Penalty::Card => "потеря случайной карты.",
            Penalty::Level => "−1 уровень.",
            Penalty::Armor => "сброс брони; без брони — потеря карты.",
            Penalty::Weapon => "сброс оружия; без оружия — потеря карты.",
            Penalty::TwoCards => "потеря двух карт.",
            Penalty::LevelCard => "−1 уровень и потеря карты.",
            Penalty::LevelCoins => "−1 уровень и до 2 крышаков.",
            Penalty::Coins => "потеря до 2 крышаков.",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Slot {
    Weapon,
    Armor,
}

impl Slot {
    pub const ALL: [Slot; 2] = [Slot::Weapon, Slot::Armor];

    pub fn title(self) -> &'static str {
        match self {
            Slot::Weapon => "Оружие",
            Slot::Armor => "Броня",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CardDef {
    pub id: CardId,
    pub name: String,
    #[serde(rename = "type")]
    pub card_type: CardType,
    pub text: String,
    /// Путь к иллюстрации относительно корня проекта.
    pub image: String,
    /// Описание сюжета иллюстрации (для промптов).
    pub subject: String,
    pub effect: Effect,
    #[serde(default)]
    pub strength: i32,
    #[serde(default)]
    pub tag: Option<Tag>,
    #[serde(default)]
    pub level_reward: i32,
    #[serde(default)]
    pub loot_cards: u32,
    #[serde(default)]
    pub penalty: Option<Penalty>,
    #[serde(default)]
    pub bonus: i32,
    #[serde(default)]
    pub slot: Option<Slot>,
    #[serde(default)]
    pub technical: bool,
    #[serde(default)]
    pub price: u32,
    #[serde(default)]
    pub coins: u32,
    #[serde(default)]
    pub flavor: String,
    #[serde(default)]
    pub tier: i32,
}

impl CardDef {
    /// Текст правила на лице карты. У врага к нему добавляется последствие поражения,
    /// как на печатной карте (`tools/build_cards.py`).
    pub fn rules_text(&self) -> String {
        match self.penalty {
            Some(penalty) if self.card_type == CardType::Enemy => {
                format!("{} При поражении: {}", self.text, penalty.description())
            }
            _ => self.text.clone(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    #[error("не удалось разобрать каталог карт: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("повторяющийся id карты: {0}")]
    Duplicate(CardId),
    #[error("у карты {0} не хватает поля {1}")]
    Missing(CardId, &'static str),
}

/// Неизменяемый каталог карт в порядке файла данных.
#[derive(Debug)]
pub struct CardDb {
    cards: Vec<CardDef>,
    index: HashMap<CardId, usize>,
}

impl CardDb {
    pub fn from_json(json: &str) -> Result<Self, CatalogError> {
        let cards: Vec<CardDef> = serde_json::from_str(json)?;
        let mut index = HashMap::with_capacity(cards.len());
        for (i, card) in cards.iter().enumerate() {
            let missing = match card.card_type {
                CardType::Enemy if card.penalty.is_none() => Some("penalty"),
                CardType::Enemy if card.tag.is_none() => Some("tag"),
                CardType::Equipment if card.slot.is_none() => Some("slot"),
                _ => None,
            };
            if let Some(field) = missing {
                return Err(CatalogError::Missing(card.id.clone(), field));
            }
            if index.insert(card.id.clone(), i).is_some() {
                return Err(CatalogError::Duplicate(card.id.clone()));
            }
        }
        Ok(Self { cards, index })
    }

    /// Общий экземпляр встроенного каталога.
    pub fn shared() -> Arc<CardDb> {
        static DB: OnceLock<Arc<CardDb>> = OnceLock::new();
        DB.get_or_init(|| Arc::new(CardDb::from_json(EMBEDDED_CARDS).expect("data/cards.json валиден")))
            .clone()
    }

    pub fn get(&self, id: &CardId) -> Option<&CardDef> {
        self.index.get(id).map(|&i| &self.cards[i])
    }

    pub fn contains(&self, id: &CardId) -> bool {
        self.index.contains_key(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &CardDef> {
        self.cards.iter()
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    pub fn heroes(&self) -> impl Iterator<Item = &CardDef> {
        self.iter().filter(|c| c.card_type == CardType::Character)
    }
}

impl std::ops::Index<&CardId> for CardDb {
    type Output = CardDef;

    fn index(&self, id: &CardId) -> &CardDef {
        self.get(id).unwrap_or_else(|| panic!("неизвестная карта {id}"))
    }
}
